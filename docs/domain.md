# Domain

dormTK 是宿舍管理系统，核心目标是管理学生宿舍，并支持日常查寝、卫生检查、在寝签到、请假审批、任务分配等业务。

本文档描述业务对象、业务规则、角色权限和第一阶段范围。它是后续设计数据库、API、管理端和小程序页面的依据。

## 核心身份与职责

### SystemAdmin

系统管理员负责系统级配置和账号权限管理。

可以：
- 管理用户账号
- 管理角色
- 管理权限
- 给角色分配权限
- 给用户分配角色
- 查看权限变更记录

规则：
- 系统管理员权限最高，应尽量少量配置
- 系统管理员可以跨班级处理业务数据，不受 TeacherClassAssignment 限制；这只表示管理权限可以跨班级，不表示住宿允许跨班级
- 删除角色前，必须确认没有用户正在使用该角色
- 权限变更应记录操作人、操作时间和变更内容

### Teacher

教师或辅导员是非学生身份，负责查看和管理所管班级的宿舍事务。

可以：
- 查看所管班级或学生的住宿信息
- 创建和发布查寝任务
- 创建和发布卫生检查任务
- 将查寝、卫生检查等任务分配给学生管理角色
- 查看查寝结果
- 查看卫生检查结果
- 查看请假记录
- 查看异常记录
- 审批学生请假

规则：
- 一个老师可以负责多个班级
- 一个班级也可以由多个老师共同负责
- 老师的数据查看、任务创建和审批范围由 TeacherClassAssignment 决定

### Student

学生通过微信小程序使用系统。除老师和系统管理员外，其余日常管理职责都由学生担任。

可以：
- 查看本人住宿信息
- 进行在寝签到
- 提交请假申请
- 查看本人请假审批状态
- 查看本人查寝状态
- 查看本人宿舍卫生记录

规则：
- 学生只能查看和操作自己的数据
- 学生不能修改自己的宿舍分配信息
- 学生提交请假后，在审批完成前可以撤回，审批后不可直接修改

### StudentDuty

学生职责表示学生在宿舍管理中承担的管理职责。

典型职责：
- class_monitor: 班长
- inspection_executor: 查寝执行人
- hygiene_executor: 卫生检查执行人

规则：
- 学生可以同时拥有多个职责
- 学生职责必须有管理范围，例如所属班级、指定寝室列表或指定任务
- 学生职责只能授予学生用户
- 同一个学生可以同时拥有班长、查寝执行人、卫生检查执行人等多个职责
- 学生职责不等于系统管理员权限，不能管理系统角色和全局权限
- 学生执行查寝或卫生检查时，必须具备对应的任务执行人职责，并且只能处理被授权范围内的数据

### ClassMonitor

班长是学生职责之一，负责协助老师管理本班宿舍事务。

可以：
- 查看本班寝室列表
- 查看本班查寝任务执行状态
- 查看本班卫生检查结果
- 查看本班寝室卫生检查评分明细和照片

规则：
- 班长默认只管理本人所属班级
- 班长不能仅凭班长职责执行查寝任务或卫生检查任务
- 如果班长需要执行查寝或卫生检查，必须同时拥有 inspection_executor 或 hygiene_executor 职责，并被分配对应任务

## 组织与住宿对象

### Student

学生是住宿管理的主体。

字段：
- id
- student_no
- name
- gender
- phone
- class_id
- college
- major
- class_name
- status
- created_at
- updated_at

规则：
- 学号唯一
- 学生性别必须与所属班级性别一致
- 一个学生同一时间最多只能有一个有效住宿记录
- 已离校或停用学生不能创建新的住宿、签到、请假记录
- 学生所属班级用于确定可分配寝室范围和统计口径

### ClassGroup

班级是寝室归属和统计管理的基础单位。

字段：
- id
- name
- code
- college
- major
- grade
- gender
- status

规则：
- 班级编号唯一
- 一个班级内学生性别必须一致
- 一个寝室同一时间只归属一个班级
- 学生只能分配到本人班级归属的寝室
- 班级只能归属或使用同性别寝室
- 教师或辅导员查看范围按 TeacherClassAssignment 授权的班级确定

### TeacherClassAssignment

教师班级授权表示老师和班级之间的管理关系。

字段：
- id
- teacher_id
- class_id
- assignment_type
- assigned_by
- status

规则：
- 一个老师可以绑定多个班级
- 一个班级可以绑定多个老师
- assignment_type 可以区分辅导员、任课老师、临时负责人等
- 只有绑定到班级的老师，才能查看该班级宿舍信息、创建该班级范围内的任务或审批该班级学生请假
- 停用的授权不再生效

### Building

宿舍楼表示一栋宿舍建筑。

字段：
- id
- name
- code
- campus
- gender_policy
- status

规则：
- 宿舍楼编号唯一
- 宿舍楼可以设置为男生楼、女生楼或混住楼
- 混住楼可以同时包含男寝和女寝
- 如果宿舍楼设置了单一性别限制，楼内寝室性别必须符合该限制
- 停用宿舍楼下的寝室不能继续分配

### Room

寝室表示宿舍楼内的具体宿舍房间，是学生住宿、查寝、卫生检查的基础空间单位。

字段：
- id
- building_id
- class_id
- gender
- room_no
- floor
- capacity
- status

规则：
- 同一宿舍楼内寝室号唯一
- 寝室必须归属到一个班级
- 寝室必须标明性别，分为男寝或女寝
- 寝室性别必须与归属班级性别一致
- 停用寝室不能分配学生
- 寝室在住人数不能超过 capacity
- 寝室性别必须符合宿舍楼 gender_policy；mixed 表示宿舍楼允许同时存在男寝和女寝
- 第一阶段不精确管理到床位，只管理到寝室

### Accommodation

住宿记录表示学生和寝室之间的入住关系。

字段：
- id
- student_id
- room_id
- check_in_at
- check_out_at
- status

规则：
- active 住宿记录表示学生当前在住
- 创建 active 住宿记录时，学生进入对应寝室在住名单
- 退宿后，住宿记录变为 checked_out，学生不再计入寝室在住人数
- 一个学生同一时间只能有一个 active 住宿记录
- 学生只能入住本人班级归属的寝室
- 学生性别、班级性别和寝室性别必须一致
- 不允许跨班级入住

## 用户、角色与权限

### User

用户是系统登录主体，可以绑定学生、教师或管理员身份。

字段：
- id
- username
- password_hash
- phone
- status
- last_login_at

规则：
- 用户名唯一
- 用户可拥有多个角色
- 用户停用后不能登录

### Role

角色是一组权限的集合。

字段：
- id
- name
- code
- description

规则：
- 角色编码唯一
- 权限通过角色授予用户
- 不同用户可以通过不同角色组合获得不同权限
- Role 和 Permission 只表达系统权限，不表达学生职责范围
- 学生是否能查看班级、执行查寝或执行卫生检查，由 StudentDutyAssignment 和 TaskAssignment 决定

### StudentDutyAssignment

学生职责分配表示某个学生在一定范围内承担某种宿舍管理职责。

字段：
- id
- student_id
- duty_type
- scope_type
- scope_ids
- assigned_by
- status

规则：
- student_id 必须指向学生身份
- duty_type 表示班长、查寝执行人或卫生检查执行人等职责
- scope_type 可以是 room、class、building、task
- 班长职责范围必须是本人所属班级，不能配置为其他班级
- 查寝执行人和卫生检查执行人职责本身不设置有效期
- 具体任务分配提供本次任务的执行范围，不能替代学生职责
- 班长职责只提供班级范围内的查看和协助管理能力，不提供任务执行能力
- 停用学生不能继续承担职责

### Permission

权限表示系统中的一个可授权动作。

示例：
- user:create
- user:update
- user:delete
- role:assign
- dorm:read
- dorm:write
- inspection:create
- inspection:assign
- inspection:execute
- hygiene:create
- hygiene:assign
- hygiene:score
- leave:approve

规则：
- 权限应按模块和动作拆分
- 管理端按钮、菜单和 API 都应检查权限
- 权限变更应写入审计日志
- 学生访问管理能力时，应同时检查系统权限、学生职责和职责范围

## 查寝

查寝用于确认学生是否在寝，并记录异常情况。

### DormInspectionTask

查寝任务表示一次需要执行的查寝安排。

字段：
- id
- title
- scope_type
- scope_ids
- scheduled_at
- method
- status
- created_by
- created_at

规则：
- 查寝任务可以按宿舍楼、楼层、寝室、班级或学生范围创建
- 查寝任务发布时，应为任务范围内的学生生成 unknown 查寝记录
- 查寝任务创建后可以分配给老师或具备查寝执行职责的学生
- 学生执行人只能查看和提交分配给自己的查寝任务
- 学生执行人只能处理任务范围内的寝室或学生
- 日常签到和查寝任务都用于确认学生是否在寝
- 同一时间范围内同时存在日常签到和查寝任务结果时，优先采纳查寝执行人提交的查寝结果
- 已完成任务不能修改查寝范围

### DormInspectionRoomCompletion

寝室检查完成记录表示某个执行人已经完成指定寝室的查寝。

字段：
- id
- task_id
- room_id
- executor_id
- completed_at

规则：
- 只有被分配该寝室查寝任务的执行人可以提交检查完成
- 同一任务、同一寝室、同一执行人只能提交一次检查完成
- 如果同一寝室被分配给多个执行人，任一执行人提交完成只表示该执行人的检查范围完成
- 寝室整体完成状态应由任务分配规则决定；第一阶段默认同一寝室只分配给一个执行人
- 执行人提交寝室检查完成后，该执行人负责检查的寝室进入完成状态
- 寝室检查完成后，该寝室内仍处于 unknown 的学生会自动生成 abnormal 查寝结果
- 自动生成的异常结果应记录 evidence_source 为 inspection_room_completed
- 已有 present、leave 或 abnormal 结果的学生不被自动覆盖
- 如果查寝时间落在已批准请假的时间段内，自动生成结果时应优先标记为 leave

### DormInspectionRecord

查寝记录表示某个学生在一次查寝中的结果。

字段：
- id
- task_id
- student_id
- room_id
- result
- abnormal_reason
- evidence_source
- evidence_ref_id
- leave_request_id
- checked_by
- checked_at

规则：
- 同一任务中，一个学生只能有一条查寝记录
- 学生执行人只能手动标记异常，不能手动标记在寝
- 学生执行人可以通过扫描学生小程序展示的查寝二维码，将查寝记录确认为在寝；这属于二维码证据确认，不属于人工手动标记在寝
- 老师和系统管理员也不应仅凭人工操作直接标记在寝；在寝结果应来自签到、二维码或其他可信证据
- 查寝结果为 abnormal 时，必须填写异常原因
- 查寝时间落在已批准请假时间段内时，查寝结果应标记为 leave，并关联 leave_request_id
- result 为 present 时，必须有 evidence_source，例如当日有效在寝签到记录或查寝二维码记录
- result 为 present 且 evidence_source 为 qr_code 时，evidence_ref_id 应关联 DormInspectionQrScan
- result 为 abnormal 时，可以来自执行人手动标记，也可以来自寝室检查完成后的自动标记
- abnormal 结果允许学生在 checked_at 后三天内提交原因说明和图片进行申诉
- checked_by 可以是老师，也可以是被分配该任务的学生执行人

### DormInspectionQrToken

查寝二维码令牌表示学生小程序为某次查寝展示的一次性或短时有效二维码。

字段：
- id
- task_id
- inspection_record_id
- student_id
- room_id
- token_hash
- issued_at
- expires_at
- used_at
- status

规则：
- 只有任务范围内、当前有 active Accommodation 且存在 unknown 查寝记录的学生可以生成查寝二维码
- 二维码必须由服务端签发，学生小程序只负责展示，不应由前端自行拼接可信业务数据
- 二维码应短时有效，过期后必须重新生成
- token_hash 用于服务端校验二维码，不保存明文 token
- 二维码只能用于对应 task_id、student_id、room_id 的查寝确认
- 二维码被成功扫描后应标记为 used，防止重复使用
- 如果学生已被标记为 leave、present 或 abnormal，默认不再生成新的可用二维码
- 如果查寝时间落在已批准请假时间段内，应优先标记为 leave，不应通过二维码改为 present

### DormInspectionQrScan

查寝二维码扫描记录表示执行人通过微信小程序扫描学生二维码后的核验结果。

字段：
- id
- qr_token_id
- task_assignment_id
- task_id
- inspection_record_id
- student_id
- room_id
- executor_type
- executor_id
- scanned_at
- scan_result
- reject_reason

规则：
- 只有被分配该学生或该寝室查寝任务的执行人可以提交扫码结果
- 执行人必须具备 inspection_executor 职责，或是被分配该任务的老师
- task_assignment_id 记录本次扫码使用的任务分配，便于区分老师执行和学生执行
- executor_type 表示 teacher 或 student，executor_id 对应执行人身份 ID
- 扫码时必须校验二维码有效期、任务状态、执行人任务分配、学生当前住宿寝室和查寝记录状态
- 扫码成功后，将对应 DormInspectionRecord 从 unknown 更新为 present
- 扫码成功时，DormInspectionRecord.evidence_source 记为 qr_code，evidence_ref_id 关联本次 DormInspectionQrScan
- 扫码只能确认在寝，不能用于请假审批，也不能覆盖已存在的 leave、present 或 abnormal 结果
- 扫码失败应保留失败原因，便于排查过期二维码、越权扫码或学生寝室不匹配等问题

### InspectionAbnormalAppeal

查寝异常申诉表示学生对 abnormal 查寝结果提交的原因说明和证明材料。

字段：
- id
- inspection_record_id
- student_id
- appeal_type
- reason
- status
- submitted_at
- reviewed_by
- reviewed_at
- review_comment

规则：
- 只有 abnormal 查寝记录可以提交异常申诉
- 同一条 abnormal 查寝记录同一时间只能有一个 pending 申诉
- abnormal 查寝记录已有 approved 申诉后，不能再次提交申诉
- rejected 申诉允许在三天期限内重新提交
- 学生必须在异常标记时间 checked_at 后三天内提交申诉
- appeal_type 表示申请修正为请假或在寝
- 申请修正为请假时，应补充请假原因和必要证明
- 申请修正为请假审核通过后，必须关联已有 approved LeaveRequest，或自动创建一条 source 为 appeal 的 approved LeaveRequest
- 自动创建 source 为 appeal 的 LeaveRequest 时，必须通过有效请假重叠校验；如果存在可覆盖该查寝时间的 approved LeaveRequest，应优先关联已有请假
- 申请修正为请假审核通过后，原查寝结果修正为 leave，并写入 leave_request_id
- 申请修正为在寝时，应提交能证明当时在寝的说明或图片；审核通过后，原查寝结果修正为 present，evidence_source 记为 other_verified
- 异常申诉图片通过 InspectionAbnormalAppealFile 关联 FileObject
- 异常申诉必须由老师或系统管理员审核
- 审核通过或拒绝后应保留原始 abnormal 记录的审计轨迹

### InspectionAbnormalAppealFile

查寝异常申诉文件表示学生提交的异常说明图片或附件。

字段：
- id
- appeal_id
- file_object_id

规则：
- 一个 InspectionAbnormalAppeal 可以关联多个 FileObject
- 文件元数据、存储路径和清理状态由 FileObject 统一管理
- 单次申诉允许上传的文件数量、单文件大小和文件类型由文件上传策略限制

### 查寝方式

第一阶段支持：
- manual_abnormal: 人工标记异常
- qr_code: 扫描查寝二维码

后续可扩展：
- bluetooth: 蓝牙辅助定位
- location: 校园网或定位辅助

规则：
- 二维码查寝流程为：学生小程序请求查寝二维码，服务端签发短时有效令牌，执行人小程序扫码提交，服务端校验后将该学生查寝结果标记为 present
- 二维码查寝应有有效期
- 过期二维码不能继续用于查寝确认
- 人工查寝只能用于记录异常，不能作为在寝证明
- 人工查寝需要记录执行人

## 在寝签到

在寝签到用于学生每天主动确认自己在寝室。第一阶段每天最多签到一次，签到是否在寝通过扫描学生手机附近的 WiFi 列表进行范围判定。

### DormCheckInRule

签到规则表示每日在寝签到的时间窗口和适用范围。

字段：
- id
- title
- scope_type
- scope_ids
- starts_at_time
- ends_at_time
- status
- created_by
- created_at

规则：
- 在寝签到按自然日计算，每个学生每天最多有一条有效签到记录
- 同一学生同一自然日最多命中一个 active DormCheckInRule
- 如果多个 active DormCheckInRule 同时覆盖同一学生，应视为配置冲突并禁止发布
- 签到只能在 starts_at_time 和 ends_at_time 之间提交
- 签到规则可以按宿舍楼、班级、寝室或学生范围生效
- 学生必须有 active Accommodation 才能参与签到
- 停用的签到规则不再生成或接受新的签到

### DormWifiFingerprint

寝室 WiFi 指纹表示一个寝室附近可用于范围判定的 WiFi 信息。

字段：
- id
- room_id
- ssid
- bssid
- min_signal_strength
- status
- created_at
- updated_at

规则：
- WiFi 指纹归属到寝室
- 同一寝室可以配置多个 WiFi 指纹
- bssid 优先用于识别具体 WiFi 设备，ssid 只作为辅助信息
- min_signal_strength 用于判断学生是否足够接近该 WiFi
- 停用的 WiFi 指纹不参与签到判定
- 管理员应定期维护 WiFi 指纹，避免路由器更换后无法签到

### DormCheckInRecord

签到记录表示学生某一天的一次在寝签到结果。

字段：
- id
- rule_id
- student_id
- room_id
- check_in_date
- status
- submitted_at
- matched_wifi_count
- scanned_wifi_count
- wifi_evidence
- reject_reason

规则：
- 同一学生同一自然日只能有一条有效在寝签到记录
- 签到记录唯一性以 student_id 和 check_in_date 为准
- 日常签到是基础在寝证据；如果同一时间范围内存在查寝任务结果，应优先采纳查寝结果
- 签到时使用学生当前 active Accommodation 的 room_id 作为寝室判定依据
- 学生提交签到时，小程序上传当前可扫描到的 WiFi 列表
- 服务端将扫描到的 WiFi 列表与该寝室的 DormWifiFingerprint 进行匹配
- 至少匹配到一个启用的寝室 WiFi 指纹，且信号强度满足要求，才可判定为 submitted
- 未匹配到有效 WiFi 指纹时，签到结果为 rejected
- 超出签到时间窗口提交时，签到结果为 late 或 rejected，具体策略由系统配置决定
- wifi_evidence 应保存必要的判定信息，用于申诉和审计
- wifi_evidence 不应保存无关 WiFi 的完整敏感信息，建议只保存匹配结果、摘要或脱敏数据

## 卫生检查

卫生检查用于对宿舍卫生进行评分、记录和统计。

### HygieneTask

卫生检查任务表示一次检查安排。

字段：
- id
- title
- scope_type
- scope_ids
- scheduled_at
- score_template_id
- status
- created_by

规则：
- 卫生任务可以按宿舍楼、楼层、班级或寝室创建
- 卫生任务应引用一个 HygieneScoreTemplate
- 卫生任务只能引用 published 状态的 HygieneScoreTemplate
- 任务可以分配给老师或具备卫生检查职责的学生
- 学生检查人只能查看和提交分配给自己的卫生任务
- 学生检查人只能处理任务范围内的寝室
- 已完成任务不能继续新增评分，除非重新打开

### HygieneRecord

卫生检查记录表示一个寝室的一次卫生结果汇总。

字段：
- id
- task_id
- room_id
- total_score
- level
- comment
- checked_by
- checked_at

规则：
- 同一卫生任务中，一个寝室只能有一条最终评分
- 卫生评分由多个评分项明细组成
- total_score 为所有评分项得分汇总
- 评分时必须上传寝室照片
- 评分低于合格线时，应填写总体说明或扣分原因
- 卫生结果可按寝室、班级、宿舍楼统计
- 班长可以查询本人所属班级寝室的卫生汇总、评分明细和照片
- checked_by 可以是老师，也可以是被分配该任务的学生检查人

### HygieneScoreTemplate

卫生评分模板定义一次卫生检查需要评分的项目集合。

字段：
- id
- name
- version
- status
- created_by
- created_at

规则：
- 已发布并被任务引用的模板不应直接修改
- 如果评分标准变化，应创建新版本模板
- 同一任务内所有寝室应使用同一评分模板，确保结果可比较

### HygieneScoreCriterion

卫生评分标准项表示模板中的一个评分项目。

字段：
- id
- template_id
- item_name
- max_score
- sort_order

规则：
- 一个 HygieneScoreTemplate 至少包含一个 HygieneScoreCriterion
- max_score 必须大于 0
- 卫生评分不存在可选评分项，模板中的所有评分项都必须提交得分

### HygieneScoreItem

卫生评分项表示一个具体检查项的评分明细。

字段：
- id
- hygiene_record_id
- criterion_id
- item_name
- score
- max_score
- comment

规则：
- 一个 HygieneRecord 必须包含至少一个 HygieneScoreItem
- HygieneScoreItem 应由 HygieneScoreCriterion 生成
- item_name 和 max_score 应保留提交时的快照，避免模板后续变化影响历史记录
- 每个评分项得分不能超过 max_score
- 每个 HygieneScoreCriterion 都必须生成一条 HygieneScoreItem
- total_score 应等于所有 HygieneScoreItem.score 之和
- 评分明细必须长期保留，不能只保存总分
- 班长可以查询本人所属班级寝室的评分明细

### HygienePhoto

卫生检查照片表示评分时上传的寝室现场照片。

字段：
- id
- hygiene_record_id
- file_object_id
- photo_type

规则：
- 一个 HygieneRecord 必须至少关联一张 HygienePhoto
- HygienePhoto 通过 file_object_id 关联 FileObject
- 文件上传人、上传时间和文件大小以 FileObject 为准
- 单次卫生检查允许上传的照片数量、单文件大小和文件类型由文件上传策略限制
- 照片用于证明卫生检查现场情况
- 班长可以查询本人所属班级寝室的卫生检查照片

## 请假

请假用于学生申请不在寝或离校，并供查寝时判断异常。

### LeaveRequest

请假申请表示学生的一次请假。

字段：
- id
- student_id
- leave_type
- source
- appeal_id
- reason
- starts_at
- ends_at
- status
- submitted_at
- reviewed_by
- reviewed_at
- review_comment

规则：
- 请假开始时间必须早于结束时间
- 同一学生不能存在时间重叠的有效请假
- 有效请假指 pending 或 approved 状态的 LeaveRequest
- rejected、withdrawn、cancelled 状态不参与重叠判断
- pending 状态可以撤回
- approved 状态可被查寝记录引用；查寝时间落在请假时间段内时，查寝结果应标记为 leave
- rejected 状态不应作为查寝豁免依据
- source 表示线上申请、纸质假条补录或查寝异常申诉补录
- source = paper 时，必须至少存在一条关联的 PaperLeaveEvidence
- source = paper 的请假最终批准前，至少需要一条 PaperLeaveEvidence 状态为 verified
- source = appeal 时，必须关联 InspectionAbnormalAppeal
- source 不是 appeal 时，appeal_id 必须为空
- 纸质假条经老师或系统管理员批准后，等同于线上请假

### PaperLeaveEvidence

纸质假条凭证表示学生线下获得的纸质请假证明。

字段：
- id
- leave_request_id
- student_id
- file_object_id
- uploaded_by
- uploaded_at
- verified_by
- verified_at
- status

规则：
- 纸质假条可以由学生上传，也可以由老师或管理员代为补录
- 纸质假条必须关联到一条 LeaveRequest
- PaperLeaveEvidence 通过 file_object_id 关联 FileObject
- 纸质假条状态为 verified 后，对应请假申请才可以被批准为有效请假
- 被拒绝或作废的纸质假条不能作为查寝豁免依据

### LeaveApproval

请假审批表示一次审批动作。

字段：
- id
- leave_request_id
- reviewer_id
- action
- comment
- reviewed_at

规则：
- 审批动作包括 approve、reject
- 审批后应保留审批记录
- 审批人必须有 leave:approve 权限

## 任务分配

任务分配用于将查寝、卫生检查等任务指派给具体执行人。

### TaskAssignment

字段：
- id
- task_type
- task_id
- assignee_id
- assignee_type
- duty_assignment_id
- assigned_by
- assigned_at
- status

规则：
- 一个任务可以分配给多个执行人
- assignee_type 表示 teacher 或 student
- 分配查寝任务给学生时，应关联 inspection_executor 职责
- 分配卫生检查任务给学生时，应关联 hygiene_executor 职责
- class_monitor 职责不能作为任务执行职责
- 执行人只能处理分配给自己的任务，系统管理员和任务创建人除外
- 学生执行人必须同时满足任务分配和职责范围要求
- TaskAssignment.status 使用 TaskAssignmentStatus，表示某个执行人的本次任务进度，不等同于任务整体状态
- 任务完成后，分配状态应变为 completed

## 审计与日志

系统中的关键操作应记录审计日志。

需要记录的操作：
- 用户登录
- 新增、修改、删除用户
- 修改用户角色
- 修改角色权限
- 创建查寝任务
- 提交查寝结果
- 创建卫生任务
- 提交卫生评分
- 上传卫生检查照片
- 提交请假
- 上传纸质假条
- 核验纸质假条
- 审批请假
- 提交查寝异常申诉
- 审核查寝异常申诉
- 上传文件
- 标记文件删除
- 执行文件清理任务

字段：
- id
- actor_id
- action
- target_type
- target_id
- detail
- created_at

规则：
- 审计日志不可由普通用户修改或删除
- 敏感信息不应明文写入日志

## 文件与存储清理

系统中的图片、扫描件等大文件需要统一管理，避免长期占用服务器空间。

### FileObject

文件对象表示系统中上传并持久化保存的文件。

适用范围：
- 卫生检查照片
- 纸质假条图片或扫描件
- 查寝异常申诉图片
- 后续其他业务附件

字段：
- id
- owner_type
- owner_id
- file_url
- file_hash
- mime_type
- size_bytes
- storage_key
- uploaded_by
- uploaded_at
- retention_until
- status
- deleted_at
- delete_error

规则：
- 文件元数据统一保存在 FileObject 中，业务附件表通过 file_object_id 关联文件
- owner_type 和 owner_id 是文件归属的冗余索引，必须与业务附件表的关联关系保持一致
- 大文件应记录 size_bytes，便于统计空间占用
- file_hash 用于去重、校验和审计
- retention_until 表示文件最早可清理时间
- 仍被有效业务记录引用的文件不能被物理删除
- 文件删除应先标记为 pending_delete，再由后台清理任务执行物理删除
- 清理成功后状态变为 deleted，并记录 deleted_at
- 清理失败时状态变为 delete_failed，并记录 delete_error

### FileRetentionPolicy

文件保留策略表示不同业务文件的保存时间和清理规则。

字段：
- id
- owner_type
- retention_days
- audit_hold_days
- cleanup_action
- status

规则：
- 不同业务文件可以设置不同保留时间
- 第一阶段按 owner_type 配置保留策略
- 后续如有需要，可按业务状态进一步细化保留策略
- 卫生检查照片可设置较短保留期
- 纸质假条和查寝异常申诉图片涉及请假和审核，应设置较长保留期
- audit_hold_days 表示审计保留期
- 审计保留期内的文件不能清理
- retention_until 应综合业务保留期和审计保留期计算
- 清理任务应定期扫描超过 retention_until 且无有效引用的文件
- 清理任务应记录清理日志，包括文件数量、释放空间、执行时间和执行结果

## 状态枚举

### UserStatus

- active: 正常
- disabled: 停用

### StudentDutyType

- class_monitor: 班长
- inspection_executor: 查寝执行人
- hygiene_executor: 卫生检查执行人

### DutyScopeType

- room: 寝室范围
- class: 班级范围
- building: 宿舍楼范围
- task: 指定任务范围

### DutyAssignmentStatus

- active: 生效
- disabled: 停用

### TeacherClassAssignmentType

- counselor: 辅导员
- teacher: 任课老师
- temporary_manager: 临时负责人

### TeacherClassAssignmentStatus

- active: 生效
- disabled: 停用

### StudentStatus

- active: 正常在校
- suspended: 暂停
- graduated: 已毕业
- left: 已离校

### Gender

- male: 男
- female: 女

### BuildingGenderPolicy

- male_only: 仅男生
- female_only: 仅女生
- mixed: 混住楼，允许同时存在男寝和女寝

### DormRoomStatus

- active: 正常
- disabled: 停用

### AccommodationStatus

- active: 当前入住
- checked_out: 已退宿

### TaskStatus

- draft: 草稿
- published: 已发布
- processing: 进行中
- completed: 已完成
- cancelled: 已取消

### TaskAssignmentStatus

- assigned: 已分配
- processing: 执行中
- completed: 已完成
- cancelled: 已取消

### InspectionResult

- present: 证据确认在寝
- abnormal: 异常
- leave: 已请假
- unknown: 未确认

### InspectionEvidenceSource

- check_in: 当日在寝签到
- qr_code: 执行人扫描学生小程序展示的查寝二维码
- manual_abnormal: 执行人手动标记异常
- inspection_room_completed: 寝室检查完成后自动标记异常
- other_verified: 其他已验证证据

### InspectionQrTokenStatus

- active: 可使用
- used: 已使用
- expired: 已过期
- revoked: 已作废

### InspectionQrScanResult

- accepted: 扫码通过
- rejected: 扫码拒绝

### InspectionAbnormalAppealType

- leave: 申请修正为请假
- present: 申请修正为在寝

### InspectionAbnormalAppealStatus

- pending: 待审核
- approved: 已通过
- rejected: 已拒绝

### LeaveSource

- online: 线上请假
- paper: 纸质假条补录
- appeal: 查寝异常申诉补录

### PaperLeaveEvidenceStatus

- uploaded: 已上传
- verified: 已核验
- rejected: 已拒绝
- voided: 已作废

### CheckInStatus

- submitted: 已签到
- late: 迟到签到
- missed: 未签到
- rejected: 签到无效

### WifiFingerprintStatus

- active: 启用
- disabled: 停用

### HygieneScoreTemplateStatus

- draft: 草稿
- published: 已发布
- disabled: 停用

### FileObjectStatus

- active: 正常
- pending_delete: 待物理删除
- deleted: 已物理删除
- delete_failed: 删除失败

### FileRetentionPolicyStatus

- active: 生效
- disabled: 停用

### FileCleanupAction

- physical_delete: 到期后物理删除文件
- archive_then_delete: 先归档再删除原始文件

### HygieneLevel

- excellent: 优秀
- good: 良好
- passed: 合格
- failed: 不合格

### LeaveStatus

- pending: 待审批
- approved: 已通过
- rejected: 已拒绝
- withdrawn: 已撤回
- cancelled: 已取消

## 第一阶段范围

第一阶段先实现：
- 用户、角色、权限管理
- 学生职责分配和范围控制
- 学生、班级、宿舍楼、寝室管理
- 学生住宿分配和退宿
- 管理端登录
- 查寝任务创建、分配、执行、结果查询
- 学生查寝二维码生成和执行人扫码确认在寝
- 卫生检查任务创建、评分、结果查询
- 卫生评分模板和评分项管理
- 班长查看授权范围内的寝室管理信息、卫生评分明细和照片
- 学生小程序在寝签到
- 寝室 WiFi 指纹配置和维护
- 学生小程序请假申请
- 纸质假条上传、补录和核验
- 查寝异常申诉提交和审核
- 文件保留策略和定期清理任务
- 管理端请假审批
- 基础审计日志

暂不实现：
- 蓝牙定位
- 消息推送
- 复杂多级审批流

## 关键业务约束

- 学生当前住宿信息以 active Accommodation 为准。
- 查寝、签到、请假都应围绕当前住宿关系展开。
- 日常签到和查寝任务都用于判断学生是否在寝；两者都有结果时，优先采纳查寝任务结果。
- 查寝执行人提交寝室检查完成后，该寝室内仍未确认的学生自动标记为异常。
- 查寝执行人只能人工标记异常，不能人工标记在寝。
- 查寝执行人可以通过扫描学生小程序展示的查寝二维码确认在寝；在寝结果必须来自签到、二维码或其他可信证据。
- 查寝时间落在已批准请假的时间段内时，查寝结果标记为请假。
- 被标记为异常的学生，可以在异常标记后三天内提交原因说明和图片。
- 查寝异常申诉经老师或系统管理员审核通过后，根据申诉类型将原结果修正为请假或在寝。
- 请假审批通过后，可以作为查寝不在寝的合理原因。
- 经老师或系统管理员批准的纸质假条，等同于线上请假。
- 寝室必须归属班级，班级是统计、授权和任务范围的重要维度。
- 一个班级内学生性别一致，寝室按男寝、女寝区分。
- 学生、班级、寝室三者性别必须一致。
- 卫生检查结果绑定寝室，不直接绑定个人。
- 卫生检查必须基于评分模板提交，保留寝室照片和各评分项明细，最终总分由评分项汇总得到。
- 图片、扫描件等大文件必须纳入文件保留策略，定期清理无有效引用且超过保留期的文件。
- 除老师和系统管理员外，日常管理角色都由学生通过 StudentDutyAssignment 承担。
- 学生管理角色必须同时受权限和职责范围约束。
- 权限控制同时影响管理端菜单、按钮和后端 API。
- 微信小程序只面向学生，管理类操作只在管理端完成。
