import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AgentRun, Project, ServerSnapshot, Session } from "./types";

type SnapshotHandler = (snapshot: ServerSnapshot) => void;
type RunHandler = (run: AgentRun) => void;
type SessionHandler = (session: Session) => void;

const isTauri = "__TAURI_INTERNALS__" in window;

export async function loadSnapshot() {
  return invoke<ServerSnapshot>("snapshot");
}

export async function reloadConfig() {
  return invoke<ServerSnapshot>("reload_config");
}

export async function pickDirectory(startPath?: string) {
  if (isTauri) return invoke<string | null>("pick_directory", { startPath });
  return window.prompt("输入目录路径", startPath || ".") || null;
}

export async function openProject(path: string) {
  return invoke<Project>("open_project", { path });
}

export async function createSession(request: { projectId: string; agentId: string; skillIds: string[]; title?: string }) {
  return invoke<Session>("create_session", { request });
}

export async function sendMessage(request: { projectId: string; sessionId: string; agentId: string; skillIds: string[]; content: string }) {
  return invoke<AgentRun>("send_message", { request });
}

export async function cancelRun(runId: string) {
  return invoke<AgentRun>("cancel_run", { runId });
}

export async function setSkillsRoot(skillsRoot: string) {
  return invoke<ServerSnapshot>("set_skills_root", { skillsRoot });
}

export async function subscribeToUpdates(onSnapshot: SnapshotHandler, onRun: RunHandler, onSession: SessionHandler) {
  const unlistenSnapshot = await listen<ServerSnapshot>("snapshot", (event) => onSnapshot(event.payload));
  const unlistenRun = await listen<AgentRun>("run-event", (event) => onRun(event.payload));
  const unlistenSession = await listen<Session>("session-event", (event) => onSession(event.payload));
  return () => {
    unlistenSnapshot();
    unlistenRun();
    unlistenSession();
  };
}
