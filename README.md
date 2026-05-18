# 一.项目介绍
(占位符,以后再写喵)

# 二.项目结构
~~~
.
├── README.md --------------------------说明文档
├── app --------------------------------程序目录
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── config.dev.toml ----------------开发环境配置文件
│   ├── config.prod.toml ---------------生产环境配置文件
│   └── src ----------------------------代码目录
│ 
└── env --------------------------------容器环境目录
    ├── dev ----------------------------开发环境目录
    │   ├── PostgreSQL -----------------PostgreSQL数据库目录
    │   ├── init.sh --------------------开发容器环境初始化脚本
    │   ├── podman-compose.yml ---------开发容器环境编排文件
    │   └── rust -----------------------rust开发镜像目录
    └── prod ---------------------------生产环境目录
        ├── PostgreSQL -----------------PostgreSQL数据库目录
        ├── podman-compose.yml ---------生产环境编排文件
        └── rust -----------------------rust生产镜像目录
~~~

# 三.快速开始
0.准备依赖环境
---
~~~
podman >= 5.8.0
podman-compose >= 1.5.0
~~~
安装方式见[Podman Installation Instructions](https://podman.io/docs/installation)

1.克隆项目到本地
---
~~~ bash
git clone https://github.com/PenryRen/dormTK.git
~~~
使用git将仓库拉取到本地

2.配置生产环境变量
---
在 `./dormTK/env/prod`下建立`.env`文件,并配置数据库密码
~~~ bash
# 数据库连接信息
DB_HOST=db                         # Compose 服务名或容器内部名称
DB_PORT=5432                       # PostgreSQL 默认端口
DB_NAME=dormtk                     # 数据库名称
DB_USER=dormtk                     # 数据库账号
DB_PASSWORD=super-secret-password  # 数据库密码
~~~
之后编辑
~~~ bash
./dormTK/app/config.prod.toml
~~~
完成数据库配置
~~~ toml
# config.prod.toml
db_host = "db"
db_port = 5432
db_name = "dormtk"
db_user = "dormtk"
db_password = "super-secret-password"
~~~
之后在`./dormTK/env/prod`目录下运行
~~~ bash
podman-compose up -d
~~~
完成部署


# 四.开发环境配置
0.准备依赖环境
---
~~~
podman >= 5.8.0
podman-compose >= 1.5.0
~~~
安装方式见[Podman Installation Instructions](https://podman.io/docs/installation)

1.克隆项目到本地
---
~~~ bash
git clone https://github.com/PenryRen/dormTK.git
~~~
使用git将仓库拉取到本地

2.初始化开发环境
---
切换到开发环境目录
~~~ bash
cd ./dormTK/env
~~~
使用脚本构建并进入开发容器
~~~ bash
./init.sh
~~~
---
本开发环境基于容器化,使用Ubuntu22为基本镜像构建.  
也可以使用以下指令手动构建镜像
~~~ bash
cd ./dormTK/env && podman-compose up -d --build
~~~
附加终端到容器
~~~ bash
podman exec -it dormTK-dev zsh 
~~~
