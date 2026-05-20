CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    username text NOT NULL UNIQUE,
    password_hash text NOT NULL,
    phone text,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    last_login_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE roles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    code text NOT NULL UNIQUE,
    description text
);

CREATE TABLE permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code text NOT NULL UNIQUE,
    description text
);

CREATE TABLE user_roles (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE role_permissions (
    role_id uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id uuid NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE file_objects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_type text NOT NULL,
    owner_id uuid,
    file_url text NOT NULL,
    file_hash text NOT NULL,
    mime_type text NOT NULL,
    size_bytes bigint NOT NULL CHECK (size_bytes >= 0),
    storage_key text NOT NULL,
    uploaded_by uuid REFERENCES users(id) ON DELETE SET NULL,
    uploaded_at timestamptz NOT NULL DEFAULT now(),
    retention_until timestamptz,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'pending_delete', 'deleted', 'delete_failed')),
    deleted_at timestamptz,
    delete_error text
);

CREATE INDEX idx_file_objects_owner ON file_objects(owner_type, owner_id);
CREATE INDEX idx_file_objects_status_retention ON file_objects(status, retention_until);

CREATE TABLE class_groups (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    code text NOT NULL UNIQUE,
    college text,
    major text,
    grade text,
    gender text NOT NULL CHECK (gender IN ('male', 'female')),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled'))
);

CREATE TABLE students (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid UNIQUE REFERENCES users(id) ON DELETE SET NULL,
    student_no text NOT NULL UNIQUE,
    name text NOT NULL,
    gender text NOT NULL CHECK (gender IN ('male', 'female')),
    phone text,
    class_id uuid NOT NULL REFERENCES class_groups(id) ON DELETE RESTRICT,
    college text,
    major text,
    class_name text,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'graduated', 'left')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_students_class_id ON students(class_id);

CREATE TABLE teacher_class_assignments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    teacher_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    class_id uuid NOT NULL REFERENCES class_groups(id) ON DELETE CASCADE,
    assignment_type text NOT NULL CHECK (assignment_type IN ('counselor', 'teacher', 'temporary_manager')),
    assigned_by uuid REFERENCES users(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    UNIQUE (teacher_id, class_id, assignment_type)
);

CREATE TABLE buildings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    code text NOT NULL UNIQUE,
    campus text,
    gender_policy text NOT NULL CHECK (gender_policy IN ('male_only', 'female_only', 'mixed')),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled'))
);

CREATE TABLE rooms (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id uuid NOT NULL REFERENCES buildings(id) ON DELETE RESTRICT,
    class_id uuid NOT NULL REFERENCES class_groups(id) ON DELETE RESTRICT,
    gender text NOT NULL CHECK (gender IN ('male', 'female')),
    room_no text NOT NULL,
    floor integer,
    capacity integer NOT NULL CHECK (capacity > 0),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    UNIQUE (building_id, room_no)
);

CREATE INDEX idx_rooms_class_id ON rooms(class_id);

CREATE TABLE accommodations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    check_in_at timestamptz NOT NULL,
    check_out_at timestamptz,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'checked_out'))
);

CREATE UNIQUE INDEX uniq_active_accommodation_student ON accommodations(student_id) WHERE status = 'active';
CREATE INDEX idx_accommodations_room_id ON accommodations(room_id);

CREATE TABLE student_duty_assignments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    duty_type text NOT NULL CHECK (duty_type IN ('class_monitor', 'inspection_executor', 'hygiene_executor')),
    scope_type text NOT NULL CHECK (scope_type IN ('room', 'class', 'building', 'task')),
    scope_ids uuid[] NOT NULL,
    assigned_by uuid REFERENCES users(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled'))
);

CREATE INDEX idx_student_duty_assignments_student_id ON student_duty_assignments(student_id);

CREATE TABLE task_assignments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type text NOT NULL CHECK (task_type IN ('inspection', 'hygiene')),
    task_id uuid NOT NULL,
    assignee_id uuid NOT NULL,
    assignee_type text NOT NULL CHECK (assignee_type IN ('teacher', 'student')),
    duty_assignment_id uuid REFERENCES student_duty_assignments(id) ON DELETE SET NULL,
    assigned_by uuid REFERENCES users(id) ON DELETE SET NULL,
    assigned_at timestamptz NOT NULL DEFAULT now(),
    status text NOT NULL DEFAULT 'assigned' CHECK (status IN ('assigned', 'processing', 'completed', 'cancelled'))
);

CREATE INDEX idx_task_assignments_task ON task_assignments(task_type, task_id);
CREATE INDEX idx_task_assignments_assignee ON task_assignments(assignee_type, assignee_id);

CREATE TABLE dorm_inspection_tasks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    title text NOT NULL,
    scope_type text NOT NULL CHECK (scope_type IN ('building', 'floor', 'room', 'class', 'student')),
    scope_ids uuid[] NOT NULL,
    scheduled_at timestamptz NOT NULL,
    method text NOT NULL CHECK (method IN ('manual_abnormal', 'qr_code')),
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'processing', 'completed', 'cancelled')),
    created_by uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE leave_requests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    leave_type text NOT NULL,
    source text NOT NULL DEFAULT 'online' CHECK (source IN ('online', 'paper', 'appeal')),
    appeal_id uuid,
    reason text NOT NULL,
    starts_at timestamptz NOT NULL,
    ends_at timestamptz NOT NULL,
    status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'withdrawn', 'cancelled')),
    submitted_at timestamptz NOT NULL DEFAULT now(),
    reviewed_by uuid REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at timestamptz,
    review_comment text,
    CHECK (starts_at < ends_at)
);

CREATE INDEX idx_leave_requests_student_time ON leave_requests(student_id, starts_at, ends_at);

CREATE TABLE dorm_inspection_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id uuid NOT NULL REFERENCES dorm_inspection_tasks(id) ON DELETE CASCADE,
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    result text NOT NULL DEFAULT 'unknown' CHECK (result IN ('present', 'abnormal', 'leave', 'unknown')),
    abnormal_reason text,
    evidence_source text CHECK (evidence_source IN ('check_in', 'qr_code', 'manual_abnormal', 'inspection_room_completed', 'other_verified')),
    evidence_ref_id uuid,
    leave_request_id uuid REFERENCES leave_requests(id) ON DELETE SET NULL,
    checked_by uuid,
    checked_at timestamptz,
    UNIQUE (task_id, student_id)
);

CREATE INDEX idx_dorm_inspection_records_student_id ON dorm_inspection_records(student_id);
CREATE INDEX idx_dorm_inspection_records_room_id ON dorm_inspection_records(room_id);
CREATE INDEX idx_dorm_inspection_records_result ON dorm_inspection_records(result);

CREATE TABLE dorm_inspection_qr_tokens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id uuid NOT NULL REFERENCES dorm_inspection_tasks(id) ON DELETE CASCADE,
    inspection_record_id uuid NOT NULL REFERENCES dorm_inspection_records(id) ON DELETE CASCADE,
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    token_hash text NOT NULL UNIQUE,
    issued_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    used_at timestamptz,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'used', 'expired', 'revoked'))
);

CREATE INDEX idx_dorm_inspection_qr_tokens_task_student_status ON dorm_inspection_qr_tokens(task_id, student_id, status);
CREATE UNIQUE INDEX uniq_active_qr_token_per_record ON dorm_inspection_qr_tokens(inspection_record_id) WHERE status = 'active';

CREATE TABLE dorm_inspection_qr_scans (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    qr_token_id uuid NOT NULL REFERENCES dorm_inspection_qr_tokens(id) ON DELETE RESTRICT,
    task_assignment_id uuid NOT NULL REFERENCES task_assignments(id) ON DELETE RESTRICT,
    task_id uuid NOT NULL REFERENCES dorm_inspection_tasks(id) ON DELETE CASCADE,
    inspection_record_id uuid NOT NULL REFERENCES dorm_inspection_records(id) ON DELETE CASCADE,
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    executor_type text NOT NULL CHECK (executor_type IN ('teacher', 'student')),
    executor_id uuid NOT NULL,
    scanned_at timestamptz NOT NULL DEFAULT now(),
    scan_result text NOT NULL CHECK (scan_result IN ('accepted', 'rejected')),
    reject_reason text
);

CREATE INDEX idx_dorm_inspection_qr_scans_record_id ON dorm_inspection_qr_scans(inspection_record_id);

CREATE TABLE dorm_inspection_room_completions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id uuid NOT NULL REFERENCES dorm_inspection_tasks(id) ON DELETE CASCADE,
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    executor_id uuid NOT NULL,
    completed_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (task_id, room_id, executor_id)
);

CREATE TABLE inspection_abnormal_appeals (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    inspection_record_id uuid NOT NULL REFERENCES dorm_inspection_records(id) ON DELETE CASCADE,
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    appeal_type text NOT NULL CHECK (appeal_type IN ('leave', 'present')),
    reason text NOT NULL,
    status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    submitted_at timestamptz NOT NULL DEFAULT now(),
    reviewed_by uuid REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at timestamptz,
    review_comment text
);

CREATE UNIQUE INDEX uniq_pending_appeal_per_record ON inspection_abnormal_appeals(inspection_record_id) WHERE status = 'pending';

ALTER TABLE leave_requests
    ADD CONSTRAINT fk_leave_requests_appeal
    FOREIGN KEY (appeal_id)
    REFERENCES inspection_abnormal_appeals(id)
    ON DELETE SET NULL;

CREATE TABLE inspection_abnormal_appeal_files (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    appeal_id uuid NOT NULL REFERENCES inspection_abnormal_appeals(id) ON DELETE CASCADE,
    file_object_id uuid NOT NULL REFERENCES file_objects(id) ON DELETE RESTRICT
);

CREATE TABLE dorm_check_in_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    title text NOT NULL,
    scope_type text NOT NULL CHECK (scope_type IN ('building', 'class', 'room', 'student')),
    scope_ids uuid[] NOT NULL,
    starts_at_time time NOT NULL,
    ends_at_time time NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    created_by uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE dorm_wifi_fingerprints (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    ssid text NOT NULL,
    bssid text NOT NULL,
    min_signal_strength integer NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_dorm_wifi_fingerprints_room_id ON dorm_wifi_fingerprints(room_id);

CREATE TABLE dorm_check_in_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id uuid REFERENCES dorm_check_in_rules(id) ON DELETE SET NULL,
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    check_in_date date NOT NULL,
    status text NOT NULL CHECK (status IN ('submitted', 'late', 'missed', 'rejected')),
    submitted_at timestamptz,
    matched_wifi_count integer,
    scanned_wifi_count integer,
    wifi_evidence jsonb,
    reject_reason text,
    UNIQUE (student_id, check_in_date)
);

CREATE TABLE hygiene_score_templates (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    version integer NOT NULL DEFAULT 1,
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'disabled')),
    created_by uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE hygiene_score_criteria (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id uuid NOT NULL REFERENCES hygiene_score_templates(id) ON DELETE CASCADE,
    item_name text NOT NULL,
    max_score numeric NOT NULL CHECK (max_score > 0),
    sort_order integer NOT NULL
);

CREATE TABLE hygiene_tasks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    title text NOT NULL,
    scope_type text NOT NULL CHECK (scope_type IN ('building', 'floor', 'class', 'room')),
    scope_ids uuid[] NOT NULL,
    scheduled_at timestamptz NOT NULL,
    score_template_id uuid NOT NULL REFERENCES hygiene_score_templates(id) ON DELETE RESTRICT,
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'processing', 'completed', 'cancelled')),
    created_by uuid REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE hygiene_records (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id uuid NOT NULL REFERENCES hygiene_tasks(id) ON DELETE CASCADE,
    room_id uuid NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    total_score numeric NOT NULL CHECK (total_score >= 0),
    level text CHECK (level IN ('excellent', 'good', 'passed', 'failed')),
    comment text,
    checked_by uuid NOT NULL,
    checked_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (task_id, room_id)
);

CREATE TABLE hygiene_score_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    hygiene_record_id uuid NOT NULL REFERENCES hygiene_records(id) ON DELETE CASCADE,
    criterion_id uuid NOT NULL REFERENCES hygiene_score_criteria(id) ON DELETE RESTRICT,
    item_name text NOT NULL,
    score numeric NOT NULL CHECK (score >= 0),
    max_score numeric NOT NULL CHECK (max_score > 0),
    comment text
);

CREATE TABLE hygiene_photos (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    hygiene_record_id uuid NOT NULL REFERENCES hygiene_records(id) ON DELETE CASCADE,
    file_object_id uuid NOT NULL REFERENCES file_objects(id) ON DELETE RESTRICT,
    photo_type text
);

CREATE TABLE paper_leave_evidences (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id uuid NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    student_id uuid NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    file_object_id uuid NOT NULL REFERENCES file_objects(id) ON DELETE RESTRICT,
    uploaded_by uuid REFERENCES users(id) ON DELETE SET NULL,
    uploaded_at timestamptz NOT NULL DEFAULT now(),
    verified_by uuid REFERENCES users(id) ON DELETE SET NULL,
    verified_at timestamptz,
    status text NOT NULL DEFAULT 'uploaded' CHECK (status IN ('uploaded', 'verified', 'rejected', 'voided'))
);

CREATE TABLE leave_approvals (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id uuid NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    reviewer_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    action text NOT NULL CHECK (action IN ('approve', 'reject')),
    comment text,
    reviewed_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE file_retention_policies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_type text NOT NULL UNIQUE,
    retention_days integer NOT NULL CHECK (retention_days >= 0),
    audit_hold_days integer NOT NULL CHECK (audit_hold_days >= 0),
    cleanup_action text NOT NULL CHECK (cleanup_action IN ('physical_delete', 'archive_then_delete')),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled'))
);

CREATE TABLE audit_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id uuid REFERENCES users(id) ON DELETE SET NULL,
    action text NOT NULL,
    target_type text NOT NULL,
    target_id uuid,
    detail jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_target ON audit_logs(target_type, target_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
