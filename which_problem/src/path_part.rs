use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

/// Represents pieces of a PATH
///
/// A PATH is broken up by a separator. Those parts can be either
/// absolute or relative.
///
/// This struct is intended to retain the original representation of the part.
/// It provides a normalized interface through the `absolute` property that
/// should account for relative PATH pieces.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathPart {
    /// Expanded and resolved absolute path
    pub(crate) absolute: PathBuf,

    /// Current working directory when PATH was accessed
    pub(crate) cwd: PathBuf,

    // The status of the current path part i.e. if it's an empty dir or not etc.
    pub(crate) state: PartState,

    /// Original part of the PATH
    pub(crate) original: PathBuf,

    relative: bool,
}

impl PartState {
    #[must_use]
    pub(crate) fn details(&self) -> String {
        match self {
            PartState::Valid => "Path part is a valid, non-empty, directory",
            PartState::NotDir => "Path part exists, but is a file. Must be a directory",
            PartState::Missing => "Path part does not exist exist on disk, no such directory",
            PartState::EmptyDir => "Path part directory exists, but it is empty",
        }
        .to_string()
    }
}

impl Display for PathPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.state;
        let path = &self.original;
        let cwd = &self.cwd;
        if let Some(width) = f.width() {
            write!(f, "[{:width$}] ", &format!("{}", self.state))?;
        } else {
            write!(f, "[{state}] ")?;
        }

        if self.relative {
            write!(f, "(relative from {cwd:?}) ")?;
        }
        write!(f, "{path:?}")?;

        Ok(())
    }
}

impl PathPart {
    #[must_use]
    pub(crate) fn new(cwd: &Path, original: &Path) -> Self {
        let cwd = cwd.to_path_buf();
        let original = original.to_path_buf();
        let relative = original.is_relative();
        let absolute = if relative {
            cwd.join(&original)
        } else {
            original.clone()
        };

        let state = part_state(&absolute);

        Self {
            absolute,
            cwd,
            state,
            original,
            relative,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PartState {
    /// No problems detected
    Valid,

    /// Path exists, but it's not a directory
    NotDir,

    /// Dir does not exist
    Missing,

    /// Dir exists, but there's no executable files in it
    EmptyDir,
}

impl Display for PartState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartState::EmptyDir => f.write_str("EMPTY"),
            PartState::Missing => f.write_str("MISSING"),
            PartState::NotDir => f.write_str("NOT DIR"),
            PartState::Valid => f.write_str("OK"),
        }
    }
}

fn any_files_in_dir(path: &Path) -> bool {
    if let Ok(read_dir) = std::fs::read_dir(path) {
        read_dir.filter_map(std::result::Result::ok).any(|_| true)
    } else {
        false
    }
}

fn part_state(path: &Path) -> PartState {
    if path.exists() {
        if path.is_dir() {
            if any_files_in_dir(path) {
                PartState::Valid
            } else {
                PartState::EmptyDir
            }
        } else {
            PartState::NotDir
        }
    } else {
        PartState::Missing
    }
}
