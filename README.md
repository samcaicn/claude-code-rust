# Claude Code Rust (Custom Provider)

支持自定义 API 提供者的 Claude Code Rust 终端界面。基于 [srothgan/claude-code-rust](https://github.com/srothgan/claude-code-rust) 开发。

## 支持的 API 提供者

- **admin.tuptup.top/api** - 自定义 API 代理服务
- **custom** - 通用自定义提供者

## 功能特性

- 原生 Rust 编写的终端界面，性能优异
- 支持自定义 API 端点和认证
- 自动设备注册和 token 管理
- 基于 [Ratatui](https://ratatui.rs/) 的 TUI

## 安装

### 从 Release 下载

```bash
# macOS arm64
curl -fL https://github.com/samcaicn/gloai/releases/latest/download/claude-rs-aarch64-apple-darwin -o claude-rs
chmod +x claude-rs

# macOS x86_64
curl -fL https://github.com/samcaicn/gloai/releases/latest/download/claude-rs-x86_64-apple-darwin -o claude-rs
chmod +x claude-rs
```

### 从源码构建

```bash
git clone https://github.com/samcaicn/gloai.git
cd gloai
git checkout claude
cargo build --release --bin claude-rs
```

## 配置

### config.toml

在可执行文件同目录下创建 `config.toml`：

```toml
# API 配置
api_key = "YOUR_API_KEY"
base_url = "https://admin.tuptup.top/api"
model = "doubao-seed-2-0-lite-260215"
provider = "custom"
timeout_sec = 120

# 设备 UUID（自动生成）
device.uuid = ""
```

### 环境变量

| 变量 | 说明 |
|------|------|
| `ANTHROPIC_AUTH_TOKEN` | API 认证令牌 |
| `ANTHROPIC_BASE_URL` | API 基础 URL |
| `ANTHROPIC_DEFAULT_MODEL` | 默认模型 |
| `CLAUDE_RS_CONFIG` | 配置文件路径 |
| `CLAUDE_RS_PROVIDER` | 提供者类型 |

## 使用

```bash
# 首次运行会自动注册设备
./claude-rs

# 指定配置文件
CLAUDE_RS_CONFIG=/path/to/config.toml ./claude-rs
```

## 工作原理

```
┌─────────────────────────────────────┐
│         claude-rs (Rust)           │
│  ┌───────────────────────────────┐  │
│  │       device.rs              │  │
│  │  - 获取硬件指纹               │  │
│  │  - 注册设备获取 token         │  │
│  │  - 读取/保存配置文件          │  │
│  └───────────────────────────────┘  │
│                ↓                    │
│  ┌───────────────────────────────┐  │
│  │     Ratatui TUI              │  │
│  │  - 终端界面交互               │  │
│  │  - 命令执行                   │  │
│  └───────────────────────────────┘  │
│                ↓                    │
│  ┌───────────────────────────────┐  │
│  │    Agent SDK Bridge          │  │
│  │  (agent-sdk/dist/bridge.js)  │  │
│  └───────────────────────────────┘  │
│                ↓                    │
│  ┌───────────────────────────────┐  │
│  │    @anthropic-ai/sdk         │  │
│  │  读取环境变量配置             │  │
│  └───────────────────────────────┘  │
│                ↓                    │
│  ┌───────────────────────────────┐  │
│  │    API Provider              │  │
│  │  admin.tuptup.top/api        │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

## 构建

### 本地构建

```bash
cargo build --release --bin claude-rs
```

### CI 构建

推送到 `claude` 分支自动触发 GitHub Actions 构建。

## License

Apache-2.0
