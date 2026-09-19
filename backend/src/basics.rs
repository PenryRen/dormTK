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
    http::StatusCode,
    routing::{delete, get, post},
};
use dormtk_api_types::basics::{
    Accommodation, AccommodationCreate, Building, BuildingUpsert, CheckoutRequest, ClassGroup,
    ClassGroupUpsert, Room, RoomUpsert, Student, StudentUpsert, TeacherClassAssignment,
    TeacherClassAssignmentCreate,
};
use dormtk_core::{BuildingGenderPolicy, DormRoomStatus, Gender, Id, StudentStatus, UserStatus};
use sqlx::{PgPool, Row};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admin/students",
            get(list_students).post(create_student),
        )
        .route(
            "/api/admin/students/{id}",
            get(get_student).patch(update_student),
        )
        .route("/api/admin/classes", get(list_classes).post(create_class))
        .route(
            "/api/admin/classes/{id}",
            get(get_class).patch(update_class),
        )
        .route(
            "/api/admin/classes/{id}/teachers",
            post(add_teacher_assignment),
        )
        .route(
            "/api/admin/classes/{id}/teachers/{teacher_id}",
            delete(remove_teacher_assignment),
        )
        .route(
            "/api/admin/buildings",
            get(list_buildings).post(create_building),
        )
        .route(
            "/api/admin/buildings/{id}",
            get(get_building).patch(update_building),
        )
        .route("/api/admin/rooms", get(list_rooms).post(create_room))
        .route("/api/admin/rooms/{id}", get(get_room).patch(update_room))
        .route(
            "/api/admin/accommodations",
            get(list_accommodations).post(create_accommodation),
        )
        .route(
            "/api/admin/accommodations/{id}/checkout",
            post(checkout_accommodation),
        )
}

async fn list_students(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<Student>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM students")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT
            id::text AS id,
            user_id::text AS user_id,
            student_no,
            name,
            gender,
            phone,
            class_id::text AS class_id,
            college,
            major,
            class_name,
            status,
            created_at::text AS created_at,
            updated_at::text AS updated_at
        FROM students
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(student_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_student(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<StudentUpsert>,
) -> Result<ApiJson<Student>, ApiError> {
    validate_student(&state.db, &payload).await?;
    let status = payload.status.unwrap_or(StudentStatus::Active);
    let row = sqlx::query(
        r#"
        INSERT INTO students
            (student_no, name, gender, phone, class_id, college, major, class_name, status)
        SELECT
            $1, $2, $3, $4, $5::uuid, $6, $7, cg.name, $8
        FROM class_groups cg
        WHERE cg.id = $5::uuid
        RETURNING
            id::text AS id,
            user_id::text AS user_id,
            student_no,
            name,
            gender,
            phone,
            class_id::text AS class_id,
            college,
            major,
            class_name,
            status,
            created_at::text AS created_at,
            updated_at::text AS updated_at
        "#,
    )
    .bind(payload.student_no.trim())
    .bind(payload.name.trim())
    .bind(enum_string(payload.gender)?)
    .bind(payload.phone)
    .bind(payload.class_id)
    .bind(payload.college)
    .bind(payload.major)
    .bind(enum_string(status)?)
    .fetch_one(&state.db)
    .await?;

    Ok(data(student_from_row(row)?))
}

async fn get_student(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<Student>, ApiError> {
    Ok(data(fetch_student(&state.db, &id).await?))
}

async fn update_student(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<StudentUpsert>,
) -> Result<ApiJson<Student>, ApiError> {
    validate_student(&state.db, &payload).await?;
    sqlx::query(
        r#"
        UPDATE students
        SET
            student_no = $2,
            name = $3,
            gender = $4,
            phone = $5,
            class_id = $6::uuid,
            college = $7,
            major = $8,
            class_name = (SELECT name FROM class_groups WHERE id = $6::uuid),
            status = COALESCE($9, status),
            updated_at = now()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.student_no.trim())
    .bind(payload.name.trim())
    .bind(enum_string(payload.gender)?)
    .bind(payload.phone)
    .bind(payload.class_id)
    .bind(payload.college)
    .bind(payload.major)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_student(&state.db, &id).await?))
}

async fn list_classes(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<ClassGroup>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM class_groups")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, name, code, college, major, grade, gender, status
        FROM class_groups
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
        .map(class_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_class(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<ClassGroupUpsert>,
) -> Result<ApiJson<ClassGroup>, ApiError> {
    validate_non_empty(&payload.name, "name")?;
    validate_non_empty(&payload.code, "code")?;
    let row = sqlx::query(
        r#"
        INSERT INTO class_groups (name, code, college, major, grade, gender, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id::text AS id, name, code, college, major, grade, gender, status
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.code.trim())
    .bind(payload.college)
    .bind(payload.major)
    .bind(payload.grade)
    .bind(enum_string(payload.gender)?)
    .bind(enum_string(payload.status.unwrap_or(UserStatus::Active))?)
    .fetch_one(&state.db)
    .await?;

    Ok(data(class_from_row(row)?))
}

async fn get_class(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<ClassGroup>, ApiError> {
    Ok(data(fetch_class(&state.db, &id).await?))
}

async fn update_class(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<ClassGroupUpsert>,
) -> Result<ApiJson<ClassGroup>, ApiError> {
    validate_non_empty(&payload.name, "name")?;
    validate_non_empty(&payload.code, "code")?;
    sqlx::query(
        r#"
        UPDATE class_groups
        SET name = $2, code = $3, college = $4, major = $5, grade = $6, gender = $7, status = COALESCE($8, status)
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.code.trim())
    .bind(payload.college)
    .bind(payload.major)
    .bind(payload.grade)
    .bind(enum_string(payload.gender)?)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_class(&state.db, &id).await?))
}

async fn add_teacher_assignment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(class_id): Path<Id>,
    Json(payload): Json<TeacherClassAssignmentCreate>,
) -> Result<ApiJson<TeacherClassAssignment>, ApiError> {
    let row = sqlx::query(
        r#"
        INSERT INTO teacher_class_assignments (teacher_id, class_id, assignment_type, assigned_by)
        VALUES ($1::uuid, $2::uuid, $3, $4::uuid)
        RETURNING id::text AS id, teacher_id::text AS teacher_id, class_id::text AS class_id,
            assignment_type, assigned_by::text AS assigned_by, status
        "#,
    )
    .bind(payload.teacher_id)
    .bind(class_id)
    .bind(enum_string(payload.assignment_type)?)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(data(teacher_assignment_from_row(row)?))
}

async fn remove_teacher_assignment(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path((class_id, teacher_id)): Path<(Id, Id)>,
) -> Result<StatusCode, ApiError> {
    sqlx::query(
        "DELETE FROM teacher_class_assignments WHERE class_id = $1::uuid AND teacher_id = $2::uuid",
    )
    .bind(class_id)
    .bind(teacher_id)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_buildings(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<Building>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM buildings")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, name, code, campus, gender_policy, status
        FROM buildings
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
        .map(building_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_building(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<BuildingUpsert>,
) -> Result<ApiJson<Building>, ApiError> {
    validate_non_empty(&payload.name, "name")?;
    validate_non_empty(&payload.code, "code")?;
    let row = sqlx::query(
        r#"
        INSERT INTO buildings (name, code, campus, gender_policy, status)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id::text AS id, name, code, campus, gender_policy, status
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.code.trim())
    .bind(payload.campus)
    .bind(enum_string(payload.gender_policy)?)
    .bind(enum_string(
        payload.status.unwrap_or(DormRoomStatus::Active),
    )?)
    .fetch_one(&state.db)
    .await?;

    Ok(data(building_from_row(row)?))
}

async fn get_building(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<Building>, ApiError> {
    Ok(data(fetch_building(&state.db, &id).await?))
}

async fn update_building(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<BuildingUpsert>,
) -> Result<ApiJson<Building>, ApiError> {
    validate_non_empty(&payload.name, "name")?;
    validate_non_empty(&payload.code, "code")?;
    sqlx::query(
        r#"
        UPDATE buildings
        SET name = $2, code = $3, campus = $4, gender_policy = $5, status = COALESCE($6, status)
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.code.trim())
    .bind(payload.campus)
    .bind(enum_string(payload.gender_policy)?)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_building(&state.db, &id).await?))
}

async fn list_rooms(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<Room>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM rooms")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, building_id::text AS building_id, class_id::text AS class_id,
            gender, room_no, floor, capacity, status
        FROM rooms
        ORDER BY room_no
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(room_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_room(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<RoomUpsert>,
) -> Result<ApiJson<Room>, ApiError> {
    validate_room(&state.db, &payload).await?;
    let row = sqlx::query(
        r#"
        INSERT INTO rooms (building_id, class_id, gender, room_no, floor, capacity, status)
        VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7)
        RETURNING id::text AS id, building_id::text AS building_id, class_id::text AS class_id,
            gender, room_no, floor, capacity, status
        "#,
    )
    .bind(payload.building_id)
    .bind(payload.class_id)
    .bind(enum_string(payload.gender)?)
    .bind(payload.room_no.trim())
    .bind(payload.floor)
    .bind(payload.capacity as i32)
    .bind(enum_string(
        payload.status.unwrap_or(DormRoomStatus::Active),
    )?)
    .fetch_one(&state.db)
    .await?;

    Ok(data(room_from_row(row)?))
}

async fn get_room(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
) -> Result<ApiJson<Room>, ApiError> {
    Ok(data(fetch_room(&state.db, &id).await?))
}

async fn update_room(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<RoomUpsert>,
) -> Result<ApiJson<Room>, ApiError> {
    validate_room(&state.db, &payload).await?;
    sqlx::query(
        r#"
        UPDATE rooms
        SET building_id = $2::uuid, class_id = $3::uuid, gender = $4, room_no = $5,
            floor = $6, capacity = $7, status = COALESCE($8, status)
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(payload.building_id)
    .bind(payload.class_id)
    .bind(enum_string(payload.gender)?)
    .bind(payload.room_no.trim())
    .bind(payload.floor)
    .bind(payload.capacity as i32)
    .bind(optional_enum_string(payload.status)?)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_room(&state.db, &id).await?))
}

async fn list_accommodations(
    _auth: AuthUser,
    State(state): State<AppState>,
    query: axum::extract::Query<Pagination>,
) -> Result<PageJson<Accommodation>, ApiError> {
    let pagination = pagination(query);
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM accommodations")
        .fetch_one(&state.db)
        .await?;
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, student_id::text AS student_id, room_id::text AS room_id,
            check_in_at::text AS check_in_at, check_out_at::text AS check_out_at, status
        FROM accommodations
        ORDER BY check_in_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(accommodation_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, pagination, total))
}

async fn create_accommodation(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<AccommodationCreate>,
) -> Result<ApiJson<Accommodation>, ApiError> {
    validate_accommodation(&state.db, &payload.student_id, &payload.room_id).await?;
    let row = sqlx::query(
        r#"
        INSERT INTO accommodations (student_id, room_id, check_in_at)
        VALUES ($1::uuid, $2::uuid, $3::timestamptz)
        RETURNING id::text AS id, student_id::text AS student_id, room_id::text AS room_id,
            check_in_at::text AS check_in_at, check_out_at::text AS check_out_at, status
        "#,
    )
    .bind(payload.student_id)
    .bind(payload.room_id)
    .bind(payload.check_in_at)
    .fetch_one(&state.db)
    .await?;

    Ok(data(accommodation_from_row(row)?))
}

async fn checkout_accommodation(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Id>,
    Json(payload): Json<CheckoutRequest>,
) -> Result<ApiJson<Accommodation>, ApiError> {
    sqlx::query(
        r#"
        UPDATE accommodations
        SET check_out_at = COALESCE($2::timestamptz, now()), status = 'checked_out'
        WHERE id = $1::uuid AND status = 'active'
        "#,
    )
    .bind(&id)
    .bind(payload.check_out_at)
    .execute(&state.db)
    .await?;

    Ok(data(fetch_accommodation(&state.db, &id).await?))
}

fn student_from_row(row: sqlx::postgres::PgRow) -> Result<Student, sqlx::Error> {
    Ok(Student {
        id: text(&row, "id")?,
        user_id: opt_text(&row, "user_id")?,
        student_no: row.try_get("student_no")?,
        name: row.try_get("name")?,
        gender: enum_value(&row, "gender")?,
        phone: opt_text(&row, "phone")?,
        class_id: text(&row, "class_id")?,
        college: opt_text(&row, "college")?,
        major: opt_text(&row, "major")?,
        class_name: opt_text(&row, "class_name")?,
        status: enum_value(&row, "status")?,
        created_at: opt_text(&row, "created_at")?,
        updated_at: opt_text(&row, "updated_at")?,
    })
}

fn class_from_row(row: sqlx::postgres::PgRow) -> Result<ClassGroup, sqlx::Error> {
    Ok(ClassGroup {
        id: text(&row, "id")?,
        name: row.try_get("name")?,
        code: row.try_get("code")?,
        college: opt_text(&row, "college")?,
        major: opt_text(&row, "major")?,
        grade: opt_text(&row, "grade")?,
        gender: enum_value(&row, "gender")?,
        status: enum_value(&row, "status")?,
    })
}

fn teacher_assignment_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<TeacherClassAssignment, sqlx::Error> {
    Ok(TeacherClassAssignment {
        id: text(&row, "id")?,
        teacher_id: text(&row, "teacher_id")?,
        class_id: text(&row, "class_id")?,
        assignment_type: enum_value(&row, "assignment_type")?,
        assigned_by: opt_text(&row, "assigned_by")?,
        status: enum_value(&row, "status")?,
    })
}

fn building_from_row(row: sqlx::postgres::PgRow) -> Result<Building, sqlx::Error> {
    Ok(Building {
        id: text(&row, "id")?,
        name: row.try_get("name")?,
        code: row.try_get("code")?,
        campus: opt_text(&row, "campus")?,
        gender_policy: enum_value(&row, "gender_policy")?,
        status: enum_value(&row, "status")?,
    })
}

fn room_from_row(row: sqlx::postgres::PgRow) -> Result<Room, sqlx::Error> {
    let capacity: i32 = row.try_get("capacity")?;
    Ok(Room {
        id: text(&row, "id")?,
        building_id: text(&row, "building_id")?,
        class_id: text(&row, "class_id")?,
        gender: enum_value(&row, "gender")?,
        room_no: row.try_get("room_no")?,
        floor: row.try_get("floor")?,
        capacity: capacity as u32,
        status: enum_value(&row, "status")?,
    })
}

fn accommodation_from_row(row: sqlx::postgres::PgRow) -> Result<Accommodation, sqlx::Error> {
    Ok(Accommodation {
        id: text(&row, "id")?,
        student_id: text(&row, "student_id")?,
        room_id: text(&row, "room_id")?,
        check_in_at: text(&row, "check_in_at")?,
        check_out_at: opt_text(&row, "check_out_at")?,
        status: enum_value(&row, "status")?,
    })
}

async fn fetch_student(db: &PgPool, id: &str) -> Result<Student, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, user_id::text AS user_id, student_no, name, gender, phone,
            class_id::text AS class_id, college, major, class_name, status,
            created_at::text AS created_at, updated_at::text AS updated_at
        FROM students
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(student_from_row(row)?)
}

async fn fetch_class(db: &PgPool, id: &str) -> Result<ClassGroup, ApiError> {
    let row = sqlx::query(
        "SELECT id::text AS id, name, code, college, major, grade, gender, status FROM class_groups WHERE id = $1::uuid",
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(class_from_row(row)?)
}

async fn fetch_building(db: &PgPool, id: &str) -> Result<Building, ApiError> {
    let row = sqlx::query(
        "SELECT id::text AS id, name, code, campus, gender_policy, status FROM buildings WHERE id = $1::uuid",
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(building_from_row(row)?)
}

async fn fetch_room(db: &PgPool, id: &str) -> Result<Room, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, building_id::text AS building_id, class_id::text AS class_id,
            gender, room_no, floor, capacity, status
        FROM rooms
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(room_from_row(row)?)
}

async fn fetch_accommodation(db: &PgPool, id: &str) -> Result<Accommodation, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, student_id::text AS student_id, room_id::text AS room_id,
            check_in_at::text AS check_in_at, check_out_at::text AS check_out_at, status
        FROM accommodations
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    Ok(accommodation_from_row(row)?)
}

async fn validate_student(db: &PgPool, payload: &StudentUpsert) -> Result<(), ApiError> {
    validate_non_empty(&payload.student_no, "student_no")?;
    validate_non_empty(&payload.name, "name")?;
    let class_gender: Gender = sqlx::query("SELECT gender FROM class_groups WHERE id = $1::uuid")
        .bind(&payload.class_id)
        .fetch_one(db)
        .await
        .and_then(|row| enum_value(&row, "gender"))?;

    if class_gender != payload.gender {
        return Err(ApiError::bad_request(
            "student gender must match class gender",
        ));
    }

    Ok(())
}

async fn validate_room(db: &PgPool, payload: &RoomUpsert) -> Result<(), ApiError> {
    validate_non_empty(&payload.room_no, "room_no")?;
    if payload.capacity == 0 {
        return Err(ApiError::bad_request("capacity must be greater than zero"));
    }

    let class_gender: Gender = sqlx::query("SELECT gender FROM class_groups WHERE id = $1::uuid")
        .bind(&payload.class_id)
        .fetch_one(db)
        .await
        .and_then(|row| enum_value(&row, "gender"))?;
    if class_gender != payload.gender {
        return Err(ApiError::bad_request("room gender must match class gender"));
    }

    let building_policy: BuildingGenderPolicy =
        sqlx::query("SELECT gender_policy FROM buildings WHERE id = $1::uuid")
            .bind(&payload.building_id)
            .fetch_one(db)
            .await
            .and_then(|row| enum_value(&row, "gender_policy"))?;
    if matches!(building_policy, BuildingGenderPolicy::MaleOnly) && payload.gender != Gender::Male {
        return Err(ApiError::bad_request(
            "male-only building cannot contain female room",
        ));
    }
    if matches!(building_policy, BuildingGenderPolicy::FemaleOnly)
        && payload.gender != Gender::Female
    {
        return Err(ApiError::bad_request(
            "female-only building cannot contain male room",
        ));
    }

    Ok(())
}

async fn validate_accommodation(
    db: &PgPool,
    student_id: &str,
    room_id: &str,
) -> Result<(), ApiError> {
    let row = sqlx::query(
        r#"
        SELECT
            s.class_id::text AS student_class_id,
            s.gender AS student_gender,
            r.class_id::text AS room_class_id,
            r.gender AS room_gender,
            r.capacity,
            (
                SELECT count(*)
                FROM accommodations a
                WHERE a.room_id = r.id AND a.status = 'active'
            ) AS active_count
        FROM students s
        CROSS JOIN rooms r
        WHERE s.id = $1::uuid AND r.id = $2::uuid
        "#,
    )
    .bind(student_id)
    .bind(room_id)
    .fetch_one(db)
    .await?;

    let student_class_id = text(&row, "student_class_id")?;
    let room_class_id = text(&row, "room_class_id")?;
    if student_class_id != room_class_id {
        return Err(ApiError::bad_request(
            "student cannot be assigned to a room outside their class",
        ));
    }

    let student_gender: Gender = enum_value(&row, "student_gender")?;
    let room_gender: Gender = enum_value(&row, "room_gender")?;
    if student_gender != room_gender {
        return Err(ApiError::bad_request(
            "student gender must match room gender",
        ));
    }

    let capacity: i32 = row.try_get("capacity")?;
    let active_count: i64 = row.try_get("active_count")?;
    if active_count >= capacity as i64 {
        return Err(ApiError::conflict("room capacity is full"));
    }

    Ok(())
}
