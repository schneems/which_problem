use crate::file_state::{file_state, FileState};
use core::fmt::Display;
use std::path::PathBuf;

/// Represents a file on disk inside of a PATH directory
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PathWithState {
    pub(crate) path: PathBuf,
    pub(crate) state: FileState,
}

impl PathWithState {
    pub(crate) fn new(path: PathBuf) -> Self {
        let state = file_state(&path);
        PathWithState { path, state }
    }
}

impl Display for PathWithState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.state;
        let path = &self.path;
        if let Some(width) = f.width() {
            write!(f, "[{:width$}] {path:?}", &format!("{}", self.state))?;
        } else {
            write!(f, "[{state}] {path:?}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_with_state_display() {
        let p = PathWithState {
            path: PathBuf::from("/lol"),
            state: FileState::Valid,
        };

        assert_eq!(r#"[OK        ] "/lol""#, &format!("{p:width$}", width = 10));
    }
}
