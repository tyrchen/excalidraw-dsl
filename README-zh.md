# Excalidraw DSL

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/excalidraw-dsl.svg)](https://crates.io/crates/excalidraw-dsl)

一个强大的领域特定语言（DSL），用于通过文本生成 [Excalidraw](https://excalidraw.com/) 图表。将图表编写为代码，获得精美的手绘风格可视化效果。

[English](./README.md) | [教程](./tutorial/README-zh.md) | [示例](./examples/)

## ✨ 特性

- 📝 **简单的文本语法** - 使用直观的文本命令编写图表
- 🎨 **自动布局** - 多种布局算法（Dagre、Force、ELK）
- 🎯 **智能样式** - 使用组件类型和主题实现一致的样式
- 📦 **容器和分组** - 使用层次结构组织复杂图表
- 🔄 **实时预览** - 内置 Web 服务器，支持实时更新
- 🚀 **快速编译** - 即时生成图表
- 🎭 **手绘风格** - 精美的 Excalidraw 美学效果
- 🌈 **完全样式控制** - 颜色、字体、线条样式等

## 🚀 快速开始

### 安装

```bash
# 从源码安装
git clone https://github.com/yourusername/excalidraw-dsl
cd excalidraw-dsl
cargo install --path .

# 或从 crates.io 安装（发布后）
cargo install excalidraw-dsl
```

### 您的第一个图表

创建文件 `hello.edsl`：

```
start "你好"
world "世界"
start -> world
```

编译它：

```bash
edsl hello.edsl -o hello.excalidraw
```

在 [Excalidraw](https://excalidraw.com/) 中打开 `hello.excalidraw` 查看您的图表！

## 📖 语言概览

### 基础语法

```edsl
# 注释以 # 开头

# 节点
node_id "节点标签"

# 边
source -> target
source -> target "边标签"

# 容器
container name "容器标签" {
    node1 "节点 1"
    node2 "节点 2"
    node1 -> node2
}

# 样式
styled_node "带样式的节点" {
    backgroundColor: "#ff6b6b"
    textColor: "#ffffff"
}
```

### 高级特性

#### 组件类型

定义可重用的样式：

```yaml
---
component_types:
  service:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1976d2"
  database:
    backgroundColor: "#fce4ec"
    strokeColor: "#c2185b"
---

auth "认证服务" @service
userDB "用户数据库" @database
auth -> userDB
```

#### 模板

创建可重用的组件：

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

microservice user_service {
    name: "用户"
}
```

#### 布局算法

从多个布局引擎中选择：

```yaml
---
layout: dagre  # 选项：dagre、force、elk
layout_options:
  rankdir: "TB"  # 从上到下，LR、RL、BT
  nodesep: 100
  ranksep: 150
---
```

## 🎯 实际案例

```yaml
---
layout: dagre
component_types:
  service:
    backgroundColor: "#e8f5e9"
    strokeColor: "#2e7d32"
  database:
    backgroundColor: "#e3f2fd"
    strokeColor: "#1565c0"
    roundness: 2
---

# 微服务架构
gateway "API 网关" @service

container services "微服务" {
    auth "认证服务" @service
    user "用户服务" @service
    order "订单服务" @service
    payment "支付服务" @service
}

container databases "数据库" {
    authDB "认证数据库" @database
    userDB "用户数据库" @database
    orderDB "订单数据库" @database
}

queue "消息队列" {
    backgroundColor: "#fff3e0"
    strokeColor: "#e65100"
}

# 连接
gateway -> auth
gateway -> user
gateway -> order

auth -> authDB
user -> userDB
order -> orderDB

order -> payment "处理支付"
payment -> queue "支付事件"
```

## 🛠️ CLI 使用

```bash
# 基本编译
edsl input.edsl -o output.excalidraw

# 监视模式 - 更改时自动重新编译
edsl input.edsl -o output.excalidraw --watch

# 启动 Web 服务器进行实时预览
edsl --server
# 访问 http://localhost:3030

# 仅验证语法而不输出
edsl input.edsl --validate

# 使用特定的布局算法
edsl input.edsl -o output.excalidraw --layout elk
```

### 所有选项

```
用法：edsl [选项] [输入]

参数：
  [输入]  输入 .edsl 文件

选项：
  -o, --output <输出>         输出文件路径
  -l, --layout <布局>         布局算法 [默认：dagre]
                             可能的值：dagre、force、elk
  -w, --watch                监视文件更改
  -s, --server               启动 Web 服务器
  -p, --port <端口>          服务器端口 [默认：3030]
  -v, --validate             仅验证
      --watch-delay <毫秒>    重新编译前的延迟 [默认：100]
  -h, --help                 打印帮助
  -V, --version              打印版本
```

## 📚 文档

- 📖 **[教程](./tutorial/README-zh.md)** - 面向初学者的分步指南
- 🌏 **[English Tutorial](./tutorial/README-en.md)** - 英文教程
- 📝 **[语言参考](./docs/language-reference.md)** - 完整的语法参考
- 🎨 **[示例](./examples/)** - 示例图表和模式
- 🏗️ **[架构](./docs/architecture.md)** - 技术文档

## 🧩 示例

查看[示例目录](./examples/)以获取更复杂的图表：

- [微服务架构](./examples/microservices.edsl)
- [状态机](./examples/state-machine.edsl)
- [网络拓扑](./examples/network.edsl)
- [系统架构](./examples/system-architecture.edsl)
- [流程图](./examples/flowchart.edsl)

## 🤝 贡献

我们欢迎贡献！请参阅我们的[贡献指南](CONTRIBUTING.md)了解详情。

### 开发设置

```bash
# 克隆仓库
git clone https://github.com/yourusername/excalidraw-dsl
cd excalidraw-dsl

# 构建项目
cargo build

# 运行测试
cargo test

# 使用示例运行
cargo run -- examples/basic.edsl -o output.excalidraw
```

### 项目结构

```
excalidraw-dsl/
├── src/
│   ├── ast.rs          # 抽象语法树定义
│   ├── parser.rs       # 基于 Pest 的解析器
│   ├── igr.rs          # 中间图形表示
│   ├── layout/         # 布局算法
│   ├── generator.rs    # Excalidraw JSON 生成器
│   └── main.rs         # CLI 入口点
├── grammar/
│   └── edsl.pest       # 语法定义
├── examples/           # 示例图表
├── tests/             # 集成测试
└── tutorial/          # 教程和文档
```

## 🚦 路线图

- [ ] **VSCode 扩展** - 语法高亮和实时预览
- [ ] **更多布局** - 分层、圆形和自定义布局
- [ ] **主题** - 内置颜色主题
- [ ] **导出格式** - SVG、PNG、PDF 导出
- [ ] **交互模式** - 图表创建的 REPL
- [ ] **Web 游乐场** - 在线编辑器和编译器
- [ ] **图表库** - 可重用的图表组件
- [ ] **AI 集成** - 从描述生成图表

## 📄 许可证

本项目根据 MIT 许可证授权 - 有关详细信息，请参阅 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [Excalidraw](https://excalidraw.com/) - 提供了出色的绘图工具
- [Graphviz](https://graphviz.org/) - DSL 设计的灵感来源
- [Mermaid](https://mermaid-js.github.io/) - 图表语法的想法
- [Pest](https://pest.rs/) - 优秀的解析器生成器

## 💬 社区

- 🐛 **[问题跟踪](https://github.com/yourusername/excalidraw-dsl/issues)** - 报告错误或请求功能
- 💬 **[讨论](https://github.com/yourusername/excalidraw-dsl/discussions)** - 提问和分享想法
- 🐦 **[Twitter](https://twitter.com/excalidraw_dsl)** - 关注更新

---

由 Excalidraw DSL 社区用 ❤️ 制作
