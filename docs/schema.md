# Database Schema Draft

本文档从 `docs/domain.md` 拆出第一阶段数据库表草案。字段类型以 PostgreSQL 为目标，最终以迁移文件为准。

通用约定：
- 主键统一使用 `uuid`
- 时间字段使用 `timestamptz`
- 状态字段使用枚举或受约束的 `text`
- 软删除如有需要统一使用 `status`，第一阶段不默认加 `deleted_at`
- 业务图片、扫描件等文件统一进入 `file_objects`

## 用户与权限

### users

| 字段 | 说明 |
| --- | --- |
| id | 用户 ID |
| username | 登录名，唯一 |
| password_hash | 密码哈希 |
| phone | 手机号 |
| status | active / disabled |
| last_login_at | 最后登录时间 |
| created_at | 创建时间 |
| updated_at | 更新时间 |

### roles

| 字段 | 说明 |
| --- | --- |
| id | 角色 ID |
| name | 角色名称 |
| code | 角色编码，唯一 |
| description | 描述 |

### permissions

| 字段 | 说明 |
| --- | --- |
| id | 权限 ID |
| code | 权限编码，唯一 |
| description | 描述 |

### user_roles

| 字段 | 说明 |
| --- | --- |
| user_id | 用户 ID |
| role_id | 角色 ID |

唯一约束：`(user_id, role_id)`

### role_permissions

| 字段 | 说明 |
| --- | --- |
| role_id | 角色 ID |
| permission_id | 权限 ID |

唯一约束：`(role_id, permission_id)`

## 组织与住宿

### students

| 字段 | 说明 |
| --- | --- |
| id | 学生 ID |
| user_id | 关联用户，可为空 |
| student_no | 学号，唯一 |
| name | 姓名 |
| gender | male / female |
| phone | 手机号 |
| class_id | 所属班级 |
| college | 学院 |
| major | 专业 |
| class_name | 班级名称快照 |
| status | active / suspended / graduated / left |
| created_at | 创建时间 |
| updated_at | 更新时间 |

规则：
- `students.gender` 必须等于 `class_groups.gender`
- 非 active 学生不能创建住宿、签到、请假

### class_groups

| 字段 | 说明 |
| --- | --- |
| id | 班级 ID |
| name | 班级名称 |
| code | 班级编号，唯一 |
| college | 学院 |
| major | 专业 |
| grade | 年级 |
| gender | male / female |
| status | active / disabled |

规则：
- 一个班级内学生性别必须一致
- 班级只能使用同一性别的寝室

### teacher_class_assignments

| 字段 | 说明 |
| --- | --- |
| id | 授权 ID |
| teacher_id | 老师用户 ID |
| class_id | 班级 ID |
| assignment_type | counselor / teacher / temporary_manager |
| assigned_by | 授权人 |
| status | active / disabled |

唯一约束：`(teacher_id, class_id, assignment_type)`

### buildings

| 字段 | 说明 |
| --- | --- |
| id | 宿舍楼 ID |
| name | 名称 |
| code | 编号，唯一 |
| campus | 校区 |
| gender_policy | male_only / female_only / mixed |
| status | active / disabled |

### rooms

| 字段 | 说明 |
| --- | --- |
| id | 寝室 ID |
| building_id | 宿舍楼 ID |
| class_id | 归属班级 ID |
| gender | male / female |
| room_no | 寝室号 |
| floor | 楼层 |
| capacity | 容量 |
| status | active / disabled |

唯一约束：`(building_id, room_no)`

规则：
- `rooms.gender` 必须等于 `class_groups.gender`
- `rooms.gender` 必须符合 `buildings.gender_policy`
- 寝室必须固定归属一个班级
- 不允许跨班级入住，住宿记录中的学生班级必须等于寝室归属班级

### accommodations

| 字段 | 说明 |
| --- | --- |
| id | 住宿记录 ID |
| student_id | 学生 ID |
| room_id | 寝室 ID |
| check_in_at | 入住时间 |
| check_out_at | 退宿时间 |
| status | active / checked_out |

约束：
- 每个学生最多一条 active 住宿记录
- active 住宿记录必须满足学生班级、性别与寝室一致

## 学生职责与任务分配

### student_duty_assignments

| 字段 | 说明 |
| --- | --- |
| id | 职责分配 ID |
| student_id | 学生 ID |
| duty_type | class_monitor / inspection_executor / hygiene_executor |
| scope_type | room / class / building / task |
| scope_ids | 范围 ID 数组 |
| assigned_by | 分配人 |
| status | active / disabled |

### task_assignments

| 字段 | 说明 |
| --- | --- |
| id | 任务分配 ID |
| task_type | inspection / hygiene |
| task_id | 任务 ID |
| assignee_id | 执行人 ID |
| assignee_type | teacher / student |
| duty_assignment_id | 学生职责分配 ID，老师执行时为空 |
| assigned_by | 分配人 |
| assigned_at | 分配时间 |
| status | assigned / processing / completed / cancelled |

规则：
- 查寝任务分配给学生时必须关联 `inspection_executor`
- 卫生任务分配给学生时必须关联 `hygiene_executor`
- `status` 表示单个执行人的任务进度，不等同于任务整体状态

## 查寝

### dorm_inspection_tasks

| 字段 | 说明 |
| --- | --- |
| id | 查寝任务 ID |
| title | 标题 |
| scope_type | building / floor / room / class / student |
| scope_ids | 范围 ID 数组 |
| scheduled_at | 计划查寝时间 |
| method | manual_abnormal / qr_code |
| status | draft / published / processing / completed / cancelled |
| created_by | 创建人 |
| created_at | 创建时间 |

规则：
- 发布任务时，为范围内学生生成 `unknown` 查寝记录

### dorm_inspection_records

| 字段 | 说明 |
| --- | --- |
| id | 查寝记录 ID |
| task_id | 查寝任务 ID |
| student_id | 学生 ID |
| room_id | 寝室 ID |
| result | present / abnormal / leave / unknown |
| abnormal_reason | 异常原因 |
| evidence_source | check_in / qr_code / manual_abnormal / inspection_room_completed / other_verified |
| evidence_ref_id | 证据记录 ID |
| leave_request_id | 请假 ID |
| checked_by | 检查人 |
| checked_at | 检查时间 |

唯一约束：`(task_id, student_id)`

规则：
- result = present 且 evidence_source = qr_code 时，evidence_ref_id 关联 `dorm_inspection_qr_scans.id`
- result = present 时必须存在可信 evidence_source
- result = abnormal 时必须填写 abnormal_reason
- result = leave 时必须关联 approved 状态的 leave_request_id

### dorm_inspection_qr_tokens

| 字段 | 说明 |
| --- | --- |
| id | 二维码令牌 ID |
| task_id | 查寝任务 ID |
| inspection_record_id | 查寝记录 ID |
| student_id | 学生 ID |
| room_id | 寝室 ID |
| token_hash | 令牌哈希 |
| issued_at | 签发时间 |
| expires_at | 过期时间 |
| used_at | 使用时间 |
| status | active / used / expired / revoked |

索引：
- `token_hash` 唯一索引
- `(task_id, student_id, status)`
- `inspection_record_id` 在 status = active 时唯一

规则：
- 二维码由服务端签发，学生小程序只负责展示
- 只为 unknown 查寝记录生成 active 令牌
- 同一查寝记录同一时间最多只能存在一个 active 令牌
- 刷新二维码时，必须先将旧 active 令牌标记为 revoked，再签发新令牌
- 签发前必须校验学生当前存在 active Accommodation，且住宿寝室等于查寝记录 room_id
- 签发前必须校验查寝任务状态允许执行，且学生属于任务范围
- 签发前如查寝时间落在已批准请假时间段内，应优先将查寝记录标记为 leave，不生成 active 令牌
- token_hash 只保存哈希值，不保存明文 token
- 成功扫码后令牌状态变为 used
- 已使用、已过期或已作废令牌不能再次扫码通过

### dorm_inspection_qr_scans

| 字段 | 说明 |
| --- | --- |
| id | 扫码记录 ID |
| qr_token_id | 二维码令牌 ID |
| task_assignment_id | 任务分配 ID |
| task_id | 查寝任务 ID |
| inspection_record_id | 查寝记录 ID |
| student_id | 学生 ID |
| room_id | 寝室 ID |
| executor_type | teacher / student |
| executor_id | 执行人 ID |
| scanned_at | 扫码时间 |
| scan_result | accepted / rejected |
| reject_reason | 拒绝原因 |

规则：
- 执行人必须被分配该学生或该寝室的查寝任务
- task_assignment_id 必须指向本次扫码使用的任务分配
- executor_type 和 executor_id 必须与 task_assignment 的 assignee_type 和 assignee_id 一致
- 学生执行人必须具备 active inspection_executor 职责
- 扫码时必须校验二维码有效期、任务状态、执行人任务分配、职责范围、学生当前住宿寝室和查寝记录状态
- 扫码通过后，将对应查寝记录从 unknown 更新为 present
- 扫码通过后，查寝记录 evidence_source = qr_code，evidence_ref_id = dorm_inspection_qr_scans.id
- 扫码不能覆盖已存在的 present、leave 或 abnormal 结果
- 扫码失败也应保留 rejected 记录和 reject_reason

### dorm_inspection_room_completions

| 字段 | 说明 |
| --- | --- |
| id | 完成记录 ID |
| task_id | 查寝任务 ID |
| room_id | 寝室 ID |
| executor_id | 执行人 |
| completed_at | 完成时间 |

唯一约束：`(task_id, room_id, executor_id)`

规则：
- 第一阶段同一寝室默认只分配给一个执行人
- 提交完成后，仍为 unknown 的学生自动标记 abnormal
- 如果查寝时间落在已批准请假时间段内，应标记 leave 并关联请假记录，不生成 abnormal

### inspection_abnormal_appeals

| 字段 | 说明 |
| --- | --- |
| id | 申诉 ID |
| inspection_record_id | 查寝记录 ID |
| student_id | 学生 ID |
| appeal_type | leave / present |
| reason | 说明 |
| status | pending / approved / rejected |
| submitted_at | 提交时间 |
| reviewed_by | 审核人 |
| reviewed_at | 审核时间 |
| review_comment | 审核意见 |

约束：
- 同一异常记录同时最多一个 pending 申诉
- 已 approved 后不可再次提交
- 只能在异常记录 checked_at 后三天内提交
- 审核通过后按 appeal_type 将原查寝结果修正为 leave 或 present

### inspection_abnormal_appeal_files

| 字段 | 说明 |
| --- | --- |
| id | 记录 ID |
| appeal_id | 申诉 ID |
| file_object_id | 文件 ID |

## 在寝签到

### dorm_check_in_rules

| 字段 | 说明 |
| --- | --- |
| id | 规则 ID |
| title | 标题 |
| scope_type | building / class / room / student |
| scope_ids | 范围 ID 数组 |
| starts_at_time | 每日开始时间 |
| ends_at_time | 每日结束时间 |
| status | active / disabled |
| created_by | 创建人 |
| created_at | 创建时间 |

规则：
- 同一学生同一自然日最多命中一个 active 规则

### dorm_wifi_fingerprints

| 字段 | 说明 |
| --- | --- |
| id | WiFi 指纹 ID |
| room_id | 寝室 ID |
| ssid | WiFi 名称 |
| bssid | WiFi BSSID |
| min_signal_strength | 最小信号强度 |
| status | active / disabled |
| created_at | 创建时间 |
| updated_at | 更新时间 |

### dorm_check_in_records

| 字段 | 说明 |
| --- | --- |
| id | 签到记录 ID |
| rule_id | 签到规则 ID |
| student_id | 学生 ID |
| room_id | 寝室 ID |
| check_in_date | 签到日期 |
| status | submitted / late / missed / rejected |
| submitted_at | 提交时间 |
| matched_wifi_count | 匹配 WiFi 数 |
| scanned_wifi_count | 扫描 WiFi 数 |
| wifi_evidence | 脱敏判定证据 |
| reject_reason | 拒绝原因 |

唯一约束：`(student_id, check_in_date)`

## 卫生检查

### hygiene_score_templates

| 字段 | 说明 |
| --- | --- |
| id | 模板 ID |
| name | 模板名 |
| version | 版本 |
| status | draft / published / disabled |
| created_by | 创建人 |
| created_at | 创建时间 |

### hygiene_score_criteria

| 字段 | 说明 |
| --- | --- |
| id | 标准项 ID |
| template_id | 模板 ID |
| item_name | 项目名称 |
| max_score | 满分 |
| sort_order | 排序 |

规则：
- 模板中的所有评分项都是必填项，不存在可选评分项
- `max_score` 必须大于 0

### hygiene_tasks

| 字段 | 说明 |
| --- | --- |
| id | 卫生任务 ID |
| title | 标题 |
| scope_type | building / floor / class / room |
| scope_ids | 范围 ID 数组 |
| scheduled_at | 计划时间 |
| score_template_id | 评分模板 ID |
| status | draft / published / processing / completed / cancelled |
| created_by | 创建人 |

规则：
- 只能引用 published 状态模板

### hygiene_records

| 字段 | 说明 |
| --- | --- |
| id | 卫生记录 ID |
| task_id | 卫生任务 ID |
| room_id | 寝室 ID |
| total_score | 总分 |
| level | excellent / good / passed / failed |
| comment | 总体说明 |
| checked_by | 检查人 |
| checked_at | 检查时间 |

唯一约束：`(task_id, room_id)`

### hygiene_score_items

| 字段 | 说明 |
| --- | --- |
| id | 明细 ID |
| hygiene_record_id | 卫生记录 ID |
| criterion_id | 评分标准项 ID |
| item_name | 项目名称快照 |
| score | 得分 |
| max_score | 满分快照 |
| comment | 说明 |

规则：
- 每个模板评分项都必须生成一条评分明细
- `score` 不能小于 0，不能大于 `max_score`
- `total_score` 必须等于所有评分明细得分之和

### hygiene_photos

| 字段 | 说明 |
| --- | --- |
| id | 照片记录 ID |
| hygiene_record_id | 卫生记录 ID |
| file_object_id | 文件 ID |
| photo_type | 照片类型 |

规则：
- 每条卫生检查记录至少关联一张照片
- 照片文件统一通过 `file_objects` 管理

## 请假

### leave_requests

| 字段 | 说明 |
| --- | --- |
| id | 请假 ID |
| student_id | 学生 ID |
| leave_type | 请假类型 |
| source | online / paper / appeal |
| appeal_id | 异常申诉 ID，仅 source = appeal 时使用 |
| reason | 原因 |
| starts_at | 开始时间 |
| ends_at | 结束时间 |
| status | pending / approved / rejected / withdrawn / cancelled |
| submitted_at | 提交时间 |
| reviewed_by | 审核人 |
| reviewed_at | 审核时间 |
| review_comment | 审核意见 |

规则：
- 同一学生不能存在时间重叠的 pending / approved 请假
- source = paper 时，批准前至少一条纸质假条凭证为 verified

### paper_leave_evidences

| 字段 | 说明 |
| --- | --- |
| id | 凭证 ID |
| leave_request_id | 请假 ID |
| student_id | 学生 ID |
| file_object_id | 文件 ID |
| uploaded_by | 凭证提交人 |
| uploaded_at | 凭证提交时间 |
| verified_by | 核验人 |
| verified_at | 核验时间 |
| status | uploaded / verified / rejected / voided |

### leave_approvals

| 字段 | 说明 |
| --- | --- |
| id | 审批 ID |
| leave_request_id | 请假 ID |
| reviewer_id | 审核人 |
| action | approve / reject |
| comment | 意见 |
| reviewed_at | 审核时间 |

## 文件与审计

### file_objects

| 字段 | 说明 |
| --- | --- |
| id | 文件 ID |
| owner_type | 归属类型冗余索引 |
| owner_id | 归属 ID 冗余索引 |
| file_url | 文件访问地址 |
| file_hash | 文件哈希 |
| mime_type | MIME 类型 |
| size_bytes | 文件大小 |
| storage_key | 存储键 |
| uploaded_by | 上传人 |
| uploaded_at | 上传时间 |
| retention_until | 最早可清理时间 |
| status | active / pending_delete / deleted / delete_failed |
| deleted_at | 删除时间 |
| delete_error | 删除失败原因 |

### file_retention_policies

| 字段 | 说明 |
| --- | --- |
| id | 策略 ID |
| owner_type | 归属类型 |
| retention_days | 业务保留天数 |
| audit_hold_days | 审计保留天数 |
| cleanup_action | 清理动作 |
| status | active / disabled |

规则：
- `cleanup_action` 第一阶段使用 physical_delete；后续可扩展 archive_then_delete
- 清理任务只能处理超过 retention_until 且无有效业务引用的文件
- 每次清理应写入 audit_logs，记录释放空间、文件数量和失败原因

### audit_logs

| 字段 | 说明 |
| --- | --- |
| id | 日志 ID |
| actor_id | 操作人 |
| action | 操作 |
| target_type | 目标类型 |
| target_id | 目标 ID |
| detail | 脱敏详情 |
| created_at | 创建时间 |
