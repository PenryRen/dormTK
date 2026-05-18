# 一.项目介绍
(占位符,以后再写喵)

# 二.项目结构
~~~
── README.md ----------------------介绍文档
├── app
│   ├── Cargo.lock
│   ├── Cargo.toml ------------rust配置文件
│   └── src -----------------------代码目录
└── env
    ├── PostgreSQL ----------PostgreSQL容器
    ├── dev --------------------开发环境容器
    ├── init.sh -----------开发环境初始化脚本
    └── podman-compose.yml -----容器编排文件
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
~~~
git clone https://github.com/PenryRen/dormTK.git
~~~
使用git将仓库拉取到本地

2.初始化开发环境
---
切换到开发环境目录
~~~
cd ./dormTK/env
~~~
使用脚本构建并进入开发容器
~~~
./init.sh
~~~
本开发环境基于容器化,使用Ubuntu22为基本镜像构建.  
也可以使用以下指令手动构建镜像
~~~
cd ./dormTK/env && podman-compose up -d --build
~~~
附加终端到容器
~~~
podman exec -it dormTK-dev zsh 
~~~
