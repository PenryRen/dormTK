use dormtk_core::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

pub mod auth {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoginUser {
        pub id: Id,
        pub username: String,
        pub role_ids: Vec<Id>,
        pub roles: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoginResponse {
        pub access_token: String,
        pub user: LoginUser,
    }
}

pub mod access {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: Id,
        pub username: String,
        pub phone: Option<String>,
        pub status: UserStatus,
        pub last_login_at: Option<DateTimeString>,
        pub created_at: Option<DateTimeString>,
        pub updated_at: Option<DateTimeString>,
        pub role_ids: Vec<Id>,
        pub roles: Vec<Role>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserCreate {
        pub username: String,
        pub password: String,
        pub phone: Option<String>,
        pub status: Option<UserStatus>,
        pub role_ids: Option<Vec<Id>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserUpdate {
        pub username: Option<String>,
        pub password: Option<String>,
        pub phone: Option<String>,
        pub status: Option<UserStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Role {
        pub id: Id,
        pub name: String,
        pub code: String,
        pub description: Option<String>,
        pub permission_ids: Vec<Id>,
        pub permissions: Vec<Permission>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RoleUpsert {
        pub name: String,
        pub code: String,
        pub description: Option<String>,
        pub permission_ids: Option<Vec<Id>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Permission {
        pub id: Id,
        pub code: String,
        pub description: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PermissionUpsert {
        pub code: String,
        pub description: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RoleIdsRequest {
        pub role_ids: Vec<Id>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PermissionIdsRequest {
        pub permission_ids: Vec<Id>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ImportJobCreate {
        pub import_type: String,
        pub file_object_id: Id,
    }
}

pub mod basics {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Student {
        pub id: Id,
        pub user_id: Option<Id>,
        pub student_no: String,
        pub name: String,
        pub gender: Gender,
        pub phone: Option<String>,
        pub class_id: Id,
        pub college: Option<String>,
        pub major: Option<String>,
        pub class_name: Option<String>,
        pub status: StudentStatus,
        pub created_at: Option<DateTimeString>,
        pub updated_at: Option<DateTimeString>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StudentUpsert {
        pub student_no: String,
        pub name: String,
        pub gender: Gender,
        pub phone: Option<String>,
        pub class_id: Id,
        pub college: Option<String>,
        pub major: Option<String>,
        pub status: Option<StudentStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClassGroup {
        pub id: Id,
        pub name: String,
        pub code: String,
        pub college: Option<String>,
        pub major: Option<String>,
        pub grade: Option<String>,
        pub gender: Gender,
        pub status: UserStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClassGroupUpsert {
        pub name: String,
        pub code: String,
        pub college: Option<String>,
        pub major: Option<String>,
        pub grade: Option<String>,
        pub gender: Gender,
        pub status: Option<UserStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TeacherClassAssignment {
        pub id: Id,
        pub teacher_id: Id,
        pub class_id: Id,
        pub assignment_type: TeacherClassAssignmentType,
        pub assigned_by: Option<Id>,
        pub status: TeacherClassAssignmentStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TeacherClassAssignmentCreate {
        pub teacher_id: Id,
        pub assignment_type: TeacherClassAssignmentType,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Building {
        pub id: Id,
        pub name: String,
        pub code: String,
        pub campus: Option<String>,
        pub gender_policy: BuildingGenderPolicy,
        pub status: DormRoomStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BuildingUpsert {
        pub name: String,
        pub code: String,
        pub campus: Option<String>,
        pub gender_policy: BuildingGenderPolicy,
        pub status: Option<DormRoomStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Room {
        pub id: Id,
        pub building_id: Id,
        pub class_id: Id,
        pub gender: Gender,
        pub room_no: String,
        pub floor: Option<i32>,
        pub capacity: u32,
        pub status: DormRoomStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RoomUpsert {
        pub building_id: Id,
        pub class_id: Id,
        pub gender: Gender,
        pub room_no: String,
        pub floor: Option<i32>,
        pub capacity: u32,
        pub status: Option<DormRoomStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Accommodation {
        pub id: Id,
        pub student_id: Id,
        pub room_id: Id,
        pub check_in_at: DateTimeString,
        pub check_out_at: Option<DateTimeString>,
        pub status: AccommodationStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AccommodationCreate {
        pub student_id: Id,
        pub room_id: Id,
        pub check_in_at: DateTimeString,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CheckoutRequest {
        pub check_out_at: Option<DateTimeString>,
    }
}

pub mod duties {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StudentDutyAssignment {
        pub id: Id,
        pub student_id: Id,
        pub duty_type: StudentDutyType,
        pub scope_type: DutyScopeType,
        pub scope_ids: Vec<Id>,
        pub assigned_by: Option<Id>,
        pub status: DutyAssignmentStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StudentDutyAssignmentCreate {
        pub student_id: Id,
        pub duty_type: StudentDutyType,
        pub scope_type: DutyScopeType,
        pub scope_ids: Vec<Id>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StudentDutyAssignmentUpdate {
        pub scope_ids: Option<Vec<Id>>,
        pub status: Option<DutyAssignmentStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskAssignment {
        pub id: Id,
        pub task_type: TaskType,
        pub task_id: Id,
        pub assignee_id: Id,
        pub assignee_type: AssigneeType,
        pub duty_assignment_id: Option<Id>,
        pub assigned_by: Option<Id>,
        pub assigned_at: Option<DateTimeString>,
        pub status: TaskAssignmentStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskAssignmentCreate {
        pub task_type: TaskType,
        pub task_id: Id,
        pub assignee_id: Id,
        pub assignee_type: AssigneeType,
        pub duty_assignment_id: Option<Id>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskAssignmentUpdate {
        pub status: Option<TaskAssignmentStatus>,
    }
}

pub mod inspection {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionTask {
        pub id: Id,
        pub title: String,
        pub scope_type: ScopeType,
        pub scope_ids: Vec<Id>,
        pub scheduled_at: DateTimeString,
        pub method: InspectionMethod,
        pub status: TaskStatus,
        pub created_by: Option<Id>,
        pub created_at: Option<DateTimeString>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionTaskCreate {
        pub title: String,
        pub scope_type: ScopeType,
        pub scope_ids: Vec<Id>,
        pub scheduled_at: DateTimeString,
        pub method: InspectionMethod,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionTaskUpdate {
        pub title: Option<String>,
        pub scheduled_at: Option<DateTimeString>,
        pub status: Option<TaskStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionRecord {
        pub id: Id,
        pub task_id: Id,
        pub student_id: Id,
        pub room_id: Id,
        pub result: InspectionResult,
        pub abnormal_reason: Option<String>,
        pub evidence_source: Option<InspectionEvidenceSource>,
        pub evidence_ref_id: Option<Id>,
        pub leave_request_id: Option<Id>,
        pub checked_by: Option<Id>,
        pub checked_at: Option<DateTimeString>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InspectionAbnormalRequest {
        pub abnormal_reason: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InspectionPresentByEvidenceRequest {
        pub evidence_source: InspectionEvidenceSource,
        pub evidence_ref_id: Id,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InspectionLeaveRequest {
        pub leave_request_id: Id,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionQrToken {
        pub id: Id,
        pub task_id: Id,
        pub inspection_record_id: Id,
        pub student_id: Id,
        pub room_id: Id,
        pub token: String,
        pub issued_at: Option<DateTimeString>,
        pub expires_at: DateTimeString,
        pub used_at: Option<DateTimeString>,
        pub status: InspectionQrTokenStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionQrScan {
        pub id: Id,
        pub qr_token_id: Id,
        pub task_assignment_id: Id,
        pub task_id: Id,
        pub inspection_record_id: Id,
        pub student_id: Id,
        pub room_id: Id,
        pub executor_type: AssigneeType,
        pub executor_id: Id,
        pub scanned_at: DateTimeString,
        pub scan_result: InspectionQrScanResult,
        pub reject_reason: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionQrScanSubmit {
        pub token: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormInspectionRoomCompletion {
        pub id: Id,
        pub task_id: Id,
        pub room_id: Id,
        pub executor_id: Id,
        pub completed_at: DateTimeString,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InspectionAbnormalAppeal {
        pub id: Id,
        pub inspection_record_id: Id,
        pub student_id: Id,
        pub appeal_type: InspectionAbnormalAppealType,
        pub reason: String,
        pub status: InspectionAbnormalAppealStatus,
        pub submitted_at: DateTimeString,
        pub reviewed_by: Option<Id>,
        pub reviewed_at: Option<DateTimeString>,
        pub review_comment: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InspectionAbnormalAppealCreate {
        pub appeal_type: InspectionAbnormalAppealType,
        pub reason: String,
        pub file_object_ids: Vec<Id>,
    }
}

pub mod check_in {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormCheckInRule {
        pub id: Id,
        pub title: String,
        pub scope_type: ScopeType,
        pub scope_ids: Vec<Id>,
        pub starts_at_time: TimeString,
        pub ends_at_time: TimeString,
        pub status: UserStatus,
        pub created_by: Option<Id>,
        pub created_at: Option<DateTimeString>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormCheckInRuleCreate {
        pub title: String,
        pub scope_type: ScopeType,
        pub scope_ids: Vec<Id>,
        pub starts_at_time: TimeString,
        pub ends_at_time: TimeString,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormCheckInRuleUpdate {
        pub title: Option<String>,
        pub starts_at_time: Option<TimeString>,
        pub ends_at_time: Option<TimeString>,
        pub status: Option<UserStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormWifiFingerprint {
        pub id: Id,
        pub room_id: Id,
        pub ssid: String,
        pub bssid: String,
        pub min_signal_strength: i32,
        pub status: WifiFingerprintStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormWifiFingerprintCreate {
        pub ssid: String,
        pub bssid: String,
        pub min_signal_strength: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormWifiFingerprintUpdate {
        pub ssid: Option<String>,
        pub bssid: Option<String>,
        pub min_signal_strength: Option<i32>,
        pub status: Option<WifiFingerprintStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ScannedWifi {
        pub ssid: String,
        pub bssid: String,
        pub signal_strength: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormCheckInSubmit {
        pub wifi_list: Vec<ScannedWifi>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormCheckInRecord {
        pub id: Id,
        pub rule_id: Option<Id>,
        pub student_id: Id,
        pub room_id: Id,
        pub check_in_date: DateString,
        pub status: CheckInStatus,
        pub submitted_at: Option<DateTimeString>,
        pub matched_wifi_count: Option<u32>,
        pub scanned_wifi_count: Option<u32>,
        pub wifi_evidence: Option<serde_json::Value>,
        pub reject_reason: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DormCheckInToday {
        pub rule: Option<DormCheckInRule>,
        pub record: Option<DormCheckInRecord>,
    }
}

pub mod hygiene {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneScoreCriterion {
        pub id: Id,
        pub template_id: Id,
        pub item_name: String,
        pub max_score: f64,
        pub sort_order: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneScoreCriterionCreate {
        pub item_name: String,
        pub max_score: f64,
        pub sort_order: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneScoreTemplate {
        pub id: Id,
        pub name: String,
        pub version: i32,
        pub status: HygieneScoreTemplateStatus,
        pub criteria: Vec<HygieneScoreCriterion>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneScoreTemplateCreate {
        pub name: String,
        pub criteria: Vec<HygieneScoreCriterionCreate>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneScoreTemplateUpdate {
        pub name: Option<String>,
        pub status: Option<HygieneScoreTemplateStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneTask {
        pub id: Id,
        pub title: String,
        pub scope_type: ScopeType,
        pub scope_ids: Vec<Id>,
        pub scheduled_at: DateTimeString,
        pub score_template_id: Id,
        pub status: TaskStatus,
        pub created_by: Option<Id>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneTaskCreate {
        pub title: String,
        pub scope_type: ScopeType,
        pub scope_ids: Vec<Id>,
        pub scheduled_at: DateTimeString,
        pub score_template_id: Id,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneTaskUpdate {
        pub title: Option<String>,
        pub scheduled_at: Option<DateTimeString>,
        pub status: Option<TaskStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneScoreItem {
        pub criterion_id: Id,
        pub item_name: String,
        pub score: f64,
        pub max_score: f64,
        pub comment: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygienePhoto {
        pub file_object_id: Id,
        pub photo_type: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneRecord {
        pub id: Id,
        pub task_id: Id,
        pub room_id: Id,
        pub total_score: f64,
        pub level: Option<HygieneLevel>,
        pub comment: Option<String>,
        pub checked_by: Id,
        pub checked_at: DateTimeString,
        pub items: Vec<HygieneScoreItem>,
        pub photos: Vec<HygienePhoto>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HygieneRecordCreate {
        pub room_id: Id,
        pub comment: Option<String>,
        pub items: Vec<HygieneScoreItem>,
        pub photos: Vec<HygienePhoto>,
    }
}

pub mod leave {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LeaveRequest {
        pub id: Id,
        pub student_id: Id,
        pub leave_type: String,
        pub source: LeaveSource,
        pub appeal_id: Option<Id>,
        pub reason: String,
        pub starts_at: DateTimeString,
        pub ends_at: DateTimeString,
        pub status: LeaveStatus,
        pub submitted_at: Option<DateTimeString>,
        pub reviewed_by: Option<Id>,
        pub reviewed_at: Option<DateTimeString>,
        pub review_comment: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LeaveRequestCreate {
        pub leave_type: String,
        pub reason: String,
        pub starts_at: DateTimeString,
        pub ends_at: DateTimeString,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PaperLeaveEvidence {
        pub id: Id,
        pub leave_request_id: Id,
        pub student_id: Id,
        pub file_object_id: Id,
        pub uploaded_by: Option<Id>,
        pub uploaded_at: Option<DateTimeString>,
        pub verified_by: Option<Id>,
        pub verified_at: Option<DateTimeString>,
        pub status: PaperLeaveEvidenceStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PaperLeaveEvidenceCreate {
        pub file_object_id: Id,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReviewRequest {
        pub comment: Option<String>,
    }
}

pub mod files {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileObject {
        pub id: Id,
        pub owner_type: String,
        pub owner_id: Option<Id>,
        pub file_url: String,
        pub file_hash: String,
        pub mime_type: String,
        pub size_bytes: u64,
        pub storage_key: String,
        pub uploaded_by: Option<Id>,
        pub uploaded_at: Option<DateTimeString>,
        pub retention_until: Option<DateTimeString>,
        pub status: FileObjectStatus,
        pub deleted_at: Option<DateTimeString>,
        pub delete_error: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileUploadRequest {
        pub owner_type: String,
        pub owner_id: Option<Id>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileRelationRequest {
        pub file_object_id: Id,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileRetentionPolicy {
        pub id: Id,
        pub owner_type: String,
        pub retention_days: u32,
        pub audit_hold_days: u32,
        pub cleanup_action: FileCleanupAction,
        pub status: FileRetentionPolicyStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileRetentionPolicyCreate {
        pub owner_type: String,
        pub retention_days: u32,
        pub audit_hold_days: u32,
        pub cleanup_action: FileCleanupAction,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileRetentionPolicyUpdate {
        pub retention_days: Option<u32>,
        pub audit_hold_days: Option<u32>,
        pub cleanup_action: Option<FileCleanupAction>,
        pub status: Option<FileRetentionPolicyStatus>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileCleanupRun {
        pub deleted_count: u32,
        pub failed_count: u32,
        pub released_bytes: u64,
    }
}

pub mod student {
    use super::{basics::*, duties::*, *};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StudentMe {
        pub student: Student,
        pub accommodation: Option<Accommodation>,
        pub duties: Vec<StudentDutyAssignment>,
    }
}

pub use auth::*;
pub use basics::*;
pub use check_in::*;
pub use duties::*;
pub use files::*;
pub use hygiene::*;
pub use inspection::*;
pub use leave::*;
pub use student::*;
