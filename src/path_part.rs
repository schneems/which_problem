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
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct PathPart {
    /// Expanded and resolved absolute path
    pub absolute: PathBuf,

    /// Current working directory when PATH was accessed
    pub cwd: PathBuf,

    // The status of the current path part i.e. if it's an empty dir or not etc.
    pub state: PathState,

    /// Original part of the PATH
    pub original: PathBuf,

    relative: bool,
}

impl PathPart {
    #[must_use]
    pub fn new(cwd: &Path, original: &Path) -> Self {
        let cwd = cwd.to_path_buf();
        let original = original.to_path_buf();
        let relative = original.is_relative();
        let absolute = if relative {
            cwd.join(&original)
        } else {
            original.clone()
        };

        let state = path_state(&absolute);

        Self {
            absolute,
            cwd,
            state,
            original,
            relative,
        }
    }

    #[must_use]
    pub fn is_relative(&self) -> bool {
        self.relative
    }

    // fn is_empty_dir(&self) -> bool {
    //     files_iter(&self.absolute).and_then(|iter| iter.any(|file| file.exist()))
    // }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PathState {
    /// Dir exists, but there's no executable files in it
    EmptyDir,

    /// Dir does not exist
    Missing,

    /// Path exists, but it's not a directory
    NotDir,

    /// No problems detected
    Valid,
}

impl Display for PathState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathState::EmptyDir => f.write_str("EMPTY  "),
            PathState::Missing => f.write_str("MISSING"),
            PathState::NotDir => f.write_str("NOT DIR"),
            PathState::Valid => f.write_str("OK    "),
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

fn path_state(path: &Path) -> PathState {
    if path.exists() {
        if path.is_dir() {
            if any_files_in_dir(path) {
                PathState::Valid
            } else {
                PathState::EmptyDir
            }
        } else {
            PathState::NotDir
        }
    } else {
        PathState::Missing
    }
}
