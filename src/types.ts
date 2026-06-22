export type AgentConfig = {
  id: string;
  label: string;
  command: string;
  args: string[];
  concurrency: number;
  cwd: string;
  shell?: boolean;
  env: Record<string, string>;
  capabilities?: string[];
};

export type AppConfig = {
  providers: AgentConfig[];
  defaults: {
    providerId: string;
    maxGlobalConcurrency: number;
    workspaceRoot: string;
    skillsRoot?: string;
  };
};

export type Skill = {
  id: string;
  title: string;
  body: string;
  path: string;
};

export type RunStatus = "queued" | "running" | "done" | "failed" | "cancelled";

export type MessageRole = "user" | "assistant" | "system" | "tool";

export type Message = {
  id: string;
  role: MessageRole;
  content: string;
  createdAt: string;
};

export type AgentRun = {
  id: string;
  projectId: string;
  sessionId: string;
  agentId: string;
  status: RunStatus;
  createdAt: string;
  startedAt?: string;
  finishedAt?: string;
  exitCode?: number | null;
  inputTokens: number;
  outputTokens: number;
};

export type Session = {
  id: string;
  projectId: string;
  title: string;
  agentId: string;
  skillIds: string[];
  status: RunStatus;
  createdAt: string;
  updatedAt: string;
  messages: Message[];
  runIds: string[];
};

export type FileNode = {
  name: string;
  path: string;
  kind: "file" | "directory";
  children: FileNode[];
};

export type Project = {
  id: string;
  name: string;
  path: string;
  createdAt: string;
  updatedAt: string;
  sessions: Session[];
  files: FileNode[];
};

export type UsageSnapshot = {
  running: number;
  queued: number;
  completedToday: number;
  tokenBudget: number;
  tokensUsed: number;
  providerLoad: Record<string, { running: number; queued: number; concurrency: number }>;
};

export type ServerSnapshot = {
  config: AppConfig;
  skills: Skill[];
  projects: Project[];
  activeProjectId?: string;
  activeSessionId?: string;
  runs: AgentRun[];
  usage: UsageSnapshot;
};
