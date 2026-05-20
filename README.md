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
├── docs/                            # 架构、API、部署文档
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
