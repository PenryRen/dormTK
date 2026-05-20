use crate::{
    auth::{AuthUser, encode_token, hash_password, verify_password},
    db::{enum_value, opt_text, text, validate_non_empty},
    error::ApiError,
    http::{ApiJson, PageJson, Pagination, data, page, pagination},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put},
};
use dormtk_api_types::{
    access::{
        ImportJobCreate, Permission, PermissionIdsRequest, PermissionUpsert, Role, RoleIdsRequest,
        RoleUpsert, User, UserCreate, UserUpdate,
    },
    auth::{LoginRequest, LoginResponse, LoginUser},
};
use dormtk_core::{Id, UserStatus};
use sqlx::{PgPool, Row};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/admin/users", get(list_users).post(create_user))
        .route("/api/admin/users/{id}", get(get_user).patch(update_user))
        .route("/api/admin/users/{id}/roles", put(replace_user_roles))
        .route("/api/admin/roles", get(list_roles).post(create_role))
        .route("/api/admin/roles/{id}", get(get_role).patch(update_role))
        .route(
            "/api/admin/roles/{id}/permissions",
            put(replace_role_permissions),
        )
        .route(
            "/api/admin/permissions",
            get(list_permissions).post(create_permission),
        )
        .route(
            "/api/admin/permissions/{id}",
            get(get_permission).patch(update_permission),
        )
        .route("/api/admin/import-jobs", post(reserve_import_job))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<ApiJson<LoginResponse>, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            username,
            password_hash,
            status
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(payload.username.trim())
    .fetch_optional(&state.db)
    .await?;

    let row = row.ok_or_else(|| ApiError::unauthorized("invalid username or password"))?;
    let status: UserStatus = enum_value(&row, "status")?;
    if status != UserStatus::Active {
        return Err(ApiError::unauthorized("user is disabled"));
    }

    let password_hash: String = row.try_get("password_hash")?;
    if !verify_password(&payload.password, &password_hash)? {
        return Err(ApiError::unauthorized("invalid username or password"));
    }

    let user_id = text(&row, "id")?;
    let role_refs = role_refs_for_user(&state.db, &user_id).await?;
    let role_claim = role_refs
        .iter()
        .map(|role| role.code.as_str())
        .next()
        .unwrap_or("user");
    let access_token = encode_token(&user_id, role_claim, &state.config.jwt)?;

    sqlx::query("UPDATE users SET last_login_at = now(), updated_at = now() WHERE id = $1::uuid")
        .bind(&user_id)
        .execute(&state.db)
        .await?;

    Ok(data(LoginResponse {
        access_token,
        user: LoginUser {
            id: user_id,
            username: row.try_get("username")?,
            role_ids: role_refs.iter().map(|role| role.id.clone()).collect(),
            roles: role_refs.into_iter().map(|role| role.code).collect(),
        },
    }))
}

async fn list_users(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<User>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            username,
            phone,
            status,
            last_login_at::text AS last_login_at,
            created_at::text AS created_at,
            updated_at::text AS updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(user_from_row(&state.db, row).await?);
    }

    Ok(page(items, pagination, total))
}

async fn create_user(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<UserCreate>,
) -> Result<ApiJson<User>, ApiError> {
    validate_non_empty(&payload.username, "username")?;
    validate_non_empty(&payload.password, "password")?;
    let password_hash = hash_password(&payload.password)?;
    let status = payload.status.unwrap_or(UserStatus::Active);

    let row = sqlx::query(
        r#"
        INSERT INTO users (username, password_hash, phone, status)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id::text AS id,
            username,
            phone,
            status,
            last_login_at::text AS last_login_at,
            created_at::text AS created_at,
            updated_at::text AS updated_at
        "#,
    )
    .bind(payload.username.trim())
    .bind(password_hash)
    .bind(payload.phone)
    .bind(enum_string(status)?)
    .fetch_one(&state.db)
    .await?;

    let user = user_from_row(&state.db, row).await?;
    if let Some(role_ids) = payload.role_ids {
        replace_user_roles_inner(&state.db, &user.id, &role_ids).await?;
        return Ok(data(fetch_user(&state.db, &user.id).await?));
    }

    Ok(data(user))
}

async fn get_user(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<User>, ApiError> {
    Ok(data(fetch_user(&state.db, &id).await?))
}

async fn update_user(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<UserUpdate>,
) -> Result<ApiJson<User>, ApiError> {
    if let Some(username) = &payload.username {
        validate_non_empty(username, "username")?;
    }
    if let Some(password) = &payload.password {
        validate_non_empty(password, "password")?;
    }

    let password_hash = match payload.password {
        Some(password) => Some(hash_password(&password)?),
        None => None,
    };

    sqlx::query(
        r#"
        UPDATE users
        SET
            username = COALESCE($2, username),
            password_hash = COALESCE($3, password_hash),
            phone = COALESCE($4, phone),
            status = COALESCE($5, status),
            updated_at = now()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.username.map(|value| value.trim().to_owned()))
    .bind(password_hash)
    .bind(payload.phone)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_user(&state.db, &id).await?))
}

async fn replace_user_roles(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<RoleIdsRequest>,
) -> Result<ApiJson<User>, ApiError> {
    replace_user_roles_inner(&state.db, &id, &payload.role_ids).await?;
    Ok(data(fetch_user(&state.db, &id).await?))
}

async fn list_roles(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<Role>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM roles")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, name, code, description
        FROM roles
        ORDER BY code
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(role_from_row(&state.db, row).await?);
    }

    Ok(page(items, pagination, total))
}

async fn create_role(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<RoleUpsert>,
) -> Result<ApiJson<Role>, ApiError> {
    validate_non_empty(&payload.name, "name")?;
    validate_non_empty(&payload.code, "code")?;
    let row = sqlx::query(
        r#"
        INSERT INTO roles (name, code, description)
        VALUES ($1, $2, $3)
        RETURNING id::text AS id, name, code, description
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.code.trim())
    .bind(payload.description)
    .fetch_one(&state.db)
    .await?;

    let role = role_from_row(&state.db, row).await?;
    if let Some(permission_ids) = payload.permission_ids {
        replace_role_permissions_inner(&state.db, &role.id, &permission_ids).await?;
        return Ok(data(fetch_role(&state.db, &role.id).await?));
    }

    Ok(data(role))
}

async fn get_role(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<Role>, ApiError> {
    Ok(data(fetch_role(&state.db, &id).await?))
}

async fn update_role(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<RoleUpsert>,
) -> Result<ApiJson<Role>, ApiError> {
    validate_non_empty(&payload.name, "name")?;
    validate_non_empty(&payload.code, "code")?;
    sqlx::query(
        r#"
        UPDATE roles
        SET name = $2, code = $3, description = $4
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.code.trim())
    .bind(payload.description)
    .execute(&state.db)
    .await?;

    if let Some(permission_ids) = payload.permission_ids {
        replace_role_permissions_inner(&state.db, &id, &permission_ids).await?;
    }

    Ok(data(fetch_role(&state.db, &id).await?))
}

async fn replace_role_permissions(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<PermissionIdsRequest>,
) -> Result<ApiJson<Role>, ApiError> {
    replace_role_permissions_inner(&state.db, &id, &payload.permission_ids).await?;
    Ok(data(fetch_role(&state.db, &id).await?))
}

async fn list_permissions(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<Permission>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM permissions")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, code, description
        FROM permissions
        ORDER BY code
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(permission_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(page(items, pagination, total))
}

async fn create_permission(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<PermissionUpsert>,
) -> Result<ApiJson<Permission>, ApiError> {
    validate_non_empty(&payload.code, "code")?;
    let row = sqlx::query(
        r#"
        INSERT INTO permissions (code, description)
        VALUES ($1, $2)
        RETURNING id::text AS id, code, description
        "#,
    )
    .bind(payload.code.trim())
    .bind(payload.description)
    .fetch_one(&state.db)
    .await?;

    Ok(data(permission_from_row(row)?))
}

async fn get_permission(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<Permission>, ApiError> {
    Ok(data(fetch_permission(&state.db, &id).await?))
}

async fn update_permission(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<PermissionUpsert>,
) -> Result<ApiJson<Permission>, ApiError> {
    validate_non_empty(&payload.code, "code")?;
    sqlx::query(
        r#"
        UPDATE permissions
        SET code = $2, description = $3
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.code.trim())
    .bind(payload.description)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_permission(&state.db, &id).await?))
}

async fn reserve_import_job(
    _auth: AuthUser,
    Json(_payload): Json<ImportJobCreate>,
) -> Result<ApiJson<()>, ApiError> {
    Err(ApiError::not_implemented(
        "basic data import is reserved for a later phase",
    ))
}

async fn fetch_user(db: &PgPool, id: &str) -> Result<User, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            username,
            phone,
            status,
            last_login_at::text AS last_login_at,
            created_at::text AS created_at,
            updated_at::text AS updated_at
        FROM users
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;

    user_from_row(db, row).await
}

async fn user_from_row(db: &PgPool, row: sqlx::postgres::PgRow) -> Result<User, ApiError> {
    let id = text(&row, "id")?;
    let roles = role_refs_for_user(db, &id).await?;
    let role_ids = roles.iter().map(|role| role.id.clone()).collect();

    Ok(User {
        id,
        username: row.try_get("username")?,
        phone: opt_text(&row, "phone")?,
        status: enum_value(&row, "status")?,
        last_login_at: opt_text(&row, "last_login_at")?,
        created_at: opt_text(&row, "created_at")?,
        updated_at: opt_text(&row, "updated_at")?,
        role_ids,
        roles,
    })
}

async fn fetch_role(db: &PgPool, id: &str) -> Result<Role, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, name, code, description
        FROM roles
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;

    role_from_row(db, row).await
}

async fn role_from_row(db: &PgPool, row: sqlx::postgres::PgRow) -> Result<Role, ApiError> {
    let id = text(&row, "id")?;
    let permissions = permissions_for_role(db, &id).await?;
    let permission_ids = permissions
        .iter()
        .map(|permission| permission.id.clone())
        .collect();

    Ok(Role {
        id,
        name: row.try_get("name")?,
        code: row.try_get("code")?,
        description: opt_text(&row, "description")?,
        permission_ids,
        permissions,
    })
}

fn permission_from_row(row: sqlx::postgres::PgRow) -> Result<Permission, sqlx::Error> {
    Ok(Permission {
        id: text(&row, "id")?,
        code: row.try_get("code")?,
        description: opt_text(&row, "description")?,
    })
}

async fn fetch_permission(db: &PgPool, id: &str) -> Result<Permission, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, code, description
        FROM permissions
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;

    Ok(permission_from_row(row)?)
}

async fn role_refs_for_user(db: &PgPool, user_id: &str) -> Result<Vec<Role>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT r.id::text AS id, r.name, r.code, r.description
        FROM roles r
        INNER JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = $1::uuid
        ORDER BY r.code
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let mut roles = Vec::with_capacity(rows.len());
    for row in rows {
        roles.push(role_from_row(db, row).await?);
    }

    Ok(roles)
}

async fn permissions_for_role(db: &PgPool, role_id: &str) -> Result<Vec<Permission>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT p.id::text AS id, p.code, p.description
        FROM permissions p
        INNER JOIN role_permissions rp ON rp.permission_id = p.id
        WHERE rp.role_id = $1::uuid
        ORDER BY p.code
        "#,
    )
    .bind(role_id)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(permission_from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)
}

async fn replace_user_roles_inner(
    db: &PgPool,
    user_id: &str,
    role_ids: &[Id],
) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1::uuid")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    for role_id in role_ids {
        sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES ($1::uuid, $2::uuid)
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

async fn replace_role_permissions_inner(
    db: &PgPool,
    role_id: &str,
    permission_ids: &[Id],
) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1::uuid")
        .bind(role_id)
        .execute(&mut *tx)
        .await?;

    for permission_id in permission_ids {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            VALUES ($1::uuid, $2::uuid)
            "#,
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

fn enum_string<T>(value: T) -> Result<String, ApiError>
where
    T: serde::Serialize,
{
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| ApiError::internal("failed to serialize enum"))
}

fn optional_enum_string<T>(value: Option<T>) -> Result<Option<String>, ApiError>
where
    T: serde::Serialize,
{
    value.map(enum_string).transpose()
}
