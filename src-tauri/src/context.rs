use std::{
    fs,
    path::{Path, PathBuf},
};

const MAX_CONTEXT_FILES: usize = 12;
const MAX_CONTEXT_BYTES_PER_FILE: usize = 24 * 1024;
const MAX_CONTEXT_BYTES_TOTAL: usize = 96 * 1024;

pub fn pack_message_with_context(
    project_path: &str,
    content: &str,
    context_paths: &[String],
) -> String {
    if context_paths.is_empty() {
        return content.trim().to_string();
    }

    let context = pack_context_files(project_path, context_paths);
    format!(
        "You are given selected project context files below. Use this context as the primary source of truth before making changes or answering.\n\n{context}\n\nUser task:\n{}",
        content.trim()
    )
}

fn pack_context_files(project_path: &str, context_paths: &[String]) -> String {
    let root = normalize_path(PathBuf::from(project_path));
    let mut parts = vec![format!(
        "Context limits: max {MAX_CONTEXT_FILES} files, {MAX_CONTEXT_BYTES_PER_FILE} bytes per file, {MAX_CONTEXT_BYTES_TOTAL} bytes total."
    )];
    let mut used_total = 0usize;
    let mut packed = 0usize;

    for raw_path in context_paths.iter().filter(|path| !path.trim().is_empty()) {
        if packed >= MAX_CONTEXT_FILES || used_total >= MAX_CONTEXT_BYTES_TOTAL {
            break;
        }

        let candidate = PathBuf::from(raw_path);
        let candidate = if candidate.is_absolute() {
            candidate
        } else {
            root.join(candidate)
        };
        let path = normalize_path(candidate);

        if !path.starts_with(&root) {
            parts.push(format!("Skipped `{}`: outside project root.", raw_path));
            continue;
        }
        if !path.is_file() {
            parts.push(format!(
                "Skipped `{}`: not a readable file.",
                relative_path(&root, &path)
            ));
            continue;
        }

        let remaining = MAX_CONTEXT_BYTES_TOTAL.saturating_sub(used_total);
        let limit = MAX_CONTEXT_BYTES_PER_FILE.min(remaining);
        if limit == 0 {
            break;
        }

        match fs::read(&path) {
            Ok(bytes) => {
                let truncated = bytes.len() > limit;
                let take = bytes.len().min(limit);
                used_total += take;
                packed += 1;
                let body = String::from_utf8_lossy(&bytes[..take]);
                let rel = relative_path(&root, &path);
                parts.push(format!(
                    "## File: `{rel}`\n{}\n```\n{}\n```",
                    if truncated {
                        "_Truncated by context budget._"
                    } else {
                        ""
                    },
                    body
                ));
            }
            Err(error) => parts.push(format!(
                "Skipped `{}`: {}.",
                relative_path(&root, &path),
                error
            )),
        }
    }

    if packed == 0 {
        parts.push("No selected context files could be packed.".to_string());
    }
    parts.join("\n\n")
}

fn normalize_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
