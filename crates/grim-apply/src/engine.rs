//! Planning and executing an apply: walk the source tree, render templates, diff against what's on
//! disk, and (unless this is a dry run) write the changes atomically.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::ApplyError;
use crate::name::{NamePart, parse_component};
use crate::render::RenderContext;

/// Whether an action creates, updates, or leaves a target unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// The target does not exist and will be created.
    Create,
    /// The target exists with different content/mode and will be updated.
    Update,
    /// The target already matches; nothing to do.
    Unchanged,
}

/// The resolved content of a source file: text (renderable/diffable) or opaque bytes.
#[derive(Debug, Clone)]
pub enum Body {
    /// UTF-8 text (templates always land here).
    Text(String),
    /// Non-UTF-8 bytes, copied verbatim.
    Binary(Vec<u8>),
}

impl Body {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Body::Text(s) => s.as_bytes(),
            Body::Binary(b) => b,
        }
    }
}

/// A single planned filesystem action.
#[derive(Debug, Clone)]
pub struct FileAction {
    /// The source path this action came from.
    pub source: PathBuf,
    /// The target path to be written.
    pub target: PathBuf,
    /// Whether the target is a directory.
    pub is_dir: bool,
    /// What this action does.
    pub change: Change,
    /// The mode to set, if any.
    pub mode: Option<u32>,
    /// The body to write (None for directories).
    pub body: Option<Body>,
    /// A unified diff against the existing target, when both sides are text and this is an update.
    pub diff: Option<String>,
}

/// An ordered set of planned actions.
#[derive(Debug, Default)]
pub struct Plan {
    /// The actions, ordered so parent directories precede their contents.
    pub actions: Vec<FileAction>,
}

impl Plan {
    /// Actions that actually change something.
    pub fn changed(&self) -> impl Iterator<Item = &FileAction> {
        self.actions
            .iter()
            .filter(|a| a.change != Change::Unchanged)
    }

    /// `(created, updated, unchanged)` counts.
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut c = (0, 0, 0);
        for a in &self.actions {
            match a.change {
                Change::Create => c.0 += 1,
                Change::Update => c.1 += 1,
                Change::Unchanged => c.2 += 1,
            }
        }
        c
    }
}

/// Plan an apply of `source_dir` into `target_root`, rendering templates with `ctx`. If `source_dir`
/// does not exist, the plan is empty. This reads source and target files but writes nothing.
pub fn plan(
    source_dir: &Path,
    target_root: &Path,
    ctx: &RenderContext,
) -> Result<Plan, ApplyError> {
    let mut actions = Vec::new();
    if source_dir.is_dir() {
        walk(source_dir, source_dir, target_root, ctx, &mut actions)?;
    }
    // Order by target path so a directory precedes everything beneath it.
    actions.sort_by(|a, b| a.target.cmp(&b.target));
    Ok(Plan { actions })
}

/// Execute a plan. With `dry_run`, does nothing (the plan is the preview). Otherwise creates
/// directories, writes files atomically, and sets modes.
pub fn execute(plan: &Plan, dry_run: bool) -> Result<(), ApplyError> {
    if dry_run {
        return Ok(());
    }
    for action in &plan.actions {
        if action.change == Change::Unchanged {
            continue;
        }
        if action.is_dir {
            fs::create_dir_all(&action.target).map_err(|source| ApplyError::Write {
                path: action.target.clone(),
                source,
            })?;
        } else {
            if let Some(parent) = action.target.parent() {
                fs::create_dir_all(parent).map_err(|source| ApplyError::Write {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            let body = action.body.as_ref().expect("file action has a body");
            atomic_write(&action.target, body.as_bytes())?;
        }
        set_mode(&action.target, action.mode)?;
    }
    Ok(())
}

fn walk(
    root: &Path,
    dir: &Path,
    target_root: &Path,
    ctx: &RenderContext,
    actions: &mut Vec<FileAction>,
) -> Result<(), ApplyError> {
    let entries = fs::read_dir(dir).map_err(|source| ApplyError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ApplyError::Walk {
            path: dir.to_path_buf(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        // Skip hidden entries in the source (e.g. .git, .DS_Store); grim sources use `dot_`.
        if name.starts_with('.') {
            continue;
        }
        let source = entry.path();
        let file_type = entry.file_type().map_err(|s| ApplyError::Walk {
            path: source.clone(),
            source: s,
        })?;
        let rel = source.strip_prefix(root).unwrap_or(&source);
        let (target, leaf) = map_target(target_root, rel);

        if file_type.is_dir() {
            let change = if target.is_dir() {
                Change::Unchanged
            } else {
                Change::Create
            };
            actions.push(FileAction {
                source: source.clone(),
                target,
                is_dir: true,
                change,
                mode: leaf.attr.dir_mode(),
                body: None,
                diff: None,
            });
            walk(root, &source, target_root, ctx, actions)?;
        } else if file_type.is_file() {
            actions.push(plan_file(&source, target, &leaf, ctx)?);
        }
    }
    Ok(())
}

fn plan_file(
    source: &Path,
    target: PathBuf,
    leaf: &NamePart,
    ctx: &RenderContext,
) -> Result<FileAction, ApplyError> {
    let raw = fs::read(source).map_err(|s| ApplyError::Read {
        path: source.to_path_buf(),
        source: s,
    })?;
    let body = if leaf.is_template {
        let text = String::from_utf8(raw).map_err(|_| ApplyError::NonUtf8Template {
            path: source.to_path_buf(),
        })?;
        Body::Text(ctx.render(source, &text)?)
    } else {
        match String::from_utf8(raw) {
            Ok(text) => Body::Text(text),
            Err(e) => Body::Binary(e.into_bytes()),
        }
    };

    let existing = fs::read(&target).ok();
    let (change, diff) = match &existing {
        None => (Change::Create, None),
        Some(current) if current == body.as_bytes() => (Change::Unchanged, None),
        Some(current) => (Change::Update, unified_diff(current, &body)),
    };

    Ok(FileAction {
        source: source.to_path_buf(),
        target,
        is_dir: false,
        change,
        mode: leaf.attr.file_mode(),
        body: Some(body),
        diff,
    })
}

/// Map a source path (relative to the source root) to its target path, transforming every component
/// and returning the parsed leaf component (which carries mode/template attributes).
fn map_target(target_root: &Path, rel: &Path) -> (PathBuf, NamePart) {
    let mut out = target_root.to_path_buf();
    let mut leaf = NamePart {
        target: String::new(),
        attr: crate::name::ModeAttr::None,
        is_template: false,
    };
    let components: Vec<_> = rel.components().collect();
    for (i, component) in components.iter().enumerate() {
        let part = parse_component(&component.as_os_str().to_string_lossy());
        out.push(&part.target);
        if i + 1 == components.len() {
            leaf = part;
        }
    }
    (out, leaf)
}

/// A unified diff between the current bytes and the new body, when both are valid UTF-8.
fn unified_diff(current: &[u8], new: &Body) -> Option<String> {
    let (Ok(current), Body::Text(new)) = (std::str::from_utf8(current), new) else {
        return None;
    };
    let diff = similar::TextDiff::from_lines(current, new);
    Some(diff.unified_diff().header("current", "new").to_string())
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write `bytes` to `target` atomically: write a temp file in the same directory, then rename over
/// the target so a reader never sees a partial file.
fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), ApplyError> {
    let dir = target.parent().unwrap_or_else(|| Path::new("."));
    let stem = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".grim-tmp-{}-{n}-{stem}", std::process::id()));

    fs::write(&tmp, bytes).map_err(|source| ApplyError::Write {
        path: tmp.clone(),
        source,
    })?;
    fs::rename(&tmp, target).map_err(|source| {
        let _ = fs::remove_file(&tmp);
        ApplyError::Write {
            path: target.to_path_buf(),
            source,
        }
    })
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: Option<u32>) -> Result<(), ApplyError> {
    use std::os::unix::fs::PermissionsExt;
    if let Some(mode) = mode {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|source| {
            ApplyError::Write {
                path: path.to_path_buf(),
                source,
            }
        })?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: Option<u32>) -> Result<(), ApplyError> {
    Ok(())
}
