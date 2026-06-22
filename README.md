# Intra Codex

Intra Codex 是一个面向公司内网的 Codex-like Agent Workbench。它把本地或内网已有的 Agent CLI 能力组织成「项目 / 会话 / Skills / Agent Pool / 用量看板」的桌面交互工具，让用户可以像使用 Codex 一样围绕项目文件夹持续对话，而底层执行仍由已配置的 CLI 完成。

## 核心能力

- **项目会话管理**：按本地文件夹打开项目，每个项目下维护多个会话。
- **Agent 能力池**：通过配置接入 `opencode`、`cac`、`codeagent` 等 CLI。
- **Skills 装配**：从 Markdown 目录加载团队工作流，在会话运行时注入 prompt。
- **文件上下文**：展示当前项目文件树，帮助人类理解会话所处上下文。
- **流式运行反馈**：CLI 的 stdout/stderr 会进入当前会话消息流。
- **并发控制**：支持全局并发和每个 Agent 的独立并发限制。
- **本地持久化**：项目、会话、运行记录保存到本机应用数据目录。
- **Tauri 单 exe**：Windows 下可直接编译为单个 `intra-codex.exe`。

## 技术栈

- Frontend: React 19 + Vite + TypeScript
- Desktop: Tauri 2
- Backend: Rust command handlers
- UI Icons: lucide-react
- Config: YAML

## 快速开始

### 环境要求

- Node.js
- npm
- Rust / Cargo
- Windows WebView2 Runtime
- 已安装并可在命令行访问的 Agent CLI，例如 `opencode`

### 安装依赖

```powershell
npm install
```

### 开发运行

```powershell
npm run dev
```

### 类型检查

```powershell
npm run typecheck
cargo check --manifest-path src-tauri/Cargo.toml
```

### 编译 Windows exe

```powershell
npm run dist:win
```

生成位置：

```text
src-tauri/target/release/intra-codex.exe
```

## Agent 配置

Agent Pool 由 `config/providers.yaml` 配置。示例：

```yaml
providers:
  - id: opencode
    label: OpenCode
    command: opencode
    args:
      - run
      - "--json"
    concurrency: 2
    cwd: "."
    shell: true
    capabilities:
      - repo-search
      - code-edit
      - json-output
    env: {}

defaults:
  providerId: opencode
  maxGlobalConcurrency: 3
  workspaceRoot: ".."
  skillsRoot: "skills"
```

字段说明：

| 字段 | 说明 |
| --- | --- |
| `id` | Agent 唯一标识 |
| `label` | UI 显示名称 |
| `command` | CLI 命令 |
| `args` | CLI 参数 |
| `concurrency` | 该 Agent 最大并发 |
| `shell` | Windows wrapper 类命令通常建议设为 `true` |
| `capabilities` | UI 展示的能力标签 |
| `env` | 注入给 CLI 的环境变量 |

## Skills

默认从 `skills` 目录加载 Markdown 文件。每个 `.md` 文件会成为一个 Skill，一级标题作为显示名称。

示例：

```markdown
# Code Review

用于代码审查任务。优先关注 bug、回归风险、遗漏测试和可维护性问题。
```

运行时也可以在右侧 Skills 面板选择自定义 Skills 目录。

## 使用方式

1. 点击左侧「打开项目」，选择一个本地代码目录。
2. 在项目下新建或选择会话。
3. 在右侧选择 Agent，例如 `OpenCode`。
4. 勾选需要注入的 Skills。
5. 在底部输入框描述任务，点击发送。
6. Agent CLI 的执行输出会进入当前会话。

## 项目结构

```text
intra-codex/
├─ config/              # Agent 配置
├─ skills/              # 默认 Skills 目录
├─ src/                 # React 前端
├─ src-tauri/           # Tauri / Rust 桌面后端
├─ index.html
├─ package.json
└─ README.md
```

## 说明

这个项目的目标不是替代 `opencode`、`cac` 或 `codeagent`，而是把这些本地/内网 Agent CLI 汇聚成一个更适合人类交互的上层工作台。

