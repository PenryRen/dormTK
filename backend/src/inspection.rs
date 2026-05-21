use crate::{
    auth::AuthUser,
    db::{enum_string, enum_value, opt_text, optional_enum_string, text, validate_non_empty},
    error::ApiError,
    http::{ApiJson, PageJson, Pagination, data, page, pagination},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, patch, post},
};
use dormtk_api_types::inspection::{
    DormInspectionQrToken, DormInspectionRecord, DormInspectionTask, DormInspectionTaskCreate,
    DormInspectionTaskUpdate, InspectionAbnormalRequest,
};
use dormtk_core::{
    Id, InspectionEvidenceSource, InspectionMethod, InspectionResult, ScopeType, TaskStatus,
};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};

const QR_TOKEN_TTL_MINUTES: i64 = 5;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admin/inspection-tasks",
            get(list_tasks).post(create_task),
        )
        .route(
            "/api/admin/inspection-tasks/{id}",
            get(get_task).patch(update_task),
        )
        .route(
            "/api/admin/inspection-tasks/{id}/publish",
            post(publish_task),
        )
        .route(
            "/api/admin/inspection-tasks/{id}/complete",
            post(complete_task),
        )
        .route(
            "/api/admin/inspection-tasks/{task_id}/records",
            get(list_task_records),
        )
        .route(
            "/api/admin/inspection-records/{id}/abnormal",
            patch(mark_record_abnormal),
        )
        .route(
            "/api/student/inspection-records",
            get(list_my_inspection_records),
        )
        .route(
            "/api/student/inspection-records/{id}/qr-token",
            get(create_my_qr_token),
        )
        .route(
            "/api/student/inspection-qr-tokens/{id}/refresh",
            post(refresh_my_qr_token),
        )
}

async fn list_tasks(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<DormInspectionTask>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM dorm_inspection_tasks")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, title, scope_type,
            ARRAY(SELECT x::text FROM unnest(scope_ids) AS x) AS scope_ids,
            scheduled_at::text AS scheduled_at, method, status,
            created_by::text AS created_by, created_at::text AS created_at
        FROM dorm_inspection_tasks
        ORDER BY scheduled_at DESC, created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(task_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_task(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<DormInspectionTaskCreate>,
) -> Result<ApiJson<DormInspectionTask>, ApiError> {
    validate_task_create(&payload)?;
    let row = sqlx::query(
        r#"
        INSERT INTO dorm_inspection_tasks
            (title, scope_type, scope_ids, scheduled_at, method, created_by)
        VALUES ($1, $2, ARRAY(SELECT unnest($3::text[])::uuid), $4::timestamptz, $5, $6::uuid)
        RETURNING id::text AS id, title, scope_type,
            ARRAY(SELECT x::text FROM unnest(scope_ids) AS x) AS scope_ids,
            scheduled_at::text AS scheduled_at, method, status,
            created_by::text AS created_by, created_at::text AS created_at
        "#,
    )
    .bind(payload.title.trim())
    .bind(enum_string(payload.scope_type)?)
    .bind(payload.scope_ids)
    .bind(payload.scheduled_at)
    .bind(enum_string(payload.method)?)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(data(task_from_row(row)?))
}

async fn get_task(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<DormInspectionTask>, ApiError> {
    Ok(data(fetch_task(&state.db, &id).await?))
}

async fn update_task(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<DormInspectionTaskUpdate>,
) -> Result<ApiJson<DormInspectionTask>, ApiError> {
    if let Some(title) = &payload.title {
        validate_non_empty(title, "title")?;
    }

    sqlx::query(
        r#"
        UPDATE dorm_inspection_tasks
        SET
            title = COALESCE($2, title),
            scheduled_at = COALESCE($3::timestamptz, scheduled_at),
            status = COALESCE($4, status)
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.title.map(|title| title.trim().to_owned()))
    .bind(payload.scheduled_at)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_task(&state.db, &id).await?))
}

async fn publish_task(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<DormInspectionTask>, ApiError> {
    let task = fetch_task(&state.db, &id).await?;
    if task.status == TaskStatus::Completed || task.status == TaskStatus::Cancelled {
        return Err(ApiError::bad_request(
            "completed or cancelled task cannot be published",
        ));
    }
    if task.scope_type == ScopeType::Floor {
        return Err(ApiError::bad_request(
            "floor scope is not implemented in P3 because there is no floor entity",
        ));
    }

    generate_unknown_records(&state.db, &task).await?;
    sqlx::query(
        "UPDATE dorm_inspection_tasks SET status = 'published' WHERE id = $1::uuid AND status <> 'completed'",
    )
    .bind(&id)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_task(&state.db, &id).await?))
}

async fn complete_task(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<DormInspectionTask>, ApiError> {
    let unknown_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM dorm_inspection_records WHERE task_id = $1::uuid AND result = 'unknown'",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;
    if unknown_count > 0 {
        return Err(ApiError::bad_request(
            "inspection task still has unknown records",
        ));
    }

    sqlx::query("UPDATE dorm_inspection_tasks SET status = 'completed' WHERE id = $1::uuid")
        .bind(&id)
        .execute(&state.db)
        .await?;

    Ok(data(fetch_task(&state.db, &id).await?))
}

async fn list_task_records(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(task_id): Path<Id>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<DormInspectionRecord>, ApiError> {
    let pagination = pagination(query);
    let total: i64 =
        sqlx::query_scalar("SELECT count(*) FROM dorm_inspection_records WHERE task_id = $1::uuid")
            .bind(&task_id)
            .fetch_one(&state.db)
            .await?;
    let sql = record_select_sql(
        "WHERE task_id = $1::uuid ORDER BY room_id, student_id LIMIT $2 OFFSET $3",
    );
    let rows = sqlx::query(&sql)
        .bind(task_id)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&state.db)
        .await?;

    let items = rows
        .into_iter()
        .map(record_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn mark_record_abnormal(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<InspectionAbnormalRequest>,
) -> Result<ApiJson<DormInspectionRecord>, ApiError> {
    validate_non_empty(&payload.abnormal_reason, "abnormal_reason")?;
    sqlx::query(
        r#"
        UPDATE dorm_inspection_records
        SET result = 'abnormal',
            abnormal_reason = $2,
            evidence_source = 'manual_abnormal',
            evidence_ref_id = NULL,
            leave_request_id = NULL,
            checked_by = $3::uuid,
            checked_at = now()
        WHERE id = $1::uuid AND result = 'unknown'
        "#,
    )
    .bind(&id)
    .bind(payload.abnormal_reason.trim())
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_record(&state.db, &id).await?))
}

async fn list_my_inspection_records(
    auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<DormInspectionRecord>, ApiError> {
    let student_id = current_student_id(&state.db, &auth.user_id).await?;
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM dorm_inspection_records WHERE student_id = $1::uuid",
    )
    .bind(&student_id)
    .fetch_one(&state.db)
    .await?;
    let sql = record_select_sql(
        "WHERE student_id = $1::uuid ORDER BY checked_at DESC NULLS FIRST, id DESC LIMIT $2 OFFSET $3",
    );
    let rows = sqlx::query(&sql)
        .bind(student_id)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&state.db)
        .await?;
    let items = rows
        .into_iter()
        .map(record_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(page(items, pagination, total))
}

async fn create_my_qr_token(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(record_id): Path<Id>,
) -> Result<ApiJson<DormInspectionQrToken>, ApiError> {
    let student_id = current_student_id(&state.db, &auth.user_id).await?;
    let token = issue_qr_token_for_record(&state.db, &record_id, &student_id).await?;

    Ok(data(token))
}

async fn refresh_my_qr_token(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(token_id): Path<Id>,
) -> Result<ApiJson<DormInspectionQrToken>, ApiError> {
    let student_id = current_student_id(&state.db, &auth.user_id).await?;
    let record_id: Id = sqlx::query_scalar(
        r#"
        SELECT inspection_record_id::text
        FROM dorm_inspection_qr_tokens
        WHERE id = $1::uuid AND student_id = $2::uuid
        "#,
    )
    .bind(token_id)
    .bind(&student_id)
    .fetch_one(&state.db)
    .await?;

    let token = issue_qr_token_for_record(&state.db, &record_id, &student_id).await?;
    Ok(data(token))
}

fn validate_task_create(payload: &DormInspectionTaskCreate) -> Result<(), ApiError> {
    validate_non_empty(&payload.title, "title")?;
    if payload.scope_ids.is_empty() {
        return Err(ApiError::bad_request("scope_ids must not be empty"));
    }
    if payload.method != InspectionMethod::QrCode
        && payload.method != InspectionMethod::ManualAbnormal
    {
        return Err(ApiError::bad_request("unsupported inspection method"));
    }
    Ok(())
}

async fn generate_unknown_records(db: &PgPool, task: &DormInspectionTask) -> Result<(), ApiError> {
    match task.scope_type {
        ScopeType::Building => {
            insert_records_for_scope(
                db,
                &task.id,
                r#"
                SELECT a.student_id, a.room_id
                FROM accommodations a
                INNER JOIN rooms r ON r.id = a.room_id
                WHERE a.status = 'active' AND r.building_id = ANY(ARRAY(SELECT unnest($2::text[])::uuid))
                "#,
                &task.scope_ids,
            )
            .await
        }
        ScopeType::Room => {
            insert_records_for_scope(
                db,
                &task.id,
                r#"
                SELECT a.student_id, a.room_id
                FROM accommodations a
                WHERE a.status = 'active' AND a.room_id = ANY(ARRAY(SELECT unnest($2::text[])::uuid))
                "#,
                &task.scope_ids,
            )
            .await
        }
        ScopeType::Class => {
            insert_records_for_scope(
                db,
                &task.id,
                r#"
                SELECT a.student_id, a.room_id
                FROM accommodations a
                INNER JOIN students s ON s.id = a.student_id
                WHERE a.status = 'active' AND s.class_id = ANY(ARRAY(SELECT unnest($2::text[])::uuid))
                "#,
                &task.scope_ids,
            )
            .await
        }
        ScopeType::Student => {
            insert_records_for_scope(
                db,
                &task.id,
                r#"
                SELECT a.student_id, a.room_id
                FROM accommodations a
                WHERE a.status = 'active' AND a.student_id = ANY(ARRAY(SELECT unnest($2::text[])::uuid))
                "#,
                &task.scope_ids,
            )
            .await
        }
        ScopeType::Floor => Err(ApiError::bad_request("floor scope is not implemented in P3")),
    }
}

async fn insert_records_for_scope(
    db: &PgPool,
    task_id: &str,
    scoped_select: &str,
    scope_ids: &[Id],
) -> Result<(), ApiError> {
    let sql = format!(
        r#"
        INSERT INTO dorm_inspection_records (task_id, student_id, room_id)
        SELECT $1::uuid, scoped.student_id, scoped.room_id
        FROM ({scoped_select}) scoped
        ON CONFLICT (task_id, student_id) DO NOTHING
        "#
    );
    sqlx::query(&sql)
        .bind(task_id)
        .bind(scope_ids)
        .execute(db)
        .await?;

    Ok(())
}

async fn fetch_task(db: &PgPool, id: &str) -> Result<DormInspectionTask, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, title, scope_type,
            ARRAY(SELECT x::text FROM unnest(scope_ids) AS x) AS scope_ids,
            scheduled_at::text AS scheduled_at, method, status,
            created_by::text AS created_by, created_at::text AS created_at
        FROM dorm_inspection_tasks
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;

    Ok(task_from_row(row)?)
}

async fn fetch_record(db: &PgPool, id: &str) -> Result<DormInspectionRecord, ApiError> {
    let sql = record_select_sql("WHERE id = $1::uuid");
    let row = sqlx::query(&sql).bind(id).fetch_one(db).await?;
    Ok(record_from_row(row)?)
}

async fn issue_qr_token_for_record(
    db: &PgPool,
    record_id: &str,
    student_id: &str,
) -> Result<DormInspectionQrToken, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT
            r.id::text AS record_id,
            r.task_id::text AS task_id,
            r.student_id::text AS student_id,
            r.room_id::text AS room_id,
            r.result,
            t.status AS task_status,
            t.scheduled_at::text AS scheduled_at,
            a.room_id::text AS accommodation_room_id
        FROM dorm_inspection_records r
        INNER JOIN dorm_inspection_tasks t ON t.id = r.task_id
        LEFT JOIN accommodations a ON a.student_id = r.student_id AND a.status = 'active'
        WHERE r.id = $1::uuid AND r.student_id = $2::uuid
        "#,
    )
    .bind(record_id)
    .bind(student_id)
    .fetch_one(db)
    .await?;

    let result: InspectionResult = enum_value(&row, "result")?;
    if result != InspectionResult::Unknown {
        return Err(ApiError::bad_request(
            "only unknown inspection records can issue QR tokens",
        ));
    }
    let task_status: TaskStatus = enum_value(&row, "task_status")?;
    if task_status != TaskStatus::Published && task_status != TaskStatus::Processing {
        return Err(ApiError::bad_request(
            "inspection task is not available for QR token issuing",
        ));
    }
    let room_id = text(&row, "room_id")?;
    let accommodation_room_id = opt_text(&row, "accommodation_room_id")?
        .ok_or_else(|| ApiError::bad_request("student has no active accommodation"))?;
    if accommodation_room_id != room_id {
        return Err(ApiError::bad_request(
            "active accommodation room does not match inspection record",
        ));
    }

    if let Some(record) = mark_leave_if_approved(db, record_id).await? {
        return Err(ApiError::bad_request(format!(
            "inspection record is covered by approved leave: {}",
            record.leave_request_id.unwrap_or_default()
        )));
    }

    sqlx::query(
        "UPDATE dorm_inspection_qr_tokens SET status = 'revoked' WHERE inspection_record_id = $1::uuid AND status = 'active'",
    )
    .bind(record_id)
    .execute(db)
    .await?;

    let plain_token = generate_plain_token();
    let token_hash = hash_qr_token(&plain_token);
    let token_row = sqlx::query(
        r#"
        INSERT INTO dorm_inspection_qr_tokens
            (task_id, inspection_record_id, student_id, room_id, token_hash, expires_at)
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4::uuid, $5, now() + ($6::text || ' minutes')::interval)
        RETURNING id::text AS id, task_id::text AS task_id, inspection_record_id::text AS inspection_record_id,
            student_id::text AS student_id, room_id::text AS room_id,
            issued_at::text AS issued_at, expires_at::text AS expires_at, used_at::text AS used_at, status
        "#,
    )
    .bind(text(&row, "task_id")?)
    .bind(record_id)
    .bind(student_id)
    .bind(room_id)
    .bind(token_hash)
    .bind(QR_TOKEN_TTL_MINUTES.to_string())
    .fetch_one(db)
    .await?;

    let mut token = qr_token_from_row(token_row)?;
    token.token = plain_token;
    Ok(token)
}

async fn mark_leave_if_approved(
    db: &PgPool,
    record_id: &str,
) -> Result<Option<DormInspectionRecord>, ApiError> {
    let updated = sqlx::query(
        r#"
        WITH matched_leave AS (
            SELECT lr.id
            FROM dorm_inspection_records r
            INNER JOIN dorm_inspection_tasks t ON t.id = r.task_id
            INNER JOIN leave_requests lr ON lr.student_id = r.student_id
            WHERE r.id = $1::uuid
                AND r.result = 'unknown'
                AND lr.status = 'approved'
                AND lr.starts_at <= t.scheduled_at
                AND lr.ends_at >= t.scheduled_at
            ORDER BY lr.submitted_at DESC
            LIMIT 1
        )
        UPDATE dorm_inspection_records r
        SET result = 'leave',
            abnormal_reason = NULL,
            evidence_source = NULL,
            evidence_ref_id = NULL,
            leave_request_id = matched_leave.id,
            checked_by = NULL,
            checked_at = now()
        FROM matched_leave
        WHERE r.id = $1::uuid
        RETURNING r.id::text AS id
        "#,
    )
    .bind(record_id)
    .fetch_optional(db)
    .await?;

    if updated.is_some() {
        return Ok(Some(fetch_record(db, record_id).await?));
    }

    Ok(None)
}

async fn current_student_id(db: &PgPool, user_id: &str) -> Result<Id, ApiError> {
    sqlx::query_scalar("SELECT id::text FROM students WHERE user_id = $1::uuid")
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::not_found("current user is not linked to a student"))
}

fn task_from_row(row: sqlx::postgres::PgRow) -> Result<DormInspectionTask, sqlx::Error> {
    Ok(DormInspectionTask {
        id: text(&row, "id")?,
        title: row.try_get("title")?,
        scope_type: enum_value(&row, "scope_type")?,
        scope_ids: row.try_get("scope_ids")?,
        scheduled_at: text(&row, "scheduled_at")?,
        method: enum_value(&row, "method")?,
        status: enum_value(&row, "status")?,
        created_by: opt_text(&row, "created_by")?,
        created_at: opt_text(&row, "created_at")?,
    })
}

fn record_from_row(row: sqlx::postgres::PgRow) -> Result<DormInspectionRecord, sqlx::Error> {
    Ok(DormInspectionRecord {
        id: text(&row, "id")?,
        task_id: text(&row, "task_id")?,
        student_id: text(&row, "student_id")?,
        room_id: text(&row, "room_id")?,
        result: enum_value(&row, "result")?,
        abnormal_reason: opt_text(&row, "abnormal_reason")?,
        evidence_source: optional_inspection_evidence(&row)?,
        evidence_ref_id: opt_text(&row, "evidence_ref_id")?,
        leave_request_id: opt_text(&row, "leave_request_id")?,
        checked_by: opt_text(&row, "checked_by")?,
        checked_at: opt_text(&row, "checked_at")?,
    })
}

fn qr_token_from_row(row: sqlx::postgres::PgRow) -> Result<DormInspectionQrToken, sqlx::Error> {
    Ok(DormInspectionQrToken {
        id: text(&row, "id")?,
        task_id: text(&row, "task_id")?,
        inspection_record_id: text(&row, "inspection_record_id")?,
        student_id: text(&row, "student_id")?,
        room_id: text(&row, "room_id")?,
        token: String::new(),
        issued_at: opt_text(&row, "issued_at")?,
        expires_at: text(&row, "expires_at")?,
        used_at: opt_text(&row, "used_at")?,
        status: enum_value(&row, "status")?,
    })
}

fn optional_inspection_evidence(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<InspectionEvidenceSource>, sqlx::Error> {
    let value: Option<String> = row.try_get("evidence_source")?;
    value
        .map(|value| serde_json::from_value(serde_json::Value::String(value)))
        .transpose()
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

fn record_select_sql(tail: &str) -> String {
    format!(
        r#"
        SELECT id::text AS id, task_id::text AS task_id, student_id::text AS student_id,
            room_id::text AS room_id, result, abnormal_reason, evidence_source,
            evidence_ref_id::text AS evidence_ref_id, leave_request_id::text AS leave_request_id,
            checked_by::text AS checked_by, checked_at::text AS checked_at
        FROM dorm_inspection_records
        {tail}
        "#
    )
}

fn generate_plain_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex_encode(&bytes)
}

fn hash_qr_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex_encode(&digest)
}

#[cfg(test)]
fn verify_qr_token(token: &str, token_hash: &str) -> bool {
    hash_qr_token(token) == token_hash
}

#[cfg(test)]
fn token_is_expired(now_epoch_seconds: i64, expires_epoch_seconds: i64) -> bool {
    now_epoch_seconds >= expires_epoch_seconds
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_token_hash_verifies_original_token() {
        let token = "sample-token";
        let hash = hash_qr_token(token);

        assert!(verify_qr_token(token, &hash));
        assert!(!verify_qr_token("other-token", &hash));
    }

    #[test]
    fn generated_qr_token_is_hex_encoded_32_bytes() {
        let token = generate_plain_token();

        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn token_expiration_uses_inclusive_boundary() {
        assert!(!token_is_expired(99, 100));
        assert!(token_is_expired(100, 100));
        assert!(token_is_expired(101, 100));
    }
}
