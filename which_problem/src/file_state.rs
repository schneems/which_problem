use is_executable::IsExecutable;
use std::fmt::Display;
use std::path::Path;

/// Return the state of a file inside of a PATH directory
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

/// All the various states a file inside of a PATH directory
/// can hold.
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
            FileState::Valid => {
                "File found matching program name with executable permissions. Valid executable."
            }
            FileState::IsDir => {
                "Entry found matching program name, but is a directory. Executables must be a file"
            }
            FileState::Missing => "File not found at this path",
            FileState::BadSymlink => "File found matching program name, but is a broken symlink",
            FileState::NotExecutable => {
                "File found matching program name, but it does not have executable permissions"
            }
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
