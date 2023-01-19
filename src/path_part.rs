use std::path::{Path, PathBuf};

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
            cwd,
            state,
            original,
            absolute,
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
    EmptyDir,
    Missing,
    NotDir,
    Valid,
}

fn any_files_in_dir(path: &Path) -> bool {
    if let Some(read_dir) = std::fs::read_dir(&path).ok() {
        read_dir.filter_map(|read_dir| read_dir.ok()).any(|_| true)
    } else {
        false
    }
}

fn path_state(path: &Path) -> PathState {
    if path.exists() {
        if path.is_dir() {
            if !any_files_in_dir(&path) {
                PathState::EmptyDir
            } else {
                PathState::Valid
            }
        } else {
            PathState::NotDir
        }
    } else {
        PathState::Missing
    }
}
