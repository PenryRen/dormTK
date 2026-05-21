# dormTK

dormTK 是一个多端项目，当前规划为 Rust 后端服务、Rust + Slint 管理端原生应用、微信小程序客户端，以及一套后端容器化开发/部署环境。

## 项目结构

```text
.
├── backend/                         # Rust 后端服务
│   ├── Cargo.toml
│   ├── config/
│   │   ├── dev.toml
│   │   └── prod.example.toml
│   └── src/
│
├── clients/
│   ├── admin-desktop/               # Rust + Slint 管理端原生应用
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── src/
│   │   └── ui/
│   │
│   └── weapp/                       # 微信小程序客户端
│       ├── project.config.json
│       └── miniprogram/
│
├── crates/                          # Rust 共享库
│   ├── dormtk-api-types/            # 后端与管理端共享 API 类型
│   └── dormtk-core/                 # 纯业务类型与通用枚举
│
├── shared/
│   └── openapi/
│       └── openapi.yaml             # 跨语言 API 契约，供小程序生成类型
│
├── env/
│   ├── dev/                         # 本地容器开发环境
│   │   ├── backend/Containerfile
│   │   ├── podman-compose.yml
│   │   └── PostgreSQL/
│   │
│   └── prod/                        # 生产后端与数据库部署环境
│       ├── backend/Containerfile
│       ├── podman-compose.yml
│       └── PostgreSQL/
│
├── docs/                            # 领域、架构、数据库、API、部署文档
├── scripts/                         # 常用开发/构建脚本
├── Cargo.toml                       # Rust workspace
├── Cargo.lock
└── README.md
```

## 开发环境

依赖：

```text
podman >= 5.8.0
podman-compose >= 1.5.0
```

启动后端开发容器和 PostgreSQL：

```bash
./scripts/dev-backend.sh
```

进入后端开发容器：

```bash
podman exec -it dormTK-dev zsh
```

启动 Slint 管理端：

```bash
./scripts/dev-admin.sh
```

微信小程序使用微信开发者工具打开：

```text
clients/weapp
```

## 生产部署

生产容器只包含后端服务和 PostgreSQL。管理端原生应用按系统单独打包分发，小程序通过微信开发者工具上传。

在 `env/prod` 下创建 `.env`：

```bash
cp env/prod/.env.example env/prod/.env
```

编辑数据库变量：

```dotenv
DB_NAME=dormtk
DB_USER=dormtk
DB_PASSWORD=change-me
```

启动生产环境：

```bash
cd env/prod
podman-compose up -d --build
```

## API 契约

跨端接口契约放在：

```text
shared/openapi/openapi.yaml
```

Rust 后端和 Slint 管理端优先共享 `crates/dormtk-api-types` 中的类型；微信小程序通过 OpenAPI 生成或对照 TypeScript 类型。

## 项目推进计划

当前优先目标是先打通“后端数据模型 + 管理端基础资料 + 微信小程序查寝执行”的最小业务闭环，再逐步补齐请假、卫生检查、文件清理和多端体验。

### 阶段计划

| 阶段 | 目标 | 主要产出 | 完成标准 |
| --- | --- | --- | --- |
| P0 | 固化业务和数据边界 | `docs/domain.md`、`docs/schema.md`、`docs/api.md` | 查寝、签到、请假、卫生、住宿、职责分配规则无明显冲突 |
| P1 | 后端基础设施 | 配置加载、数据库连接、迁移、错误格式、认证骨架 | 后端可启动，能连接 PostgreSQL，迁移可重复执行 |
| P2 | 基础资料管理 | 用户、学生、班级、宿舍楼、寝室、住宿、教师班级授权、学生职责 API | 管理端可维护查寝所需基础数据 |
| P3 | 查寝最小闭环 | 查寝任务、任务分配、unknown 记录生成、学生二维码、执行人扫码、异常标记、寝室完成 | 一次查寝任务能从发布执行到完成，并产出 present/abnormal/leave 结果 |
| P4 | 请假与异常申诉 | 线上请假、纸质假条、审批、异常三天内申诉 | 请假能影响查寝结果，异常能通过审核修正为请假或在寝 |
| P5 | 卫生检查 | 评分模板、卫生任务、评分明细、照片上传、班长查询 | 卫生检查能按寝室提交总分、明细和照片 |
| P6 | 文件与审计 | FileObject、文件保留策略、定期清理任务、审计日志 | 图片/扫描件可追踪、可清理、关键操作可审计 |
| P7 | 客户端完善 | Slint 管理端页面、微信小程序页面、OpenAPI 类型同步 | 常用业务能在对应客户端完成 |
| P8 | 部署与验收 | 生产容器、备份策略、初始化脚本、测试数据、验收清单 | 可在生产环境部署并完成端到端验收 |

### TODO

#### P0 文档与契约

- [x] 完成领域模型草案：`docs/domain.md`
- [x] 完成数据库表草案：`docs/schema.md`
- [x] 完成 API 草案：`docs/api.md`
- [x] 将 API 草案整理为 `shared/openapi/openapi.yaml`
- [x] 将核心枚举同步到 `crates/dormtk-core`
- [x] 将 API 请求/响应类型同步到 `crates/dormtk-api-types`

#### P1 后端基础设施

- [x] 选择并接入数据库迁移方案，优先使用 `sqlx` migrations
- [x] 创建第一批 PostgreSQL migration
- [x] 实现后端配置加载和环境区分
- [x] 实现数据库连接池
- [x] 统一 API 错误响应格式
- [x] 实现健康检查和基础日志
- [x] 搭建认证 token 骨架

#### P2 基础资料管理

- [x] 用户、角色、权限基础表和接口
- [x] 学生、班级、教师班级授权接口
- [x] 宿舍楼、寝室接口
- [x] 住宿分配和退宿接口
- [x] 学生职责分配接口
- [x] 基础资料导入预留设计

#### P3 查寝最小闭环

- [ ] 创建和发布查寝任务
- [ ] 发布任务时生成学生 `unknown` 查寝记录
- [ ] 分配查寝执行人
- [ ] 学生小程序生成短时有效查寝二维码
- [ ] 执行人小程序扫码确认在寝
- [ ] 执行人手动标记异常
- [ ] 执行人提交寝室检查完成
- [ ] 寝室完成后将剩余 `unknown` 自动处理为 `abnormal` 或 `leave`

#### P4 请假与异常申诉

- [ ] 学生线上请假申请
- [ ] 纸质假条上传和补录
- [ ] 老师或管理员审批请假
- [ ] 查寝时按已批准请假优先标记 `leave`
- [ ] 异常记录三天内申诉
- [ ] 申诉审核后修正为 `leave` 或 `present`

#### P5 卫生检查

- [ ] 卫生评分模板和必填评分项
- [ ] 卫生任务创建和分配
- [ ] 卫生评分提交
- [ ] 卫生照片上传
- [ ] 班长查询本班卫生汇总、明细和照片

#### P6 文件、审计与清理

- [ ] 统一文件上传 API
- [ ] 建立 FileObject 记录
- [ ] 文件大小、类型、数量限制
- [ ] 文件保留策略配置
- [ ] 定期清理超过保留期且无有效引用的文件
- [ ] 关键业务操作审计日志

#### P7 客户端

- [ ] Slint 管理端登录和基础布局
- [ ] Slint 管理端基础资料管理页面
- [ ] Slint 管理端查寝任务页面
- [ ] 微信小程序登录和本人住宿信息
- [ ] 微信小程序在寝签到
- [ ] 微信小程序查寝二维码展示
- [ ] 微信小程序执行人查寝任务、扫码、异常、完成页面
- [ ] 微信小程序请假和异常申诉页面

#### P8 部署与验收

- [ ] 开发环境一键初始化数据库
- [ ] 生产环境配置模板补齐
- [ ] 数据库备份和恢复说明
- [ ] 初始化管理员账号脚本
- [ ] 测试数据种子脚本
- [ ] 端到端验收清单
