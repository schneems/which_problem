use is_executable::IsExecutable;
use std::fmt::Display;
use std::path::Path;

pub(crate) fn file_state(path: &Path) -> FileState {
    if path.is_symlink() {
        match symlink_state(path) {
            SymlinkState::Valid => FileState::Valid,
            _ => FileState::BadSymlink,
        }
    } else if path.exists() {
        if path.is_dir() {
            FileState::IsDir
        } else if path.is_executable() {
            FileState::Valid
        } else {
            FileState::NotExecutable
        }
    } else {
        FileState::Missing
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum FileState {
    Valid,
    IsDir,
    Missing,
    BadSymlink,
    NotExecutable,
}

impl FileState {
    pub(crate) fn details(&self) -> String {
        match self {
            FileState::Valid => "File is valid and found on the PATH",
            FileState::IsDir => "Directory found, expecting a file",
            FileState::Missing => "No such file at this path",
            FileState::BadSymlink => "File containing a broken symlink found",
            FileState::NotExecutable => "File found but it does not have executable permissions",
        }
        .to_string()
    }
}

impl Display for FileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileState::Valid => f.write_str("OK"),
            FileState::IsDir => f.write_str("IS DIR"),
            FileState::Missing => f.write_str("MISSING"),
            FileState::BadSymlink => f.write_str("BAD SYM"),
            FileState::NotExecutable => f.write_str("NOT EXE"),
        }
    }
}

fn symlink_state(path: &Path) -> SymlinkState {
    if let Ok(link) = path.canonicalize()
    // Resolves symlink to path
    {
        match file_state(&link) {
            FileState::IsDir => SymlinkState::IsDir,
            FileState::Valid => SymlinkState::Valid,
            FileState::Missing | FileState::BadSymlink => SymlinkState::Missing,
            FileState::NotExecutable => SymlinkState::NotExecutable,
        }
    } else {
        SymlinkState::Missing
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum SymlinkState {
    IsDir,
    Valid,
    Missing,
    NotExecutable,
}
