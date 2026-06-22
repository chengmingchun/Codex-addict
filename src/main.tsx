import React, { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  Activity,
  Bot,
  Check,
  CircleStop,
  Cpu,
  File,
  Folder,
  FolderOpen,
  Gauge,
  MessageSquarePlus,
  Moon,
  PanelRight,
  RefreshCw,
  Send,
  Sparkles,
  Sun,
  Terminal,
  Zap
} from "lucide-react";
import {
  cancelRun,
  createSession,
  loadSnapshot,
  openProject,
  pickDirectory,
  reloadConfig,
  sendMessage,
  setSkillsRoot,
  subscribeToUpdates
} from "./api";
import type { FileNode, ServerSnapshot } from "./types";
import "./styles.css";

type Theme = "dark" | "light";

const statusText: Record<string, string> = {
  queued: "排队",
  running: "运行",
  done: "空闲",
  failed: "失败",
  cancelled: "取消"
};

function App() {
  const [snapshot, setSnapshot] = useState<ServerSnapshot | null>(null);
  const [activeProjectId, setActiveProjectId] = useState<string | undefined>();
  const [activeSessionId, setActiveSessionId] = useState<string | undefined>();
  const [agentId, setAgentId] = useState("");
  const [skillIds, setSkillIds] = useState<string[]>(["frontend"]);
  const [draft, setDraft] = useState("");
  const [theme, setTheme] = useState<Theme>(() => (localStorage.getItem("theme") as Theme) || "light");
  const [isSending, setIsSending] = useState(false);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
  }, [theme]);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    void refresh();
    void subscribeToUpdates(
      (next) => setSnapshot(next),
      (run) => {
        setSnapshot((current) => {
          if (!current) return current;
          const exists = current.runs.some((item) => item.id === run.id);
          return { ...current, runs: exists ? current.runs.map((item) => (item.id === run.id ? run : item)) : [run, ...current.runs] };
        });
      },
      (session) => {
        setSnapshot((current) => {
          if (!current) return current;
          return {
            ...current,
            projects: current.projects.map((project) =>
              project.id === session.projectId
                ? {
                    ...project,
                    sessions: project.sessions.some((item) => item.id === session.id)
                      ? project.sessions.map((item) => (item.id === session.id ? session : item))
                      : [session, ...project.sessions]
                  }
                : project
            )
          };
        });
      }
    ).then((unlisten) => {
      cleanup = unlisten;
    });
    return () => cleanup?.();
  }, []);

  useEffect(() => {
    if (!snapshot) return;
    if (!agentId) setAgentId(snapshot.config.defaults.providerId);
    if (!activeProjectId) setActiveProjectId(snapshot.activeProjectId || snapshot.projects[0]?.id);
    if (!activeSessionId) {
      const project = snapshot.projects.find((item) => item.id === (snapshot.activeProjectId || snapshot.projects[0]?.id));
      setActiveSessionId(snapshot.activeSessionId || project?.sessions[0]?.id);
    }
  }, [activeProjectId, activeSessionId, agentId, snapshot]);

  async function refresh() {
    setSnapshot(await loadSnapshot());
  }

  async function chooseProject() {
    const picked = await pickDirectory();
    if (!picked) return;
    const project = await openProject(picked);
    const next = await loadSnapshot();
    setSnapshot(next);
    setActiveProjectId(project.id);
    setActiveSessionId(project.sessions[0]?.id);
  }

  async function newSession() {
    if (!activeProject) return;
    const session = await createSession({ projectId: activeProject.id, agentId, skillIds, title: "新会话" });
    setSnapshot(await loadSnapshot());
    setActiveSessionId(session.id);
  }

  async function submit() {
    if (!activeProject || !activeSession || !draft.trim()) return;
    setIsSending(true);
    try {
      await sendMessage({ projectId: activeProject.id, sessionId: activeSession.id, agentId, skillIds, content: draft });
      setDraft("");
      setSnapshot(await loadSnapshot());
    } finally {
      setIsSending(false);
    }
  }

  async function stopActiveRun() {
    if (!activeRun) return;
    await cancelRun(activeRun.id);
    await refresh();
  }

  async function chooseSkillsRoot() {
    const picked = await pickDirectory(snapshot?.config.defaults.skillsRoot || "skills");
    if (!picked) return;
    setSnapshot(await setSkillsRoot(picked));
    setSkillIds([]);
  }

  async function reloadEverything() {
    setSnapshot(await reloadConfig());
  }

  const activeProject = useMemo(() => snapshot?.projects.find((project) => project.id === activeProjectId) ?? snapshot?.projects[0], [activeProjectId, snapshot]);
  const activeSession = useMemo(
    () => activeProject?.sessions.find((session) => session.id === activeSessionId) ?? activeProject?.sessions[0],
    [activeProject, activeSessionId]
  );
  const activeAgent = useMemo(() => snapshot?.config.providers.find((agent) => agent.id === agentId) ?? snapshot?.config.providers[0], [agentId, snapshot]);
  const activeRun = useMemo(() => snapshot?.runs.find((run) => run.sessionId === activeSession?.id && run.status === "running"), [activeSession, snapshot]);

  if (!snapshot) {
    return (
      <main className="loading">
        <Terminal size={32} />
        <span>正在加载 Intra Codex...</span>
      </main>
    );
  }

  return (
    <main className="appShell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brandMark">
            <Bot size={23} />
          </div>
          <div>
            <strong>Intra Codex</strong>
            <span>Agent Workbench</span>
          </div>
        </div>

        <div className="sidebarActions">
          <button className="openProjectButton" onClick={chooseProject}>
            <FolderOpen size={17} />
            打开项目
          </button>
          <button className="iconButton" onClick={newSession} disabled={!activeProject} title="新建会话">
            <MessageSquarePlus size={17} />
          </button>
        </div>

        <section className="navSection">
          <div className="sectionLabel">Projects</div>
          {snapshot.projects.length ? (
            snapshot.projects.map((project) => (
              <button
                key={project.id}
                className={`navProject ${project.id === activeProject?.id ? "active" : ""}`}
                onClick={() => {
                  setActiveProjectId(project.id);
                  setActiveSessionId(project.sessions[0]?.id);
                }}
              >
                <Folder size={15} />
                <span>{project.name}</span>
              </button>
            ))
          ) : (
            <div className="sidebarEmpty">选择一个代码目录作为项目。</div>
          )}
        </section>

        <section className="navSection sessionsNav">
          <div className="sectionLabel">Sessions</div>
          {activeProject?.sessions.length ? (
            activeProject.sessions.map((session) => (
              <button key={session.id} className={`sessionItem ${session.id === activeSession?.id ? "active" : ""}`} onClick={() => setActiveSessionId(session.id)}>
                <span className={`statusDot ${session.status}`} />
                <div>
                  <strong>{session.title}</strong>
                  <span>{statusText[session.status]} · {session.agentId}</span>
                </div>
              </button>
            ))
          ) : (
            <button className="createFirstSession" onClick={newSession} disabled={!activeProject}>
              <MessageSquarePlus size={16} />
              新建会话
            </button>
          )}
        </section>
      </aside>

      <section className="mainPane">
        <header className="sessionHeader">
          <div>
            <span className="eyebrow">Project Session</span>
            <h1>{activeSession?.title || activeProject?.name || "打开项目开始"}</h1>
            <p>{activeProject?.path || "把本地 agent CLI 组织成项目、会话、skills 和上下文。"}</p>
          </div>
          <div className="headerActions">
            <Metric icon={<Activity size={16} />} label="运行" value={snapshot.usage.running.toString()} />
            <Metric icon={<Gauge size={16} />} label="排队" value={snapshot.usage.queued.toString()} />
            <button className="iconButton" onClick={reloadEverything} title="重新加载配置">
              <RefreshCw size={17} />
            </button>
            <button className="iconButton" onClick={() => setTheme(theme === "dark" ? "light" : "dark")} title="切换浅色/深色">
              {theme === "dark" ? <Sun size={17} /> : <Moon size={17} />}
            </button>
          </div>
        </header>

        <section className="thread">
          {activeSession ? (
            activeSession.messages.length ? (
              activeSession.messages.map((message) => (
                <article key={message.id} className={`message ${message.role}`}>
                  <header>{roleLabel(message.role)}</header>
                  <pre>{message.content}</pre>
                </article>
              ))
            ) : (
              <div className="threadEmpty">
                <Sparkles size={28} />
                <strong>这个会话还没有消息</strong>
                <span>右侧选择 Agent 和 Skills，然后从下方输入目标。</span>
              </div>
            )
          ) : (
            <div className="threadEmpty">
              <FolderOpen size={28} />
              <strong>先打开项目，再创建会话</strong>
              <span>项目会保留文件树、会话历史和 Agent 运行记录。</span>
            </div>
          )}
        </section>

        <section className="composer">
          <div className="composerMeta">
            <span>{activeAgent ? `${activeAgent.label}` : "未选择 Agent"}</span>
            <span>{skillIds.length} skills</span>
          </div>
          <textarea value={draft} onChange={(event) => setDraft(event.target.value)} placeholder="像使用 Codex 一样，描述你希望当前项目会话继续完成什么..." />
          <div className="composerFooter">
            <span>{activeAgent ? `${activeAgent.command} ${activeAgent.args.join(" ")}` : "请选择右侧 Agent"}</span>
            {activeRun ? (
              <button className="ghostButton" onClick={stopActiveRun}>
                <CircleStop size={17} />
                停止
              </button>
            ) : (
              <button className="primaryButton" onClick={submit} disabled={!activeSession || !draft.trim() || isSending}>
                <Send size={17} />
                发送
              </button>
            )}
          </div>
        </section>
      </section>

      <aside className="contextPane">
        <PanelTitle icon={<Cpu size={18} />} title="Agent Pool" />
        <div className="agentPool">
          {snapshot.config.providers.map((agent) => (
            <button key={agent.id} className={`agentCard ${agent.id === activeAgent?.id ? "active" : ""}`} onClick={() => setAgentId(agent.id)}>
              <strong>{agent.label}</strong>
              <span>{agent.command} {agent.args.join(" ")}</span>
              <div className="chips">{(agent.capabilities || []).map((capability) => <b key={capability}>{capability}</b>)}</div>
            </button>
          ))}
        </div>

        <PanelTitle icon={<Sparkles size={18} />} title="Skills" />
        <div className="pathConfig">
          <div>
            <strong>{snapshot.config.defaults.skillsRoot || "skills"}</strong>
            <span>Markdown skill 目录</span>
          </div>
          <button className="secondaryButton" onClick={chooseSkillsRoot}>
            <FolderOpen size={16} />
            选择
          </button>
        </div>
        <div className="skillSummary">
          {snapshot.skills.map((skill) => (
            <label key={skill.id} className="checkRow">
              <input type="checkbox" checked={skillIds.includes(skill.id)} onChange={() => toggleSkill(skill.id, skillIds, setSkillIds)} />
              <span><Check size={14} /> {skill.title}</span>
            </label>
          ))}
        </div>

        <PanelTitle icon={<PanelRight size={18} />} title="Project Files" />
        <div className="contextFileTree">
          {activeProject?.files.length ? activeProject.files.map((node) => <FileTreeNode key={node.path} node={node} depth={0} />) : <span className="mutedText">打开项目后显示文件树</span>}
        </div>

        <PanelTitle icon={<Zap size={18} />} title="Usage" />
        <div className="usageCompact">
          <strong>{snapshot.usage.tokensUsed.toLocaleString()}</strong>
          <span>/ {snapshot.usage.tokenBudget.toLocaleString()} tokens</span>
        </div>
      </aside>
    </main>
  );
}

function FileTreeNode({ node, depth }: { node: FileNode; depth: number }) {
  return (
    <div className="fileNode" style={{ paddingLeft: `${depth * 12 + 8}px` }}>
      <div title={node.path}>{node.kind === "directory" ? <Folder size={14} /> : <File size={14} />} {node.name}</div>
      {node.children.map((child) => <FileTreeNode key={child.path} node={child} depth={depth + 1} />)}
    </div>
  );
}

function Metric({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className="metric">
      {icon}
      <span>{label}</span>
      <b>{value}</b>
    </div>
  );
}

function PanelTitle({ icon, title }: { icon: React.ReactNode; title: string }) {
  return (
    <h2 className="panelTitle">
      {icon}
      {title}
    </h2>
  );
}

function roleLabel(role: string) {
  if (role === "user") return "You";
  if (role === "assistant") return "Agent";
  if (role === "tool") return "CLI";
  return "System";
}

function toggleSkill(skillId: string, skillIds: string[], setSkillIds: React.Dispatch<React.SetStateAction<string[]>>) {
  setSkillIds(skillIds.includes(skillId) ? skillIds.filter((id) => id !== skillId) : [...skillIds, skillId]);
}

createRoot(document.getElementById("root")!).render(<App />);
