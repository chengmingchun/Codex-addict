# Intra Codex (Polished)

Intra Codex 是一个面向公司内网的 **Agent Workbench（类 Codex 桌面工作台）**，用于把分散的 CLI Agent 能力（opencode / cac / codeagent 等）统一编排成“项目 + 会话 + Skills + 上下文”的持续工作流。

---

## ✨ 这版增强点（Polished）

这一版主要强化三件事：

### 🧠 1. 上下文工程（Context Engineering Ready）

核心升级点：从“文件展示”升级为“上下文输入源”。

目标链路：

```
File Tree → Select Files → Read Content → Pack Context → Inject Prompt → CLI Agent
```

能力规划：

- ✔ 支持文件级上下文选择
- ✔ 支持目录级展开/过滤
- ✔ token budget 控制（防止 prompt 爆炸）
- ✔ ignore/include 规则（类似 .gitignore 思路）

---

### 📦 2. Skills 工程化

Skills 从“Markdown 片段”升级为“可组合工作流单元”。

未来能力：

- Skill chaining（技能链）
- Skill 参数化（可配置 prompt 变量）
- 团队级 Skill 共享库

---

### 🧩 3. 从 CLI Launcher → Agent Workbench

系统定位升级：

| 阶段 | 定位 |
|------|------|
| v0 | CLI 启动器 |
| v1 | Agent 面板 |
| v2 | Agent Workbench |

当前已接近 v2。

---

## 🧠 核心能力

- 项目级会话管理（Project / Session）
- 多 Agent Provider（opencode / cac / codeagent）
- Skills 注入机制（Markdown 工作流）
- 文件树上下文展示（升级为上下文源）
- CLI stdout / stderr 实时流式回传
- 并发调度器（全局 + Provider 级）
- 本地持久化（state.json）
- Tauri 桌面应用（单 exe）

---

## 🚀 Roadmap（更清晰版本）

### Phase 1：Context Pipeline（核心）

- [ ] file selector → prompt packer
- [ ] context compression
- [ ] token estimation / trimming

### Phase 2：Structured Output

- [ ] CLI JSON parsing
- [ ] diff / result / error 分离展示
- [ ] UI diff viewer

### Phase 3：Skill System 2.0

- [ ] skill graph
- [ ] parameterized skill
- [ ] skill marketplace

---

## 🧱 架构

```
React UI
   ↓
Tauri IPC
   ↓
Rust Orchestrator
   ↓
CLI Agents
```

---

## 🎯 项目定位

> 不是 CLI 工具的 UI wrapper，而是“面向代码执行的 Agent 操作系统”。

---

下一步关键不是 UI，而是：

👉 **Context Engineering（上下文工程）**
