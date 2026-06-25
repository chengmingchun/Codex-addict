#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod context;

use chrono::Local;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentConfig {
    id: String,
    label: String,
    command: String,
    args: Vec<String>,
    concurrency: usize,
    cwd: String,
    input_mode: Option<String>,
    shell: Option<bool>,
    env: HashMap<String, String>,
    capabilities: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Defaults {
    provider_id: String,
    max_global_concurrency: usize,
    workspace_root: String,
    skills_root: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    providers: Vec<AgentConfig>,
    defaults: Defaults,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Skill {
    id: String,
    title: String,
    body: String,
    path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum RunStatus {
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    id: String,
    role: String,
    content: String,
    created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentRun {
    id: String,
    project_id: String,
    session_id: String,
    agent_id: String,
    status: RunStatus,
    created_at: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    exit_code: Option<i32>,
    input_tokens: usize,
    output_tokens: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    id: String,
    project_id: String,
    title: String,
    agent_id: String,
    skill_ids: Vec<String>,
    status: RunStatus,
    created_at: String,
    updated_at: String,
    messages: Vec<Message>,
    run_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileNode {
    name: String,
    path: String,
    kind: String,
    children: Vec<FileNode>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    id: String,
    name: String,
    path: String,
    created_at: String,
    updated_at: String,
    sessions: Vec<Session>,
    files: Vec<FileNode>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderLoad {
    running: usize,
    queued: usize,
    concurrency: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UsageSnapshot {
    running: usize,
    queued: usize,
    completed_today: usize,
    token_budget: usize,
    tokens_used: usize,
    provider_load: HashMap<String, ProviderLoad>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerSnapshot {
    config: AppConfig,
    skills: Vec<Skill>,
    projects: Vec<Project>,
    active_project_id: Option<String>,
    active_session_id: Option<String>,
    runs: Vec<AgentRun>,
    usage: UsageSnapshot,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSessionRequest {
    project_id: String,
    agent_id: String,
    skill_ids: Vec<String>,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendMessageRequest {
    project_id: String,
    session_id: String,
    agent_id: String,
    skill_ids: Vec<String>,
    content: String,
    context_paths: Option<Vec<String>>,
}

struct Inner {
    root_dir: PathBuf,
    state_path: PathBuf,
    config: AppConfig,
    skills: Vec<Skill>,
    projects: HashMap<String, Project>,
    runs: HashMap<String, AgentRun>,
    children: HashMap<String, Arc<Mutex<Child>>>,
    active_project_id: Option<String>,
    active_session_id: Option<String>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedState {
    projects: HashMap<String, Project>,
    runs: HashMap<String, AgentRun>,
    active_project_id: Option<String>,
    active_session_id: Option<String>,
}

struct RuntimeState(Arc<Mutex<Inner>>);

#[tauri::command]
fn snapshot(state: State<RuntimeState>) -> Result<ServerSnapshot, String> {
    let inner = state.0.lock().map_err(|error| error.to_string())?;
    Ok(make_snapshot(&inner))
}

#[tauri::command]
fn reload_config(app: AppHandle, state: State<RuntimeState>) -> Result<ServerSnapshot, String> {
    let snapshot = {
        let mut inner = state.0.lock().map_err(|error| error.to_string())?;
        inner.config = read_config(&inner.root_dir)?;
        inner.skills = read_skills(&inner.root_dir, &inner.config)?;
        make_snapshot(&inner)
    };
    emit_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn update_agent_config(
    app: AppHandle,
    state: State<RuntimeState>,
    agent: AgentConfig,
) -> Result<ServerSnapshot, String> {
    let snapshot = {
        let mut inner = state.0.lock().map_err(|error| error.to_string())?;
        let index = inner
            .config
            .providers
            .iter()
            .position(|provider| provider.id == agent.id)
            .ok_or_else(|| "Agent provider not found.".to_string())?;
        inner.config.providers[index] = agent;
        write_config(&inner.root_dir, &inner.config)?;
        inner.skills = read_skills(&inner.root_dir, &inner.config)?;
        make_snapshot(&inner)
    };
    emit_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn pick_directory(
    state: State<RuntimeState>,
    start_path: Option<String>,
) -> Result<Option<String>, String> {
    let start = {
        let inner = state.0.lock().map_err(|error| error.to_string())?;
        start_path
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                resolve_workspace_root(&inner.root_dir, &inner.config.defaults.workspace_root)
            })
    };
    let mut dialog = rfd::FileDialog::new();
    if start.exists() {
        dialog = dialog.set_directory(start);
    }
    Ok(dialog
        .pick_folder()
        .map(|path| path_to_display_string(&path)))
}

#[tauri::command]
fn open_project(
    app: AppHandle,
    state: State<RuntimeState>,
    path: String,
) -> Result<Project, String> {
    let project_path = normalize_path(PathBuf::from(path));
    if !project_path.is_dir() {
        return Err("Project path must be a directory.".to_string());
    }
    let project = {
        let mut inner = state.0.lock().map_err(|error| error.to_string())?;
        let existing_id = inner
            .projects
            .values()
            .find(|project| normalize_path(PathBuf::from(&project.path)) == project_path)
            .map(|project| project.id.clone());
        let id = existing_id.unwrap_or_else(short_id);
        let files = read_file_tree(&project_path, 0, 3);
        let name = project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let project = inner.projects.entry(id.clone()).or_insert(Project {
            id: id.clone(),
            name,
            path: path_to_display_string(&project_path),
            created_at: now(),
            updated_at: now(),
            sessions: Vec::new(),
            files: Vec::new(),
        });
        project.updated_at = now();
        project.files = files;
        let project = display_project(project.clone());
        inner.active_project_id = Some(project.id.clone());
        inner.active_session_id = project.sessions.first().map(|session| session.id.clone());
        save_state(&inner);
        project
    };
    emit_snapshot_from_state(&app, &state.0);
    Ok(project)
}

#[tauri::command]
fn create_session(
    app: AppHandle,
    state: State<RuntimeState>,
    request: CreateSessionRequest,
) -> Result<Session, String> {
    let session = {
        let mut inner = state.0.lock().map_err(|error| error.to_string())?;
        let project = inner
            .projects
            .get_mut(&request.project_id)
            .ok_or_else(|| "Project not found.".to_string())?;
        let session = Session {
            id: short_id(),
            project_id: request.project_id.clone(),
            title: request.title.unwrap_or_else(|| "新会话".to_string()),
            agent_id: request.agent_id,
            skill_ids: request.skill_ids,
            status: RunStatus::Done,
            created_at: now(),
            updated_at: now(),
            messages: Vec::new(),
            run_ids: Vec::new(),
        };
        project.sessions.insert(0, session.clone());
        project.updated_at = now();
        inner.active_project_id = Some(request.project_id);
        inner.active_session_id = Some(session.id.clone());
        save_state(&inner);
        session
    };
    let _ = app.emit("session-event", &session);
    emit_snapshot_from_state(&app, &state.0);
    Ok(session)
}

#[tauri::command]
fn send_message(
    app: AppHandle,
    state: State<RuntimeState>,
    request: SendMessageRequest,
) -> Result<AgentRun, String> {
    if request.content.trim().is_empty() {
        return Err("Message is required.".to_string());
    }

    let run = {
        let mut inner = state.0.lock().map_err(|error| error.to_string())?;
        let project = inner
            .projects
            .get_mut(&request.project_id)
            .ok_or_else(|| "Project not found.".to_string())?;
        let session = project
            .sessions
            .iter_mut()
            .find(|session| session.id == request.session_id)
            .ok_or_else(|| "Session not found.".to_string())?;
        let packed_content = context::pack_message_with_context(
            &project.path,
            &request.content,
            request.context_paths.as_deref().unwrap_or(&[]),
        );
        if session.title == "新会话" {
            session.title = request
                .content
                .lines()
                .next()
                .unwrap_or("新会话")
                .chars()
                .take(48)
                .collect();
        }
        session.agent_id = request.agent_id.clone();
        session.skill_ids = request.skill_ids.clone();
        session.status = RunStatus::Queued;
        session.updated_at = now();
        session.messages.push(Message {
            id: short_id(),
            role: "user".to_string(),
            content: packed_content.clone(),
            created_at: now(),
        });
        let run = AgentRun {
            id: short_id(),
            project_id: request.project_id.clone(),
            session_id: request.session_id.clone(),
            agent_id: request.agent_id,
            status: RunStatus::Queued,
            created_at: now(),
            started_at: None,
            finished_at: None,
            exit_code: None,
            input_tokens: rough_tokens(&packed_content),
            output_tokens: 0,
        };
        session.run_ids.push(run.id.clone());
        inner.runs.insert(run.id.clone(), run.clone());
        inner.active_project_id = Some(request.project_id);
        inner.active_session_id = Some(request.session_id);
        save_state(&inner);
        run
    };

    let _ = app.emit("run-event", &run);
    schedule(app, state.0.clone());
    Ok(run)
}

#[tauri::command]
fn cancel_run(
    app: AppHandle,
    state: State<RuntimeState>,
    run_id: String,
) -> Result<AgentRun, String> {
    let run = {
        let mut inner = state.0.lock().map_err(|error| error.to_string())?;
        if let Some(child) = inner.children.get(&run_id) {
            let _ = child.lock().map_err(|error| error.to_string())?.kill();
        }
        let run = inner
            .runs
            .get_mut(&run_id)
            .ok_or_else(|| "Run not found.".to_string())?;
        run.status = RunStatus::Cancelled;
        run.finished_at = Some(now());
        let run = run.clone();
        set_session_status(
            &mut inner,
            &run.project_id,
            &run.session_id,
            RunStatus::Cancelled,
        );
        save_state(&inner);
        run
    };
    let _ = app.emit("run-event", &run);
    emit_snapshot_from_state(&app, &state.0);
    Ok(run)
}

#[tauri::command]
fn set_skills_root(
    app: AppHandle,
    state: State<RuntimeState>,
    skills_root: String,
) -> Result<ServerSnapshot, String> {
    let snapshot = {
        let mut inner = state.0.lock().map_err(|error| error.to_string())?;
        inner.config.defaults.skills_root = Some(skills_root);
        inner.skills = read_skills(&inner.root_dir, &inner.config)?;
        save_state(&inner);
        make_snapshot(&inner)
    };
    emit_snapshot(&app, &snapshot);
    Ok(snapshot)
}

fn schedule(app: AppHandle, state: Arc<Mutex<Inner>>) {
    loop {
        let next = {
            let mut inner = match state.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            let running_total = inner
                .runs
                .values()
                .filter(|run| run.status == RunStatus::Running)
                .count();
            if running_total >= inner.config.defaults.max_global_concurrency {
                emit_snapshot(&app, &make_snapshot(&inner));
                return;
            }

            let mut queued = inner
                .runs
                .values()
                .filter(|run| run.status == RunStatus::Queued)
                .cloned()
                .collect::<Vec<_>>();
            queued.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            let mut selected = None;
            for run in queued {
                let Some(agent) = inner
                    .config
                    .providers
                    .iter()
                    .find(|agent| agent.id == run.agent_id)
                else {
                    fail_run(&mut inner, &run, "Agent provider not found.");
                    continue;
                };
                let provider_running = inner
                    .runs
                    .values()
                    .filter(|item| item.agent_id == agent.id && item.status == RunStatus::Running)
                    .count();
                if provider_running < agent.concurrency {
                    selected = Some((run, agent.clone()));
                    break;
                }
            }

            let Some((mut run, agent)) = selected else {
                emit_snapshot(&app, &make_snapshot(&inner));
                return;
            };

            run.status = RunStatus::Running;
            run.started_at = Some(now());
            inner.runs.insert(run.id.clone(), run.clone());
            set_session_status(
                &mut inner,
                &run.project_id,
                &run.session_id,
                RunStatus::Running,
            );
            push_session_message(
                &mut inner,
                &run.project_id,
                &run.session_id,
                "system",
                format!("Starting {}: {}", agent.label, command_preview(&agent)),
            );

            let project = inner.projects.get(&run.project_id).cloned();
            let session = project.as_ref().and_then(|project| {
                project
                    .sessions
                    .iter()
                    .find(|session| session.id == run.session_id)
                    .cloned()
            });
            let selected_skills = session
                .as_ref()
                .map(|session| {
                    inner
                        .skills
                        .iter()
                        .filter(|skill| session.skill_ids.contains(&skill.id))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let Some(project) = project else { continue };
            let Some(session) = session else { continue };
            (run, agent, project, session, selected_skills)
        };

        emit_snapshot_from_state(&app, &state);
        start_run(
            app.clone(),
            state.clone(),
            next.0,
            next.1,
            next.2,
            next.3,
            next.4,
        );
    }
}

fn start_run(
    app: AppHandle,
    state: Arc<Mutex<Inner>>,
    run: AgentRun,
    agent: AgentConfig,
    project: Project,
    session: Session,
    selected_skills: Vec<Skill>,
) {
    let prompt = build_prompt(&project, &session, &selected_skills);
    thread::spawn(move || {
        let input_mode = agent.input_mode.as_deref().unwrap_or("stdin").to_string();
        let mut effective_args = agent.args.clone();
        if input_mode == "arg" {
            effective_args.push(prompt.clone());
        }
        let transcript = Arc::new(Mutex::new(String::new()));
        let mut command = if agent.shell.unwrap_or(false) {
            shell_command(&agent, &effective_args)
        } else {
            let mut command = Command::new(&agent.command);
            command.args(&effective_args);
            command
        };

        command
            .current_dir(PathBuf::from(&project.path))
            .envs(agent.env.clone())
            .stdin(if input_mode == "stdin" {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        apply_no_window(&mut command);

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                append_output(&app, &state, &run, "system", error.to_string());
                finish_run(&app, &state, &run.id, RunStatus::Failed, None);
                schedule(app, state);
                return;
            }
        };

        if input_mode == "stdin" {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(prompt.as_bytes());
            }
        }
        let stdout_reader = child.stdout.take().map(|stdout| {
            spawn_reader(
                app.clone(),
                state.clone(),
                run.clone(),
                "assistant",
                stdout,
                transcript.clone(),
            )
        });
        let stderr_reader = child.stderr.take().map(|stderr| {
            spawn_reader(
                app.clone(),
                state.clone(),
                run.clone(),
                "tool",
                stderr,
                transcript.clone(),
            )
        });

        let child_ref = Arc::new(Mutex::new(child));
        if let Ok(mut inner) = state.lock() {
            inner.children.insert(run.id.clone(), child_ref.clone());
        }

        let exit_code = loop {
            let result = {
                let mut locked = match child_ref.lock() {
                    Ok(guard) => guard,
                    Err(_) => break None,
                };
                locked.try_wait()
            };
            match result {
                Ok(Some(status)) => break status.code(),
                Ok(None) => thread::sleep(Duration::from_millis(120)),
                Err(_) => break None,
            }
        };
        if let Some(reader) = stdout_reader {
            let _ = reader.join();
        }
        if let Some(reader) = stderr_reader {
            let _ = reader.join();
        }

        let transcript = transcript
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default();
        let cli_error = contains_cli_error(&transcript);
        let final_status = if exit_code == Some(0) && !cli_error {
            RunStatus::Done
        } else {
            RunStatus::Failed
        };
        finish_run(&app, &state, &run.id, final_status, exit_code);
        let exit_note = if cli_error {
            format!(
                "Exited with code {:?}. Detected an error in CLI output.",
                exit_code
            )
        } else {
            format!("Exited with code {:?}.", exit_code)
        };
        append_output(
            &app,
            &state,
            &run,
            "system",
            exit_note,
        );
        if let Ok(mut inner) = state.lock() {
            inner.children.remove(&run.id);
        }
        schedule(app, state);
    });
}

fn spawn_reader<R: Read + Send + 'static>(
    app: AppHandle,
    state: Arc<Mutex<Inner>>,
    run: AgentRun,
    role: &'static str,
    mut reader: R,
    transcript: Arc<Mutex<String>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let text = String::from_utf8_lossy(&buffer[..size]).to_string();
                    if let Ok(mut output) = transcript.lock() {
                        output.push_str(&text);
                    }
                    append_output(&app, &state, &run, role, text);
                }
                Err(_) => break,
            }
        }
    })
}

fn append_output<T: Into<String>>(
    app: &AppHandle,
    state: &Arc<Mutex<Inner>>,
    run: &AgentRun,
    role: &str,
    text: T,
) {
    let (updated_run, updated_session) = {
        let mut inner = match state.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let text = text.into();
        let output_tokens = rough_tokens(&text);
        if let Some(stored_run) = inner.runs.get_mut(&run.id) {
            if role == "assistant" || role == "tool" {
                stored_run.output_tokens += output_tokens;
            }
        }
        append_session_output(&mut inner, &run.project_id, &run.session_id, role, text);
        let updated_run = inner.runs.get(&run.id).cloned();
        let updated_session = find_session(&inner, &run.project_id, &run.session_id).cloned();
        (updated_run, updated_session)
    };
    if let Some(run) = updated_run {
        let _ = app.emit("run-event", &run);
    }
    if let Some(session) = updated_session {
        let _ = app.emit("session-event", &session);
    }
    if let Ok(inner) = state.lock() {
        save_state(&inner);
    }
}

fn finish_run(
    app: &AppHandle,
    state: &Arc<Mutex<Inner>>,
    run_id: &str,
    status: RunStatus,
    exit_code: Option<i32>,
) {
    let (run, session) = {
        let mut inner = match state.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let Some(stored_run) = inner.runs.get_mut(run_id) else {
            return;
        };
        if stored_run.status != RunStatus::Cancelled {
            stored_run.status = status.clone();
        }
        stored_run.exit_code = exit_code;
        stored_run.finished_at = Some(now());
        let run = stored_run.clone();
        set_session_status(
            &mut inner,
            &run.project_id,
            &run.session_id,
            run.status.clone(),
        );
        let session = find_session(&inner, &run.project_id, &run.session_id).cloned();
        (run, session)
    };
    let _ = app.emit("run-event", &run);
    if let Some(session) = session {
        let _ = app.emit("session-event", &session);
    }
    if let Ok(inner) = state.lock() {
        save_state(&inner);
    }
    emit_snapshot_from_state(app, state);
}

fn fail_run(inner: &mut Inner, run: &AgentRun, text: &str) {
    if let Some(stored_run) = inner.runs.get_mut(&run.id) {
        stored_run.status = RunStatus::Failed;
        stored_run.finished_at = Some(now());
    }
    set_session_status(inner, &run.project_id, &run.session_id, RunStatus::Failed);
    push_session_message(
        inner,
        &run.project_id,
        &run.session_id,
        "system",
        text.to_string(),
    );
}

fn set_session_status(inner: &mut Inner, project_id: &str, session_id: &str, status: RunStatus) {
    if let Some(session) = find_session_mut(inner, project_id, session_id) {
        session.status = status;
        session.updated_at = now();
    }
}

fn push_session_message<T: Into<String>>(
    inner: &mut Inner,
    project_id: &str,
    session_id: &str,
    role: &str,
    content: T,
) {
    if let Some(session) = find_session_mut(inner, project_id, session_id) {
        session.messages.push(Message {
            id: short_id(),
            role: role.to_string(),
            content: content.into(),
            created_at: now(),
        });
        session.updated_at = now();
    }
}

fn append_session_output(
    inner: &mut Inner,
    project_id: &str,
    session_id: &str,
    role: &str,
    content: String,
) {
    if let Some(session) = find_session_mut(inner, project_id, session_id) {
        if let Some(last) = session.messages.last_mut() {
            if last.role == role {
                last.content.push_str(&content);
                session.updated_at = now();
                return;
            }
        }
        session.messages.push(Message {
            id: short_id(),
            role: role.to_string(),
            content,
            created_at: now(),
        });
        session.updated_at = now();
    }
}

fn find_session<'a>(inner: &'a Inner, project_id: &str, session_id: &str) -> Option<&'a Session> {
    inner
        .projects
        .get(project_id)?
        .sessions
        .iter()
        .find(|session| session.id == session_id)
}

fn find_session_mut<'a>(
    inner: &'a mut Inner,
    project_id: &str,
    session_id: &str,
) -> Option<&'a mut Session> {
    inner
        .projects
        .get_mut(project_id)?
        .sessions
        .iter_mut()
        .find(|session| session.id == session_id)
}

fn build_prompt(project: &Project, session: &Session, selected_skills: &[Skill]) -> String {
    let mut parts = vec![
        format!("Project path: {}", project.path),
        "You are running inside an internal Codex-like workbench. Continue the current project session and respond with useful progress and results.".to_string(),
    ];
    if !selected_skills.is_empty() {
        let skills = selected_skills
            .iter()
            .map(|skill| format!("- {}\n{}", skill.title, skill.body))
            .collect::<Vec<_>>()
            .join("\n\n");
        parts.push(format!("Loaded skills:\n{skills}"));
    }
    let history = session
        .messages
        .iter()
        .map(|message| format!("{}: {}", message.role, message.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    parts.push(format!("Session history:\n{history}"));
    parts.join("\n\n")
}

fn make_snapshot(inner: &Inner) -> ServerSnapshot {
    let mut projects = inner.projects.values().cloned().collect::<Vec<_>>();
    for project in &mut projects {
        project
            .sessions
            .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sanitize_project_paths(project);
    }
    projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let mut runs = inner.runs.values().cloned().collect::<Vec<_>>();
    runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let skills = inner
        .skills
        .iter()
        .cloned()
        .map(|mut skill| {
            skill.path = display_path_string(&skill.path);
            skill
        })
        .collect();
    ServerSnapshot {
        config: inner.config.clone(),
        skills,
        projects,
        active_project_id: inner.active_project_id.clone(),
        active_session_id: inner.active_session_id.clone(),
        usage: make_usage(inner),
        runs,
    }
}

fn make_usage(inner: &Inner) -> UsageSnapshot {
    let mut provider_load = HashMap::new();
    for provider in &inner.config.providers {
        provider_load.insert(
            provider.id.clone(),
            ProviderLoad {
                running: inner
                    .runs
                    .values()
                    .filter(|run| run.agent_id == provider.id && run.status == RunStatus::Running)
                    .count(),
                queued: inner
                    .runs
                    .values()
                    .filter(|run| run.agent_id == provider.id && run.status == RunStatus::Queued)
                    .count(),
                concurrency: provider.concurrency,
            },
        );
    }
    let tokens_used = inner
        .runs
        .values()
        .map(|run| run.input_tokens + run.output_tokens)
        .sum();
    UsageSnapshot {
        running: inner
            .runs
            .values()
            .filter(|run| run.status == RunStatus::Running)
            .count(),
        queued: inner
            .runs
            .values()
            .filter(|run| run.status == RunStatus::Queued)
            .count(),
        completed_today: inner
            .runs
            .values()
            .filter(|run| {
                run.status == RunStatus::Done
                    && run.finished_at.as_ref().is_some_and(|date| {
                        date.starts_with(&Local::now().format("%Y-%m-%d").to_string())
                    })
            })
            .count(),
        token_budget: 2_000_000,
        tokens_used,
        provider_load,
    }
}

fn command_preview(agent: &AgentConfig) -> String {
    let mut args = agent.args.clone();
    match agent.input_mode.as_deref().unwrap_or("stdin") {
        "arg" => args.push("<prompt>".to_string()),
        "stdin" => args.push("<prompt via stdin>".to_string()),
        _ => {}
    }
    std::iter::once(agent.command.clone())
        .chain(args)
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_cli_error(output: &str) -> bool {
    let cleaned = strip_ansi(output).to_lowercase();
    cleaned.contains("error:")
        || cleaned.contains("does not have a valid codingplan")
        || cleaned.contains("subscription has expired")
        || cleaned.contains("could not auto-migrate")
        || cleaned.contains("authentication failed")
        || cleaned.contains("permission denied")
        || cleaned.contains("rate limit")
}

fn strip_ansi(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn shell_command(provider: &AgentConfig, args: &[String]) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("cmd");
        let full = std::iter::once(quote_shell_arg(&provider.command))
            .chain(args.iter().map(|arg| quote_shell_arg(arg)))
            .collect::<Vec<_>>()
            .join(" ");
        command.args(["/C", &full]);
        command
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut command = Command::new("sh");
        let full = std::iter::once(quote_shell_arg(&provider.command))
            .chain(args.iter().map(|arg| quote_shell_arg(arg)))
            .collect::<Vec<_>>()
            .join(" ");
        command.args(["-lc", &full]);
        command
    }
}

fn quote_shell_arg(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | '\\' | ':' | '=')
    }) {
        return value.to_string();
    }
    format!("\"{}\"", value.replace('"', "\\\""))
}

fn apply_no_window(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = command;
    }
}

fn read_file_tree(root: &Path, depth: usize, max_depth: usize) -> Vec<FileNode> {
    if depth >= max_depth {
        return Vec::new();
    }
    let mut nodes = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return nodes;
    };
    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();
        let a_is_dir = a_path.is_dir();
        let b_is_dir = b_path.is_dir();
        b_is_dir.cmp(&a_is_dir).then(
            a.file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase()),
        )
    });
    for entry in entries
        .into_iter()
        .filter(|entry| !should_skip(&entry.file_name().to_string_lossy()))
        .take(96)
    {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();
        nodes.push(FileNode {
            name,
            path: path_to_display_string(&path),
            kind: if is_dir { "directory" } else { "file" }.to_string(),
            children: if is_dir {
                read_file_tree(&path, depth + 1, max_depth)
            } else {
                Vec::new()
            },
        });
    }
    nodes
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "release"
            | ".run"
            | ".next"
            | ".nuxt"
            | ".cache"
            | ".turbo"
            | ".vite"
            | "coverage"
    ) || name.ends_with(".log")
}

fn emit_snapshot(app: &AppHandle, snapshot: &ServerSnapshot) {
    let _ = app.emit("snapshot", snapshot);
}

fn emit_snapshot_from_state(app: &AppHandle, state: &Arc<Mutex<Inner>>) {
    if let Ok(inner) = state.lock() {
        emit_snapshot(app, &make_snapshot(&inner));
    }
}

fn read_config(root_dir: &Path) -> Result<AppConfig, String> {
    let raw = fs::read_to_string(root_dir.join("config").join("providers.yaml"))
        .map_err(|error| error.to_string())?;
    serde_yaml::from_str(&raw).map_err(|error| error.to_string())
}

fn write_config(root_dir: &Path, config: &AppConfig) -> Result<(), String> {
    let config_dir = root_dir.join("config");
    fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
    let raw = serde_yaml::to_string(config).map_err(|error| error.to_string())?;
    fs::write(config_dir.join("providers.yaml"), raw).map_err(|error| error.to_string())
}

fn load_state(state_path: &Path) -> PersistedState {
    let Ok(raw) = fs::read_to_string(state_path) else {
        return PersistedState::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_state(inner: &Inner) {
    let state = PersistedState {
        projects: inner.projects.clone(),
        runs: inner.runs.clone(),
        active_project_id: inner.active_project_id.clone(),
        active_session_id: inner.active_session_id.clone(),
    };
    if let Some(parent) = inner.state_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(raw) = serde_json::to_string_pretty(&state) {
        let _ = fs::write(&inner.state_path, raw);
    }
}

fn read_skills(root_dir: &Path, config: &AppConfig) -> Result<Vec<Skill>, String> {
    let skills_dir = resolve_configured_dir(
        root_dir,
        config.defaults.skills_root.as_deref().unwrap_or("skills"),
    );
    let mut skills = Vec::new();
    if !skills_dir.exists() {
        return Ok(skills);
    }
    for entry in fs::read_dir(&skills_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let body = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let title = body
            .lines()
            .find(|line| line.trim_start().starts_with('#'))
            .map(|line| line.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
        skills.push(Skill {
            id: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            title,
            body,
            path: path_to_display_string(&path),
        });
    }
    skills.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(skills)
}

fn resolve_configured_dir(root_dir: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root_dir.join(path)
    }
}

fn resolve_workspace_root(root_dir: &Path, workspace_root: &str) -> PathBuf {
    let configured = PathBuf::from(workspace_root);
    if configured.is_absolute() {
        normalize_path(configured)
    } else {
        normalize_path(root_dir.join(configured))
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn path_to_display_string(path: &Path) -> String {
    display_path_string(&path.to_string_lossy())
}

fn display_project(mut project: Project) -> Project {
    sanitize_project_paths(&mut project);
    project
}

fn sanitize_project_paths(project: &mut Project) {
    project.path = display_path_string(&project.path);
    for node in &mut project.files {
        sanitize_file_node_paths(node);
    }
}

fn sanitize_file_node_paths(node: &mut FileNode) {
    node.path = display_path_string(&node.path);
    for child in &mut node.children {
        sanitize_file_node_paths(child);
    }
}

fn display_path_string(value: &str) -> String {
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        format!("\\\\{}", rest)
    } else if let Some(rest) = value.strip_prefix("\\\\?\\") {
        rest.to_string()
    } else {
        value.to_string()
    }
}

fn now() -> String {
    Local::now().to_rfc3339()
}

fn short_id() -> String {
    Uuid::new_v4().to_string()[..10].to_string()
}

fn rough_tokens(text: &str) -> usize {
    (text.chars().count() + 3) / 4
}

fn resolve_root_dir(app: &tauri::App) -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf())
    } else {
        app.path().resource_dir().map_err(|error| error.to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let root_dir = resolve_root_dir(app)?;
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            let state_path = app_data_dir.join("state.json");
            let persisted = load_state(&state_path);
            let config = read_config(&root_dir)?;
            let inner = Inner {
                skills: read_skills(&root_dir, &config)?,
                config,
                projects: persisted.projects,
                runs: persisted.runs,
                children: HashMap::new(),
                active_project_id: persisted.active_project_id,
                active_session_id: persisted.active_session_id,
                state_path,
                root_dir,
            };
            app.manage(RuntimeState(Arc::new(Mutex::new(inner))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            reload_config,
            update_agent_config,
            pick_directory,
            open_project,
            create_session,
            send_message,
            cancel_run,
            set_skills_root
        ])
        .run(tauri::generate_context!())
        .expect("error while running Intra Codex");
}
