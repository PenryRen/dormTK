use crate::{
    auth::AuthUser,
    db::{enum_string, enum_value, opt_text, optional_enum_string, text},
    error::ApiError,
    http::{ApiJson, PageJson, Pagination, data, page, pagination},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use dormtk_api_types::duties::{
    StudentDutyAssignment, StudentDutyAssignmentCreate, StudentDutyAssignmentUpdate,
    TaskAssignment, TaskAssignmentCreate, TaskAssignmentUpdate,
};
use dormtk_core::{
    AssigneeType, DutyAssignmentStatus, DutyScopeType, Id, StudentDutyType, TaskType,
};
use sqlx::{PgPool, Row};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admin/student-duty-assignments",
            get(list_student_duties).post(create_student_duty),
        )
        .route(
            "/api/admin/student-duty-assignments/{id}",
            get(get_student_duty).patch(update_student_duty),
        )
        .route(
            "/api/admin/task-assignments",
            get(list_task_assignments).post(create_task_assignment),
        )
        .route(
            "/api/admin/task-assignments/{id}",
            get(get_task_assignment).patch(update_task_assignment),
        )
}

async fn list_student_duties(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<StudentDutyAssignment>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM student_duty_assignments")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, student_id::text AS student_id, duty_type, scope_type,
            ARRAY(SELECT x::text FROM unnest(scope_ids) AS x) AS scope_ids,
            assigned_by::text AS assigned_by, status
        FROM student_duty_assignments
        ORDER BY id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(student_duty_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_student_duty(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<StudentDutyAssignmentCreate>,
) -> Result<ApiJson<StudentDutyAssignment>, ApiError> {
    validate_student_duty(&payload).await?;
    let row = sqlx::query(
        r#"
        INSERT INTO student_duty_assignments
            (student_id, duty_type, scope_type, scope_ids, assigned_by)
        VALUES ($1::uuid, $2, $3, ARRAY(SELECT unnest($4::text[])::uuid), $5::uuid)
        RETURNING id::text AS id, student_id::text AS student_id, duty_type, scope_type,
            ARRAY(SELECT x::text FROM unnest(scope_ids) AS x) AS scope_ids,
            assigned_by::text AS assigned_by, status
        "#,
    )
    .bind(payload.student_id)
    .bind(enum_string(payload.duty_type)?)
    .bind(enum_string(payload.scope_type)?)
    .bind(payload.scope_ids)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(data(student_duty_from_row(row)?))
}

async fn get_student_duty(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<StudentDutyAssignment>, ApiError> {
    Ok(data(fetch_student_duty(&state.db, &id).await?))
}

async fn update_student_duty(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<StudentDutyAssignmentUpdate>,
) -> Result<ApiJson<StudentDutyAssignment>, ApiError> {
    if let Some(scope_ids) = &payload.scope_ids
        && scope_ids.is_empty()
    {
        return Err(ApiError::bad_request("scope_ids must not be empty"));
    }

    sqlx::query(
        r#"
        UPDATE student_duty_assignments
        SET
            scope_ids = COALESCE(ARRAY(SELECT unnest($2::text[])::uuid), scope_ids),
            status = COALESCE($3, status)
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.scope_ids)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_student_duty(&state.db, &id).await?))
}

async fn list_task_assignments(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<TaskAssignment>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM task_assignments")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, task_type, task_id::text AS task_id, assignee_id::text AS assignee_id,
            assignee_type, duty_assignment_id::text AS duty_assignment_id,
            assigned_by::text AS assigned_by, assigned_at::text AS assigned_at, status
        FROM task_assignments
        ORDER BY assigned_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(task_assignment_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_task_assignment(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<TaskAssignmentCreate>,
) -> Result<ApiJson<TaskAssignment>, ApiError> {
    let duty_assignment_id = resolve_task_duty(&state.db, &payload).await?;
    let row = sqlx::query(
        r#"
        INSERT INTO task_assignments
            (task_type, task_id, assignee_id, assignee_type, duty_assignment_id, assigned_by)
        VALUES ($1, $2::uuid, $3::uuid, $4, $5::uuid, $6::uuid)
        RETURNING id::text AS id, task_type, task_id::text AS task_id, assignee_id::text AS assignee_id,
            assignee_type, duty_assignment_id::text AS duty_assignment_id,
            assigned_by::text AS assigned_by, assigned_at::text AS assigned_at, status
        "#,
    )
    .bind(enum_string(payload.task_type)?)
    .bind(payload.task_id)
    .bind(payload.assignee_id)
    .bind(enum_string(payload.assignee_type)?)
    .bind(duty_assignment_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(data(task_assignment_from_row(row)?))
}

async fn get_task_assignment(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<TaskAssignment>, ApiError> {
    Ok(data(fetch_task_assignment(&state.db, &id).await?))
}

async fn update_task_assignment(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<TaskAssignmentUpdate>,
) -> Result<ApiJson<TaskAssignment>, ApiError> {
    sqlx::query(
        r#"
        UPDATE task_assignments
        SET status = COALESCE($2, status)
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_task_assignment(&state.db, &id).await?))
}

fn student_duty_from_row(row: sqlx::postgres::PgRow) -> Result<StudentDutyAssignment, sqlx::Error> {
    Ok(StudentDutyAssignment {
        id: text(&row, "id")?,
        student_id: text(&row, "student_id")?,
        duty_type: enum_value(&row, "duty_type")?,
        scope_type: enum_value(&row, "scope_type")?,
        scope_ids: row.try_get("scope_ids")?,
        assigned_by: opt_text(&row, "assigned_by")?,
        status: enum_value(&row, "status")?,
    })
}

fn task_assignment_from_row(row: sqlx::postgres::PgRow) -> Result<TaskAssignment, sqlx::Error> {
    Ok(TaskAssignment {
        id: text(&row, "id")?,
        task_type: enum_value(&row, "task_type")?,
        task_id: text(&row, "task_id")?,
        assignee_id: text(&row, "assignee_id")?,
        assignee_type: enum_value(&row, "assignee_type")?,
        duty_assignment_id: opt_text(&row, "duty_assignment_id")?,
        assigned_by: opt_text(&row, "assigned_by")?,
        assigned_at: opt_text(&row, "assigned_at")?,
        status: enum_value(&row, "status")?,
    })
}

async fn fetch_student_duty(db: &PgPool, id: &str) -> Result<StudentDutyAssignment, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, student_id::text AS student_id, duty_type, scope_type,
            ARRAY(SELECT x::text FROM unnest(scope_ids) AS x) AS scope_ids,
            assigned_by::text AS assigned_by, status
        FROM student_duty_assignments
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(student_duty_from_row(row)?)
}

async fn fetch_task_assignment(db: &PgPool, id: &str) -> Result<TaskAssignment, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, task_type, task_id::text AS task_id, assignee_id::text AS assignee_id,
            assignee_type, duty_assignment_id::text AS duty_assignment_id,
            assigned_by::text AS assigned_by, assigned_at::text AS assigned_at, status
        FROM task_assignments
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(task_assignment_from_row(row)?)
}

async fn validate_student_duty(payload: &StudentDutyAssignmentCreate) -> Result<(), ApiError> {
    if payload.scope_ids.is_empty() {
        return Err(ApiError::bad_request("scope_ids must not be empty"));
    }

    if payload.duty_type == StudentDutyType::ClassMonitor
        && payload.scope_type != DutyScopeType::Class
    {
        return Err(ApiError::bad_request(
            "class monitor duty must use class scope",
        ));
    }

    Ok(())
}

async fn resolve_task_duty(
    db: &PgPool,
    payload: &TaskAssignmentCreate,
) -> Result<Option<Id>, ApiError> {
    if payload.assignee_type == AssigneeType::Teacher {
        return Ok(payload.duty_assignment_id.clone());
    }

    let required_duty = match payload.task_type {
        TaskType::Inspection => StudentDutyType::InspectionExecutor,
        TaskType::Hygiene => StudentDutyType::HygieneExecutor,
    };

    if let Some(duty_assignment_id) = &payload.duty_assignment_id {
        let row = sqlx::query(
            r#"
            SELECT duty_type, status
            FROM student_duty_assignments
            WHERE id = $1::uuid AND student_id = $2::uuid
            "#,
        )
        .bind(duty_assignment_id)
        .bind(&payload.assignee_id)
        .fetch_one(db)
        .await?;
        let duty_type: StudentDutyType = enum_value(&row, "duty_type")?;
        let status: DutyAssignmentStatus = enum_value(&row, "status")?;
        if duty_type != required_duty || status != DutyAssignmentStatus::Active {
            return Err(ApiError::bad_request(
                "student task assignee must have an active matching executor duty",
            ));
        }

        return Ok(Some(duty_assignment_id.clone()));
    }

    let row = sqlx::query(
        r#"
        SELECT id::text AS id
        FROM student_duty_assignments
        WHERE student_id = $1::uuid AND duty_type = $2 AND status = 'active'
        ORDER BY id
        LIMIT 1
        "#,
    )
    .bind(&payload.assignee_id)
    .bind(enum_string(required_duty)?)
    .fetch_optional(db)
    .await?;

    row.map(|row| text(&row, "id"))
        .transpose()
        .map_err(ApiError::from)?
        .map(Some)
        .ok_or_else(|| {
            ApiError::bad_request("student task assignee must have an active executor duty")
        })
}
