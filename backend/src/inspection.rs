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
    DormInspectionRecord, DormInspectionTask, DormInspectionTaskCreate, DormInspectionTaskUpdate,
    InspectionAbnormalRequest,
};
use dormtk_core::{Id, InspectionEvidenceSource, InspectionMethod, ScopeType, TaskStatus};
use sqlx::{PgPool, Row};

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
