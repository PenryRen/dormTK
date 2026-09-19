# API Draft

本文档描述第一阶段 API 草案。最终以 `shared/openapi/openapi.yaml` 和后端实现为准。

通用约定：
- 管理端 API 前缀：`/api/admin`
- 学生小程序 API 前缀：`/api/student`
- 返回 JSON
- 管理端使用登录 token
- 学生端使用学生身份 token
- 分页参数统一使用 `page`、`page_size`

## 通用

### GET /health

健康检查。

响应：

```json
{
  "status": "ok"
}
```

## 认证

### POST /api/admin/auth/login

管理端登录。

请求：

```json
{
  "username": "admin",
  "password": "password"
}
```

响应：

```json
{
  "access_token": "...",
  "user": {
    "id": "...",
    "username": "admin"
  }
}
```

## 管理端：基础资料

### Students

- `GET /api/admin/students`
- `POST /api/admin/students`
- `GET /api/admin/students/{id}`
- `PATCH /api/admin/students/{id}`

### Classes

- `GET /api/admin/classes`
- `POST /api/admin/classes`
- `GET /api/admin/classes/{id}`
- `PATCH /api/admin/classes/{id}`
- `POST /api/admin/classes/{id}/teachers`
- `DELETE /api/admin/classes/{id}/teachers/{teacher_id}`

### Buildings

- `GET /api/admin/buildings`
- `POST /api/admin/buildings`
- `GET /api/admin/buildings/{id}`
- `PATCH /api/admin/buildings/{id}`

### Rooms

- `GET /api/admin/rooms`
- `POST /api/admin/rooms`
- `GET /api/admin/rooms/{id}`
- `PATCH /api/admin/rooms/{id}`

### Accommodations

- `GET /api/admin/accommodations`
- `POST /api/admin/accommodations`
- `POST /api/admin/accommodations/{id}/checkout`

创建住宿时必须校验：
- 学生当前无 active 住宿
- 学生班级等于寝室归属班级
- 学生、班级、寝室性别一致

## 管理端：职责与任务

### Student Duties

- `GET /api/admin/student-duty-assignments`
- `POST /api/admin/student-duty-assignments`
- `PATCH /api/admin/student-duty-assignments/{id}`

### Task Assignments

- `GET /api/admin/task-assignments`
- `POST /api/admin/task-assignments`
- `PATCH /api/admin/task-assignments/{id}`

规则：
- 学生执行人必须已有对应职责，班长职责不能单独作为查寝或卫生任务执行依据
- 任务分配状态只表示该执行人的进度，不代表任务整体状态

## 管理端：查寝

### Inspection Tasks

- `GET /api/admin/inspection-tasks`
- `POST /api/admin/inspection-tasks`
- `GET /api/admin/inspection-tasks/{id}`
- `PATCH /api/admin/inspection-tasks/{id}`
- `POST /api/admin/inspection-tasks/{id}/publish`
- `POST /api/admin/inspection-tasks/{id}/complete`

发布任务时生成范围内学生的 `unknown` 记录。

### Inspection Records

- `GET /api/admin/inspection-tasks/{task_id}/records`
- `PATCH /api/admin/inspection-records/{id}/abnormal`
- `PATCH /api/admin/inspection-records/{id}/present-by-evidence`
- `PATCH /api/admin/inspection-records/{id}/leave`

规则：
- 学生执行人只能手动标记异常，不能手动标记在寝
- 学生执行人可以通过扫描学生小程序展示的查寝二维码确认在寝；这属于二维码证据，不属于人工标记在寝
- 在寝结果必须有可信证据

### Room Completion

- `POST /api/admin/inspection-tasks/{task_id}/rooms/{room_id}/complete`

提交寝室检查完成后：
- 已有结果不覆盖
- unknown 自动标记 abnormal
- 已批准请假时间覆盖查寝时间时标记 leave
- 管理员或老师代处理完成状态时，必须受系统权限、任务范围和 TeacherClassAssignment 班级授权约束

### Abnormal Appeals

- `GET /api/admin/inspection-appeals`
- `GET /api/admin/inspection-appeals/{id}`
- `POST /api/admin/inspection-appeals/{id}/approve`
- `POST /api/admin/inspection-appeals/{id}/reject`

审核规则：
- appeal_type 为 leave 时，审核通过后关联或创建已批准请假，并将查寝结果修正为 leave
- appeal_type 为 present 时，审核通过后将查寝结果修正为 present，证据来源为 other_verified

## 管理端：在寝签到

### Check-in Rules

- `GET /api/admin/check-in-rules`
- `POST /api/admin/check-in-rules`
- `PATCH /api/admin/check-in-rules/{id}`

### WiFi Fingerprints

- `GET /api/admin/rooms/{room_id}/wifi-fingerprints`
- `POST /api/admin/rooms/{room_id}/wifi-fingerprints`
- `PATCH /api/admin/wifi-fingerprints/{id}`

### Check-in Records

- `GET /api/admin/check-in-records`

## 管理端：卫生检查

### Score Templates

- `GET /api/admin/hygiene-score-templates`
- `POST /api/admin/hygiene-score-templates`
- `PATCH /api/admin/hygiene-score-templates/{id}`
- `POST /api/admin/hygiene-score-templates/{id}/publish`

### Hygiene Tasks

- `GET /api/admin/hygiene-tasks`
- `POST /api/admin/hygiene-tasks`
- `GET /api/admin/hygiene-tasks/{id}`
- `PATCH /api/admin/hygiene-tasks/{id}`
- `POST /api/admin/hygiene-tasks/{id}/publish`

### Hygiene Records

- `GET /api/admin/hygiene-tasks/{task_id}/records`
- `POST /api/admin/hygiene-tasks/{task_id}/records`
- `GET /api/admin/hygiene-records/{id}`

提交卫生记录时：
- 必须提交模板中所有评分项
- 必须上传至少一张照片
- 总分由评分项汇总
- 不接受客户端传入的最终总分作为可信值，后端按评分明细重新计算

## 管理端：请假

### Leave Requests

- `GET /api/admin/leave-requests`
- `GET /api/admin/leave-requests/{id}`
- `POST /api/admin/leave-requests/{id}/approve`
- `POST /api/admin/leave-requests/{id}/reject`

### Paper Evidence

- `POST /api/admin/leave-requests/{id}/paper-evidences`
- `POST /api/admin/paper-leave-evidences/{id}/verify`
- `POST /api/admin/paper-leave-evidences/{id}/reject`

## 管理端：文件

### Files

- `POST /api/admin/files`
- `GET /api/admin/files`
- `POST /api/admin/files/{id}/mark-delete`
- `POST /api/admin/files/cleanup-runs`

文件规则：
- 图片、扫描件等大文件上传后统一生成 FileObject
- 清理任务只能删除超过 retention_until 且无有效业务引用的文件
- 清理结果必须写入审计日志

### Retention Policies

- `GET /api/admin/file-retention-policies`
- `POST /api/admin/file-retention-policies`
- `PATCH /api/admin/file-retention-policies/{id}`

## 学生端

### Me

- `GET /api/student/me`
- `GET /api/student/accommodation`

### Daily Check-in

- `GET /api/student/check-in/today`
- `POST /api/student/check-in`

提交签到时上传附近 WiFi 列表，服务端按寝室 WiFi 指纹判定。

### Leave

- `GET /api/student/leave-requests`
- `POST /api/student/leave-requests`
- `POST /api/student/leave-requests/{id}/withdraw`
- `POST /api/student/leave-requests/{id}/paper-evidences`

### Inspection

- `GET /api/student/inspection-records`
- `GET /api/student/inspection-records/{id}/qr-token`
- `POST /api/student/inspection-qr-tokens/{id}/refresh`
- `POST /api/student/inspection-records/{id}/appeals`
- `POST /api/student/inspection-appeals/{id}/files`

异常申诉必须在异常标记后三天内提交。

查寝二维码只对当前学生本人、当前查寝任务和短时间窗口有效；学生端展示二维码，执行人端扫码提交。

### Inspection Execution

- `GET /api/student/inspection-execution/tasks`
- `GET /api/student/inspection-execution/tasks/{task_id}`
- `GET /api/student/inspection-execution/tasks/{task_id}/rooms/{room_id}/records`
- `POST /api/student/inspection-execution/qr-scans`
- `PATCH /api/student/inspection-execution/records/{id}/abnormal`
- `POST /api/student/inspection-execution/tasks/{task_id}/rooms/{room_id}/complete`

执行人小程序规则：
- 只有具备 inspection_executor 职责且被分配任务的学生，才能访问执行接口
- 执行人只能查看和处理自己被分配的任务、寝室和学生
- 手动操作只能标记 abnormal，不能手动标记 present
- 扫描学生二维码后提交 token，服务端校验通过才可将查寝记录标记为 present
- 学生执行人只能提交分配给自己的寝室完成信息
- 寝室完成后，剩余 unknown 按查寝完成规则自动处理

### Hygiene

- `GET /api/student/hygiene-records`

普通学生只能查看本人寝室相关卫生记录；班长可以查看本人班级寝室的卫生汇总、评分明细和照片。

### Files

- `POST /api/student/files`

学生端文件上传受文件数量、大小和类型策略限制。
