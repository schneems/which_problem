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
    /// Current working directory when PATH was accessed
    pub cwd: PathBuf,

    /// Original part of the PATH
    pub original: PathBuf,

    /// Expanded and resolved absolute path
    pub absolute: PathBuf,

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

        Self {
            cwd,
            original,
            absolute,
            relative,
        }
    }

    #[must_use]
    pub fn is_relative(&self) -> bool {
        self.relative
    }
}
