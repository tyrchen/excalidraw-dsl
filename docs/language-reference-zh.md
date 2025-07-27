# Excalidraw DSL 语言参考

本文档提供 Excalidraw DSL 语法和功能的完整参考。

[English Version](./language-reference.md)

## 目录

1. [文件结构](#文件结构)
2. [注释](#注释)
3. [节点](#节点)
4. [边](#边)
5. [容器](#容器)
6. [分组](#分组)
7. [样式](#样式)
8. [组件类型](#组件类型)
9. [模板](#模板)
10. [布局配置](#布局配置)
11. [属性参考](#属性参考)
12. [示例](#示例)

## 文件结构

Excalidraw DSL 文件由两个主要部分组成：

```yaml
---
# 可选的 YAML 前置内容，用于配置
layout: dagre
font: "Cascadia"
---

# DSL 内容在这里
node1 "节点 1"
node2 "节点 2"
node1 -> node2
```

### 前置内容

YAML 前置内容部分是可选的，用 `---` 标记包围。它可以包含：

- 全局配置选项
- 组件类型定义
- 模板定义
- 布局设置

## 注释

注释以 `#` 开始，一直延续到行尾：

```edsl
# 这是一个注释
node1 "节点 1"  # 这也是一个注释
```

## 节点

### 基本节点语法

```edsl
# 仅有 ID 的节点（ID 将用作标签）
node_id

# 带标签的节点
node_id "节点标签"

# 带属性的节点
node_id "节点标签" {
    backgroundColor: "#ff6b6b"
    strokeColor: "#c92a2a"
}

# 带组件类型的节点
node_id "节点标签" @service
```

### 节点 ID 规则

- 必须以字母或下划线开头
- 可以包含字母、数字和下划线
- 区分大小写
- 在图表中必须唯一

### 有效示例

```edsl
user_service
auth2
_internal_node
Service123
```

## 边

### 基本边语法

```edsl
# 简单边
source -> target

# 带标签的边
source -> target "边标签"

# 带属性的边
source -> target {
    strokeStyle: "dashed"
    strokeColor: "#868e96"
}

# 带标签和属性的边
source -> target "标签" {
    strokeWidth: 2
}
```

### 箭头类型

```edsl
# 单向箭头（默认）
a -> b

# 双向箭头
a <-> b

# 无箭头（仅线条）
a --- b
```

### 边链

在一个语句中创建多条边：

```edsl
# 创建 a->b, b->c, c->d
a -> b -> c -> d

# 带标签
start -> process "步骤 1" -> validate "检查" -> end

# 混合箭头类型
a -> b <-> c --- d
```

### 边路由

控制边的绘制方式：

```edsl
# 直线（默认）
a -> b

# 正交（直角）
a -> b @orthogonal

# 曲线
a -> b @curved
```

## 容器

容器在视觉和逻辑上对节点进行分组。

### 基本容器语法

```edsl
# 匿名容器
container {
    node1 "节点 1"
    node2 "节点 2"
}

# 带标签的命名容器
container backend "后端服务" {
    api "API 服务器"
    db "数据库"
}

# 带 ID 和标签的容器
container backend_container "后端服务" {
    # 内容
}
```

### 容器属性

```edsl
container services "服务" {
    backgroundColor: "#f8f9fa"
    strokeStyle: "dashed"

    service1 "服务 1"
    service2 "服务 2"
}
```

### 嵌套容器

```edsl
container system "系统" {
    container frontend "前端" {
        ui "用户界面"
        state "状态管理"
    }

    container backend "后端" {
        api "API"
        db "数据库"
    }
}
```

### 容器引用

从外部引用容器内的节点：

```edsl
container backend "后端" {
    api "API"
    db "数据库"
}

frontend "前端"

# 引用容器节点
frontend -> backend.api
```

## 分组

分组提供逻辑组织，但没有视觉边界。

### 基本分组语法

```edsl
# 基本分组
group team {
    alice "小红"
    bob "小明"
}

# 带标签的分组
group developers "开发团队" {
    frontend_dev "前端开发"
    backend_dev "后端开发"
}
```

### 分组类型

```edsl
# 基本分组
group basic_group {
    node1 "节点 1"
}

# 带类型的语义分组
group services:microservices {
    auth "认证服务"
    user "用户服务"
}
```

### 嵌套分组

```edsl
group department {
    group frontend_team {
        alice "小红"
        bob "小明"
    }

    group backend_team {
        charlie "小张"
        david "小李"
    }
}
```

## 样式

### 内联样式

直接对元素应用样式：

```edsl
# 节点样式
server "Web 服务器" {
    backgroundColor: "#ff6b6b"
    strokeColor: "#c92a2a"
    strokeWidth: 2
    textColor: "#ffffff"
    roughness: 1
    roundness: 2
    font: "Cascadia"
}

# 边样式
client -> server {
    strokeStyle: "dashed"
    strokeColor: "#868e96"
    strokeWidth: 2
    startArrowhead: "dot"
    endArrowhead: "triangle"
}

# 容器样式
container backend "后端" {
    backgroundColor: "#f8f9fa"
    strokeStyle: "dashed"

    # 内容
}
```

### 全局样式

在前置内容中设置默认样式：

```yaml
---
# 全局默认值
font: "Virgil"
strokeWidth: 2
roughness: 1
backgroundColor: "#ffffff"
---
```

## 组件类型

定义可重用的样式集：

```yaml
---
component_types:
  service:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1976d2"
    roundness: 2

  database:
    backgroundColor: "#fce4ec"
    strokeColor: "#c2185b"
    roundness: 1

  queue:
    backgroundColor: "#f3e5f5"
    strokeColor: "#7b1fa2"
---

# 使用组件类型
auth_service "认证服务" @service
user_db "用户数据库" @database
message_queue "消息队列" @queue
```

## 模板

创建可重用的组件结构：

```yaml
---
templates:
  microservice:
    # 模板节点
    api: "$name API"
    db: "$name 数据库"
    cache: "$name 缓存"

    # 模板边
    edges:
      - api -> db
      - api -> cache

  crud_service:
    controller: "$name 控制器"
    service: "$name 服务"
    repository: "$name 仓库"
    model: "$name 模型"

    edges:
      - controller -> service
      - service -> repository
      - repository -> model
---

# 使用模板
microservice user_service {
    name: "用户"
}

crud_service order_service {
    name: "订单"
}

# 引用模板节点
user_service.api -> order_service.controller
```

## 布局配置

### 布局算法

```yaml
---
layout: dagre  # 选项：dagre、force、elk
---
```

#### Dagre（层次布局）

最适合层次结构、流程图和组织图。

```yaml
---
layout: dagre
layout_options:
  rankdir: "TB"    # TB（从上到下）、BT、LR、RL
  nodesep: 50      # 同一层级节点之间的最小间距
  ranksep: 100     # 层级之间的最小间距
  marginx: 20      # 水平边距
  marginy: 20      # 垂直边距
---
```

#### Force（力导向布局）

最适合网络图和有机布局。

```yaml
---
layout: force
layout_options:
  iterations: 300     # 模拟迭代次数
  node_repulsion: 50  # 节点之间的斥力
  link_distance: 100  # 理想边长度
  link_strength: 1    # 边的力强度
---
```

#### ELK（Eclipse 布局内核）

提供多种算法选项的高级布局。

```yaml
---
layout: elk
layout_options:
  algorithm: "layered"  # layered、force、stress、mrtree、radial
  direction: "DOWN"     # UP、DOWN、LEFT、RIGHT
  spacing: 50          # 基础间距值
---
```

## 属性参考

### 节点属性

| 属性 | 类型 | 值 | 描述 |
|------|------|-----|------|
| `backgroundColor` | 颜色 | 十六进制颜色 | 填充颜色 |
| `strokeColor` | 颜色 | 十六进制颜色 | 边框颜色 |
| `strokeWidth` | 数字 | 1-4 | 边框粗细 |
| `strokeStyle` | 字符串 | solid、dashed、dotted | 边框样式 |
| `textColor` | 颜色 | 十六进制颜色 | 文本颜色 |
| `font` | 字符串 | Virgil、Helvetica、Cascadia | 字体系列 |
| `fontSize` | 数字 | 12-48 | 字体大小 |
| `roughness` | 数字 | 0-2 | 手绘效果 |
| `roundness` | 数字 | 0-3 | 圆角程度 |
| `fillStyle` | 字符串 | solid、hachure、cross-hatch | 填充图案 |

### 边属性

| 属性 | 类型 | 值 | 描述 |
|------|------|-----|------|
| `strokeColor` | 颜色 | 十六进制颜色 | 线条颜色 |
| `strokeWidth` | 数字 | 1-4 | 线条粗细 |
| `strokeStyle` | 字符串 | solid、dashed、dotted | 线条样式 |
| `startArrowhead` | 字符串 | none、triangle、dot、diamond | 起始箭头 |
| `endArrowhead` | 字符串 | none、triangle、dot、diamond | 结束箭头 |
| `curvature` | 数字 | 0-1 | 曲线程度（用于曲线边） |

### 容器属性

| 属性 | 类型 | 值 | 描述 |
|------|------|-----|------|
| `backgroundColor` | 颜色 | 十六进制颜色 | 填充颜色 |
| `strokeColor` | 颜色 | 十六进制颜色 | 边框颜色 |
| `strokeWidth` | 数字 | 1-4 | 边框粗细 |
| `strokeStyle` | 字符串 | solid、dashed、dotted | 边框样式 |
| `textColor` | 颜色 | 十六进制颜色 | 标签文本颜色 |
| `font` | 字符串 | Virgil、Helvetica、Cascadia | 字体系列 |
| `padding` | 数字 | 像素 | 内边距 |

### 颜色值

颜色可以指定为：
- 十六进制颜色：`"#ff6b6b"`、`"#000000"`
- RGB：`"rgb(255, 107, 107)"`
- 命名颜色：`"red"`、`"blue"`（有限支持）

## 示例

### 示例 1：简单流程

```edsl
start "开始"
process "处理数据"
decision "有效？"
success "成功"
error "错误"

start -> process
process -> decision
decision -> success "是"
decision -> error "否"
```

### 示例 2：带样式的微服务

```yaml
---
component_types:
  service:
    backgroundColor: "#e8f5e9"
    strokeColor: "#2e7d32"
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1565c0"
---

gateway "API 网关" @service

container services "服务" {
    auth "认证" @service
    user "用户" @service
    order "订单" @service
}

container data "数据层" {
    auth_db "认证数据库" @database
    user_db "用户数据库" @database
    order_db "订单数据库" @database
}

gateway -> services.auth
gateway -> services.user
gateway -> services.order

services.auth -> data.auth_db
services.user -> data.user_db
services.order -> data.order_db
```

### 示例 3：使用模板

```yaml
---
templates:
  layer:
    frontend: "$name 前端"
    backend: "$name 后端"
    database: "$name 数据库"
    edges:
      - frontend -> backend
      - backend -> database
---

layer user_management {
    name: "用户"
}

layer order_management {
    name: "订单"
}

# 跨层连接
user_management.backend -> order_management.backend "API 调用"
```

### 示例 4：复杂样式

```edsl
# 自定义样式的标题
header "系统架构" {
    backgroundColor: "#1a1a1a"
    textColor: "#ffffff"
    fontSize: 24
    roundness: 0
}

# 带样式的容器
container critical "关键服务" {
    backgroundColor: "#ffebee"
    strokeColor: "#d32f2f"
    strokeWidth: 3
    strokeStyle: "solid"

    payment "支付服务" {
        backgroundColor: "#ff5252"
        textColor: "#ffffff"
    }

    fraud "欺诈检测" {
        backgroundColor: "#ff1744"
        textColor: "#ffffff"
    }
}

# 带样式的边
header -> critical "管理" {
    strokeStyle: "dashed"
    strokeColor: "#666666"
    strokeWidth: 2
    startArrowhead: "none"
    endArrowhead: "triangle"
}
```

## 最佳实践

1. **使用有意义的 ID**：选择描述性的 ID，使 DSL 易读
2. **使用容器组织**：将相关节点分组以获得更好的可视化效果
3. **定义组件类型**：在整个图表中创建一致的样式
4. **使用模板**：对于重复的模式，定义模板
5. **注释复杂部分**：添加注释来解释复杂的关系
6. **选择合适的布局**：选择最适合您的图表类型的布局算法

## 语法总结

Excalidraw DSL 的完整 EBNF 语法：

```ebnf
diagram = [front_matter] statement*

front_matter = "---" yaml_content "---"

statement = node_def | edge_def | container_def | group_def | comment

node_def = identifier [string] ["@" identifier] [attributes]

edge_def = edge_chain [string] [attributes]

edge_chain = node_ref (edge_op node_ref)*

edge_op = "->" | "<->" | "---"

container_def = "container" [identifier] [string] "{" statement* "}"

group_def = "group" identifier [":" identifier] [string] "{" statement* "}"

attributes = "{" (attribute_pair)* "}"

attribute_pair = identifier ":" value

node_ref = identifier | qualified_ref

qualified_ref = identifier "." identifier
```
