# Excalidraw DSL 教程

欢迎使用 Excalidraw DSL 教程！本指南将教您如何使用简单的文本领域特定语言（DSL）创建精美的图表，该语言可编译为 Excalidraw JSON 格式。

## 目录

1. [简介](#简介)
2. [安装](#安装)
3. [基础语法](#基础语法)
4. [创建节点](#创建节点)
5. [使用边连接节点](#使用边连接节点)
6. [使用容器](#使用容器)
7. [使用分组](#使用分组)
8. [样式设置](#样式设置)
9. [高级功能](#高级功能)
10. [命令行使用](#命令行使用)
11. [示例](#示例)

## 简介

Excalidraw DSL 是一种基于文本的图表创建语言。您无需手动绘制形状，只需使用简单的文本命令描述图表，工具就会自动为您生成精美的 Excalidraw 图表。

### 为什么使用 Excalidraw DSL？

- **版本控制友好**：文本文件与 Git 完美配合
- **创建速度快**：打字比绘图更快
- **布局一致**：自动布局算法确保图表整洁
- **可重用组件**：定义模板并重复使用
- **程序化生成**：从数据生成图表

## 安装

首先，确保已安装 Rust。如果没有，请从 [rustup.rs](https://rustup.rs/) 安装。

然后克隆并构建项目：

```bash
git clone https://github.com/yourusername/excalidraw-dsl
cd excalidraw-dsl
cargo build --release
```

二进制文件将位于 `target/release/edsl`。

## 基础语法

DSL 使用简单、直观的语法。基本结构如下：

```
# 注释以 # 开头

# 定义节点
节点ID "节点标签"

# 连接节点
节点1 -> 节点2

# 基础知识就这么简单！
```

### 文件结构

典型的 `.edsl` 文件结构如下：

```yaml
---
layout: dagre  # 可选配置
---

# 您的图表定义在这里
```

## 创建节点

节点是图表的基本构建块。每个节点都有一个 ID 和可选的标签。

### 简单节点

```
# 带标签的节点
start "开始"

# 不带标签的节点（ID 将用作标签）
process

# 多个节点
input "用户输入"
process "处理数据"
output "显示结果"
```

### 节点 ID

- 必须以字母开头
- 可以包含字母、数字和下划线
- 区分大小写
- 必须唯一

示例：
```
user_input "用户输入"
step1 "第一步"
dataStore "数据库"
API_endpoint "REST API"
```

## 使用边连接节点

边将节点连接在一起。Excalidraw DSL 支持各种箭头类型和样式。

### 基本连接

```
# 简单箭头
start -> process

# 带标签的箭头
process -> output "结果"

# 链式连接
input -> process -> output
```

### 箭头类型

```
# 单向箭头（默认）
a -> b

# 双向箭头
a <-> b

# 无箭头（仅线条）
a --- b
```

### 边链

您可以在一行中创建多个连接：

```
# 创建：a->b, b->c, c->d
a -> b -> c -> d

# 带标签
start -> process "步骤1" -> validate "检查" -> end
```

## 使用容器

容器将相关节点在视觉上分组在一起。

### 基本容器

```
container {
    node1 "第一个节点"
    node2 "第二个节点"
    node1 -> node2
}
```

### 命名容器

```
container backend "后端服务" {
    api "API 服务器"
    db "数据库"
    cache "Redis 缓存"

    api -> db
    api -> cache
}
```

### 嵌套容器

```
container system "系统架构" {
    container frontend "前端" {
        ui "React 应用"
        state "Redux 存储"
    }

    container backend "后端" {
        api "API 服务器"
        db "PostgreSQL"
    }

    # 跨容器连接
    ui -> api
}
```

## 使用分组

分组是可以一起设置样式的节点的逻辑集合。

### 基本分组

```
group team {
    alice "小红"
    bob "小明"
    charlie "小张"
}

# 将组成员连接到外部节点
alice -> task1
bob -> task2
```

### 语义分组

分组可以具有语义含义：

```
group services {
    auth "认证服务"
    payment "支付服务"
    notification "通知服务"
}

group databases {
    userDB "用户数据库"
    orderDB "订单数据库"
}

# 在组之间连接
auth -> userDB
payment -> orderDB
```

## 样式设置

使用样式属性自定义图表的外观。

### 节点样式

```
# 彩色节点
server "Web 服务器" {
    backgroundColor: "#ff6b6b"
    strokeColor: "#c92a2a"
    textColor: "#ffffff"
}

# 圆角
database "PostgreSQL" {
    roundness: 3
}
```

### 边样式

```
# 虚线连接
client -> server {
    strokeStyle: "dashed"
    strokeColor: "#868e96"
}

# 粗重要连接
server -> database {
    strokeWidth: 3
    strokeColor: "#ff6b6b"
}
```

### 全局样式

在配置中设置默认样式：

```yaml
---
layout: dagre
font: "Cascadia"
strokeWidth: 2
background_color: "#ffffff"
---
```

### 可用样式属性

**节点属性：**
- `backgroundColor`：十六进制颜色（例如 "#ff6b6b"）
- `strokeColor`：边框颜色
- `strokeWidth`：边框粗细（1-4）
- `textColor`：文本颜色
- `roughness`：手绘效果（0-2）
- `roundness`：圆角程度（0-3）
- `font`：字体系列（"Virgil"、"Helvetica"、"Cascadia"）

**边属性：**
- `strokeColor`：线条颜色
- `strokeWidth`：线条粗细
- `strokeStyle`："solid"、"dashed" 或 "dotted"
- `startArrowhead`："triangle"、"dot"、"diamond"、"none"
- `endArrowhead`："triangle"、"dot"、"diamond"、"none"

## 高级功能

### 模板

定义可重用的组件模板：

```yaml
---
templates:
  microservice:
    api: "$name API"
    db: "$name 数据库"
    cache: "$name 缓存"
    edges:
      - api -> db
      - api -> cache
---

# 使用模板
microservice user_service {
    name: "用户"
}

microservice order_service {
    name: "订单"
}

# 连接服务
user_service.api -> order_service.api
```

### 组件类型

定义自定义组件类型：

```yaml
---
component_types:
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1976d2"
    roundness: 2

  service:
    backgroundColor: "#f3e5f5"
    strokeColor: "#7b1fa2"
---

# 使用组件类型
auth "认证服务" @service
userDB "用户数据库" @database

auth -> userDB
```

### 布局算法

选择不同的布局算法：

```yaml
---
layout: force  # 选项：dagre、force、elk、manual
layout_options:
  rankdir: "TB"  # 从上到下（或 LR、RL、BT）
  nodesep: 100   # 节点间距
  ranksep: 150   # 层级间距
---
```

### 边路由

控制边的绘制方式：

```
# 正交路由（直角）
a -> b @orthogonal

# 曲线路由
c -> d @curved

# 直线（默认）
e -> f @straight
```

## 命令行使用

### 基本编译

```bash
# 编译为 Excalidraw JSON
edsl input.edsl -o output.excalidraw

# 使用特定布局
edsl input.edsl -o output.excalidraw --layout elk
```

### 监视模式

文件更改时自动重新编译：

```bash
edsl input.edsl -o output.excalidraw --watch
```

### Web 服务器模式

启动 Web 服务器进行实时预览：

```bash
edsl --server
# 在 http://localhost:3030 打开
```

### 验证

仅检查语法而不生成输出：

```bash
edsl input.edsl --validate
```

### CLI 选项

```
用法：edsl [选项] [输入]

参数：
  [输入]  输入 .edsl 文件

选项：
  -o, --output <输出>             输出文件路径
  -l, --layout <布局>             布局算法 [默认：dagre]
  -w, --watch                     监视文件更改
  -s, --server                    启动 Web 服务器
  -p, --port <端口>               服务器端口 [默认：3030]
  -v, --validate                  仅验证
  -h, --help                      打印帮助
  -V, --version                   打印版本
```

## 示例

### 示例 1：简单流程图

```
# 简单的认证流程
start "用户登录"
auth "认证"
success "仪表板"
failure "错误页面"

start -> auth
auth -> success "有效"
auth -> failure "无效"
```

### 示例 2：系统架构

```yaml
---
layout: dagre
font: "Helvetica"
---

container frontend "前端层" {
    web "Web 应用"
    mobile "移动应用"
}

container backend "后端层" {
    api "API 网关"
    auth "认证服务"
    users "用户服务"

    api -> auth
    api -> users
}

container data "数据层" {
    postgres "PostgreSQL"
    redis "Redis 缓存"
    s3 "S3 存储"
}

# 连接层
web -> api
mobile -> api
auth -> postgres
auth -> redis
users -> postgres
users -> s3
```

### 示例 3：带样式的微服务

```yaml
---
component_types:
  service:
    backgroundColor: "#e8f5e9"
    strokeColor: "#2e7d32"
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1565c0"
    roundness: 2
---

# 服务
auth "认证服务" @service
user "用户服务" @service
order "订单服务" @service
payment "支付服务" @service

# 数据库
authDB "认证数据库" @database
userDB "用户数据库" @database
orderDB "订单数据库" @database

# 消息队列
queue "RabbitMQ" {
    backgroundColor: "#fff3e0"
    strokeColor: "#e65100"
}

# 连接
auth -> authDB
user -> userDB
order -> orderDB

# 服务通信
user -> order "获取订单"
order -> payment "处理支付"
payment -> queue "支付事件"
auth -> queue "登录事件"
```

### 示例 4：状态机

```
# 订单处理状态机
initial "新订单"
pending "待支付"
paid "已支付"
processing "处理中"
shipped "已发货"
delivered "已送达"
cancelled "已取消"

initial -> pending "提交"
pending -> paid "支付成功"
pending -> cancelled "支付失败"
paid -> processing "开始履行"
processing -> shipped "发货"
shipped -> delivered "确认送达"

# 可从多个状态取消
paid -> cancelled "取消"
processing -> cancelled "取消"
```

### 示例 5：网络拓扑

```yaml
---
layout: force
---

container cloud "云基础设施" {
    lb "负载均衡器"

    container servers "应用服务器" {
        app1 "应用服务器 1"
        app2 "应用服务器 2"
        app3 "应用服务器 3"
    }

    lb -> app1
    lb -> app2
    lb -> app3
}

container onprem "本地部署" {
    corp "企业网络"
    vpn "VPN 网关"
}

# 外部连接
internet "互联网"
users "用户"

users -> internet
internet -> lb
corp -> vpn
vpn -> lb "安全连接" {
    strokeStyle: "dashed"
    strokeColor: "#f03e3e"
}
```

## 技巧和最佳实践

1. **使用有意义的 ID**：选择描述性的 ID，使 DSL 可读
2. **使用容器组织**：将相关节点分组到容器中
3. **一致的样式**：定义组件类型以获得一致的外观
4. **注释您的图表**：使用注释解释复杂的关系
5. **从简单开始**：从节点和边开始，稍后添加样式
6. **使用模板**：对于重复的模式，创建模板
7. **版本控制**：将 .edsl 文件保存在 Git 中
8. **监视模式**：在开发期间使用监视模式以获得即时反馈

## 下一步

- 在 `examples/` 目录中探索更多示例
- 阅读[语言参考](../docs/language-reference.md)以了解完整语法
- 加入我们的社区以获得支持和分享图表
- 在 GitHub 上为项目做出贡献

祝您绘图愉快！
