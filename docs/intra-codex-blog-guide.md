# 我们为什么做了一个内网版 Agent Workbench

> 当 CLI Agent 已经能写代码、读仓库、跑命令之后，真正影响使用体验的，往往不再是模型能力，而是：上下文怎么组织、过程怎么保留、规范怎么复用、结果怎么沉淀。

这正是 Intra Codex 想解决的问题。

它不是一个新的大模型，也不是另一个聊天机器人。它更像一个面向研发场景的桌面工作台：把公司内网已经能运行的 `opencode`、`cac`、`codeagent` 或其他 CLI Agent，统一组织成项目、会话、Skills、上下文入口和运行记录。

---

## 一、CLI Agent 能运行，不代表它已经好用

很多团队第一次接入 Agent 时，工作方式通常是这样的：

1. 打开命令行。
2. 进入代码目录。
3. 重新解释项目背景。
4. 复制一段约束 Prompt。
5. 运行 Agent。
6. 从很长的 stdout/stderr 中寻找结果。
7. 下一次再重复一遍。

这个过程能完成任务，却很难形成持续工作流。

问题不在于 Agent 不够聪明，而在于使用方式过于零散：

- 项目上下文无法持续保留。
- 优秀 Prompt 和排查经验散落在个人记录里。
- 系统状态、Agent 回复和 CLI 错误混在一起。
- 输出难以直接变成 Wiki、交接文档或技术报告。
- 内网环境又无法依赖 CDN、在线图表引擎和大量临时 npm 包。

Intra Codex 的目标，就是给这些零散能力加上一层工程化组织。

---

## 二、Intra Codex 到底是什么

一句话概括：

> **Intra Codex 是一个围绕代码项目持续工作的内网 Agent Workbench。**

它负责的不是“生成能力”，而是“组织能力”。

它主要做七件事：

- 管理项目目录。
- 管理项目会话。
- 管理不同 CLI Agent Provider。
- 管理可复用 Skills。
- 提供文件上下文选择入口。
- 展示并持久化 Agent 运行过程。
- 把结果以适合内网的方式呈现出来。

真正执行探索、代码修改和内容生成的，仍然是底层 CLI Agent。

架构可以简化为：

```text
React UI
   ↓
Tauri IPC
   ↓
Rust Runtime
   ↓
CLI Agents
```

这也是项目最重要的边界：

> Intra Codex 是驾驶舱，不是发动机。

---

## 三、最适合它的三个使用场景

### 1. 快速接手陌生项目

选择一个代码目录后，使用默认的“探索项目”Skill，让 Agent 按固定路径阅读：

- README 和使用说明
- 构建与依赖配置
- 应用入口
- 路由与控制器
- 核心 service
- 数据模型
- 脚本与部署配置

它不会一上来就修改代码，而是先输出：

- 项目定位
- 模块地图
- 核心执行链路
- 关键文件
- 风险点与不确定点
- 下一步建议

这很适合项目交接、技术预研、历史系统梳理和比赛项目分析。

### 2. 把探索结果沉淀成 Wiki 页面

默认的“HTML 探索报告”Skill 会要求 Agent 把结果整理成纯 HTML/CSS 页面。

它有几个明确限制：

- 不允许 JavaScript。
- 不允许 `<script>`。
- 不允许 CDN。
- 不加载在线字体和在线图片。
- 不调用 PlantUML server 或 Mermaid CDN。
- 所有视觉效果只能使用 CSS。

这样生成的内容可以直接复制到支持 HTML/CSS 的内网 Wiki 中，而不是只能在本地临时打开一次。

### 3. 统一团队使用 Agent 的方式

团队可以把稳定的工作方法沉淀为 Skill，例如：

- 项目探索
- 故障排查
- Code Review
- 架构分析
- SQL 风险检查
- 发布前检查
- HTML 技术报告

相比让每个人记住一大段 Prompt，Skill 更像团队级的工作流模板。

---

## 四、第一次使用：六步完成一次项目探索

### 第一步：启动应用

开发环境可以运行：

```bash
npm install
npm run dev
```

内网机器不方便执行 `npm install` 时，推荐直接使用 Windows exe。

仓库已经提供 `Build Windows EXE` GitHub Actions 工作流，构建完成后会生成：

```text
intra-codex-windows-exe
```

下载 artifact 后即可在 Windows 中运行。

### 第二步：打开项目

点击左侧的 **打开项目**，选择代码目录。

系统会自动完成：

1. 读取项目文件树。
2. 创建一个新会话。
3. 切换到新会话。

不需要再手动点击“新建会话”才能开始。

### 第三步：选择 Agent

右侧 Agent 面板来自 `config/providers.yaml`。

可以为不同任务配置不同 Provider：

```yaml
providers:
  - id: opencode
    label: OpenCode
    command: opencode
    args: [run, --format, json]
    concurrency: 2
    inputMode: arg
```

通常可以按能力选择：

- 仓库搜索能力强的 Agent：用于探索项目。
- 代码修改能力强的 Agent：用于落地变更。
- 公司内部 wrapper：用于访问内网模型或内部工具。

### 第四步：确认 Skills

应用默认选中：

```text
explore
html-report
```

也就是：

- **探索项目**：先建立项目理解。
- **HTML 探索报告**：把理解整理成 Wiki 友好的结果。

这两个 Skill 组合起来，正好形成一个完整闭环：

```text
理解项目 → 整理结果 → 沉淀 Wiki
```

### 第五步：选择上下文文件

右侧 Project Files 支持勾选文件。

建议优先选择：

- README
- package.json / Cargo.toml / pom.xml
- 应用入口
- 路由配置
- 核心 service
- 数据模型
- 部署和环境配置

不要为了“给得更多”而全选整个项目。

好的上下文不是信息量最大，而是和当前任务最相关。

> 当前版本已经具备文件选择和路径传递能力；后端文件内容打包仍在继续完善。介绍材料不会把未完全落地的能力描述成已经完成。

### 第六步：输入任务并发送

快捷键：

```text
Ctrl + Enter      Windows / Linux 发送
Command + Enter   macOS 发送
Enter             换行
```

例如：

```text
请先探索这个项目，不要修改代码。
结合已选择的上下文文件，输出项目定位、模块地图、核心链路、关键文件、风险点和下一步建议。
```

---

## 五、一次完整任务应该怎么提问

### 项目探索

```text
先不要修改代码。
请基于项目目录和我选择的上下文文件，梳理这个项目的定位、模块边界、核心调用链路、关键数据结构和运行方式。
明确区分已确认事实和推测。
```

### 故障排查

```text
请先定位这个问题可能发生在哪条调用链路。
输出：排查顺序、关键日志点、可能原因、验证方法、修复方案和回归风险。
```

### 生成 Wiki 介绍页

```text
把刚才的探索结果整理成适合嵌入内网 Wiki 的介绍页。
只使用 div 和内联 CSS，不允许 JavaScript，不允许任何外部资源。
视觉风格简洁、专业、偏内部产品文档，不要做成营销落地页。
```

---

## 六、如何区分会话中的不同消息

Intra Codex 会把输出分成三类：

### 系统提示

应用自己的状态信息，例如：

- 开始执行哪个 Provider
- 运行失败
- 进程退出码
- 调度状态

它会单独高亮，并明确标记为“不是 Agent 回复”。

### Agent 回复

CLI Agent 的主要 stdout 内容，也是用户最关注的任务结果。

### CLI 输出

工具输出和 stderr，主要用于排查命令执行问题。

这种分层可以避免用户把“进程已退出”一类系统状态误认为是 Agent 给出的业务结论。

---

## 七、为什么 Markdown 渲染必须离线

内网应用不能默认假设可以访问：

- npm CDN
- GitHub raw 资源
- 在线字体
- Mermaid CDN
- PlantUML server
- 在线图片和图标服务

因此，会话中的 Markdown 由项目自己在前端本地解析和展示。

当前支持：

- 标题
- 段落
- 有序和无序列表
- 引用
- 表格
- 代码块
- 行内代码
- 粗体和斜体

PlantUML 和 Mermaid 会保留源码代码块，但不会调用在线引擎。

这虽然没有在线渲染那么华丽，却更符合真实内网环境。

---

## 八、如何嵌入公司 Wiki

仓库提供：

```text
docs/wiki-intro-div-only.html
```

这个文件专门用于 Wiki 嵌入：

- 没有 JavaScript。
- 没有 `<script>`。
- 页面主体只使用 div。
- CSS 直接写在 `<style>` 中。
- 动效只使用 transition、animation 和 keyframes。
- 不依赖外部字体、图片或图标。

将整个文件内容复制到支持 HTML/CSS 的 Wiki 页面即可。

如果 Wiki 会过滤 `<style>` 或 CSS 动画，页面仍会退化成普通静态内容，核心信息不会丢失。

---

## 九、项目配置入口

### Provider 配置

```text
config/providers.yaml
```

用于定义：

- Agent ID 和名称
- CLI command
- args
- concurrency
- cwd
- inputMode
- shell
- env
- capabilities

### Skills 目录

```text
skills/
```

每个 Markdown 文件会作为一个 Skill 加载。

一级标题会成为 UI 中显示的 Skill 名称。

### 本地状态

项目、会话和运行记录由 Tauri/Rust runtime 在本地持久化。

---

## 十、它现在的价值与下一步

当前 Intra Codex 已经形成了一个可使用的基础工作台：

- 项目和会话管理
- 打开项目自动建会话
- 多 Provider 配置
- Skills 选择
- 文件树和上下文入口
- CLI 流式输出
- 系统提示分层
- 离线 Markdown 渲染
- Windows exe 自动构建
- Wiki 介绍页

下一阶段最值得完善的不是继续堆 UI，而是：

1. 后端安全读取选中文件内容。
2. Context Pack 和 token budget。
3. 大文件截断与二进制文件过滤。
4. 结构化 CLI JSON 输出。
5. diff、结果、错误分区展示。
6. 更成熟的团队 Skills 库。

---

## 结语

AI 编程工具真正进入团队之后，竞争点会慢慢从“谁能调用模型”转向“谁能把模型组织成稳定工作流”。

Intra Codex 做的事情并不神秘：

- 给 Agent 一个项目入口。
- 给任务一个持续会话。
- 给团队经验一个 Skill 载体。
- 给运行过程一个清晰界面。
- 给最终结果一个可沉淀的出口。

如果 CLI Agent 是发动机，那么 Intra Codex 就是驾驶舱、仪表盘和操作手册。
