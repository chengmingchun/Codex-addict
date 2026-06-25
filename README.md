# Intra Codex

Intra Codex 是一个面向公司内网的本地 Agent Workbench。它不自己实现模型能力，而是把已经安装在电脑或内网环境里的 agent CLI 做成能力池，再提供类似 Codex 的上层交互：项目、会话、Skills、上下文文件、并发调度和用量看板。

## 目标

- 用一个桌面窗口统一调用 `opencode`、`cac`、`codeagent` 等本地 CLI agent。
- 按项目文件夹管理会话，保留历史、运行状态和输出流。
- 像 Codex 一样选择 Skills，把 Markdown 工作流片段注入到 agent prompt。
- 在文件树中勾选上下文文件，由后端安全读取、裁剪并打包进任务。
- 支持浅色/深色主题，默认浅色。
- 输出单个 Windows exe，方便内网分发。

## 当前能力

- Tauri + React 桌面应用。
- 可配置 Agent Provider：命令、参数、并发、prompt 传入方式。
- `inputMode` 支持：
  - `arg`：把 prompt 作为最后一个位置参数传入，适合 `opencode run --format json <message>`。
  - `stdin`：把 prompt 写入标准输入，适合内部 wrapper。
  - `none`：只启动命令，不传 prompt。
- 项目文件树选择器。
- 安全上下文打包：
  - 最多 12 个文件。
  - 单文件最多 24 KB。
  - 总上下文最多 96 KB。
  - 路径必须位于项目根目录内。
- CLI stdout / stderr 实时回传到会话。
- 全局并发和 provider 级并发调度。

## 配置

默认配置在 `config/providers.yaml`：

```yaml
providers:
  - id: opencode
    label: OpenCode
    command: opencode
    args:
      - run
      - "--format"
      - json
    concurrency: 2
    cwd: "."
    inputMode: arg
    shell: true
    env: {}
defaults:
  providerId: opencode
  maxGlobalConcurrency: 3
  workspaceRoot: ".."
  skillsRoot: "skills"
```

应用右侧 Agent 面板也可以直接修改命令、参数、并发和 prompt 传入方式，并保存回 `providers.yaml`。

## 开发

```bash
npm install
npm run dev
```

## 构建 Windows exe

```bash
npm run dist:win
```

构建输出：

```text
src-tauri/target/release/intra-codex.exe
```

本仓库也会把最新可运行 exe 复制到：

```text
artifacts/intra-codex.exe
```

## 项目结构

```text
src/                 React UI
src-tauri/src/       Rust runtime, scheduler, CLI bridge
config/              Agent provider 配置
skills/              Markdown skills
docs/                设计文档
artifacts/           可分发 exe 产物
```
