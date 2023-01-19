#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]

pub mod path_part;

use is_executable::IsExecutable;
use itertools::Itertools;
use path_part::PathPart;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{ffi::OsString, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhichProblem {
    /// The current working directory, affects PATHs with relative parts
    pub cwd: Option<PathBuf>,

    /// The program name you're trying to execute i.e. for `which cat` it would be "cat"
    pub program: OsString,

    /// The contents of PATH environment variable
    pub path_env: Option<OsString>,

    /// How many guesses to suggest if the command could not be found
    pub guess_limit: usize,
}

impl WhichProblem {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        let program = program.as_ref().into();
        Self {
            program,
            ..Self::default()
        }
    }
}

impl Default for WhichProblem {
    fn default() -> Self {
        Self {
            program: OsString::new(),
            path_env: std::env::var_os("PATH"),
            guess_limit: 1,
            cwd: None,
        }
    }
}

struct ResolvedWhich {
    program: OsString,
    path_parts: Vec<PathPart>,
    guess_limit: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum FileState {
    Missing,
    IsDir,
    NotExecutable,
    BadSymlink,
    Valid,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum SymlinkState {
    Missing,
    NotExecutable,
    IsDir,
    Valid,
}

fn symlink_state(path: &Path) -> SymlinkState {
    if let Some(link) = path
        .canonicalize() // Resolves symlink to path
        .ok()
    {
        match file_state(&link) {
            FileState::Missing => SymlinkState::Missing,
            FileState::IsDir => SymlinkState::IsDir,
            FileState::NotExecutable => SymlinkState::NotExecutable,
            FileState::BadSymlink => SymlinkState::Missing,
            FileState::Valid => SymlinkState::Valid,
        }
    } else {
        SymlinkState::Missing
    }
}

fn file_state(path: &Path) -> FileState {
    if path.is_symlink() {
        match symlink_state(&path) {
            SymlinkState::Valid => FileState::Valid,
            _ => FileState::BadSymlink,
        }
    } else if path.exists() {
        if path.is_dir() {
            FileState::IsDir
        } else {
            if path.is_executable() {
                FileState::Valid
            } else {
                FileState::NotExecutable
            }
        }
    } else {
        FileState::Missing
    }
}

fn suggest_spelling(
    program: &OsString,
    parts: &Vec<PathPart>,
    guess_limit: usize,
) -> Option<Vec<OsString>> {
    let mut heap = std::collections::BinaryHeap::new();
    let values = parts
        .iter()
        .filter_map(|p| std::fs::read_dir(&p.absolute).ok())
        .flat_map(|r| r.filter_map(|f| f.ok()).collect::<Vec<DirEntry>>())
        .map(|d| d.path())
        .filter_map(|p| p.file_name().and_then(|f| Some(f.to_os_string())))
        .unique()
        .map(|filename| {
            let score = strsim::normalized_levenshtein(
                &program.to_string_lossy(),
                &filename.to_string_lossy(),
            );

            (ordered_float::OrderedFloat(score), filename)
        })
        .collect::<Vec<(_, _)>>();

    for value in &values {
        heap.push(value);
    }

    if heap.is_empty() {
        None
    } else {
        Some(
            heap.iter()
                .take(guess_limit)
                .map(|(_, filename)| filename.clone())
                .collect::<Vec<OsString>>(),
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct WhichFileFound {
    pub bad_dirs: Vec<PathBuf>,
    pub valid_files: Vec<PathBuf>,
    pub bad_symlinks: Vec<PathBuf>,
    pub bad_executables: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub name: OsString,
    pub is_empty: bool,
    pub suggested: Option<Vec<OsString>>,
    pub path_parts: Vec<PathPart>,
    pub found_files: WhichFileFound,
    pub contains_whitespace: bool,
}

impl ResolvedWhich {
    fn check(&self) -> Program {
        let mut found_files = WhichFileFound::default();

        for path in self
            .path_parts
            .iter()
            .map(|p| p.absolute.join(&self.program))
        {
            match file_state(&path) {
                FileState::Missing => {}
                FileState::IsDir => found_files.bad_dirs.push(path.clone()),
                FileState::Valid => found_files.valid_files.push(path.clone()),
                FileState::BadSymlink => found_files.bad_symlinks.push(path.clone()),
                FileState::NotExecutable => found_files.bad_executables.push(path.clone()),
            }
        }

        let contains_whitespace = (self.program)
            .as_bytes()
            .iter()
            .any(u8::is_ascii_whitespace);

        Program {
            name: self.program.clone(),
            is_empty: self.program.is_empty(),
            suggested: suggest_spelling(&self.program, &self.path_parts, self.guess_limit),
            path_parts: self.path_parts.clone(),
            found_files,
            contains_whitespace,
        }
    }
}

impl WhichProblem {
    fn resolve(&self) -> Result<ResolvedWhich, std::io::Error> {
        let program = self.program.clone();
        let path_env = self.path_env.clone().unwrap_or_else(|| OsString::from(""));

        let cwd = match self.cwd.clone() {
            Some(path) => path,
            None => std::env::current_dir()?,
        };

        let path_parts = std::env::split_paths(&path_env.as_os_str())
            .map(|part| PathPart::new(&cwd, &part))
            .collect::<Vec<_>>();

        let guess_limit = self.guess_limit;

        Ok(ResolvedWhich {
            program,
            path_parts,
            guess_limit,
        })
    }

    /// # Errors
    ///
    /// - If the current directory cannot be determined
    pub fn diagnose(&self) -> Result<Program, std::io::Error> {
        let which = self.resolve()?;

        Ok(which.check())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use path_part::PathState;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn empty_program_name() {
        let program = WhichProblem {
            program: OsString::new(),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert!(program.is_empty);
    }

    #[test]
    fn check_whitespace_program() {
        let program = OsString::from("lol ");
        let program = WhichProblem {
            program: program.clone(),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert!(program.contains_whitespace)
    }

    fn make_executable(file: &Path) {
        let perms = std::fs::metadata(file).unwrap().permissions();
        let mode = perms.mode() | 0o111;
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    #[test]
    fn multiple_valid() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path().to_path_buf();
        let file = dir.join("haha");

        let tmp_dir_two = tempfile::tempdir().unwrap();
        let dir_two = tmp_dir_two.path().to_path_buf();
        let file_two = dir_two.join(file.file_name().unwrap());
        let program = OsString::from(file.file_name().unwrap());

        std::fs::write(&file, "contents").unwrap();
        std::fs::write(&file_two, "contents").unwrap();
        make_executable(&file);
        make_executable(&file_two);

        let program = WhichProblem {
            program: program,
            path_env: Some(vec![dir.as_os_str(), dir_two.as_os_str()].join(&OsString::from(":"))),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(vec![file, file_two], program.found_files.valid_files);
    }

    #[test]
    fn check_executable_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());

        std::fs::write(&file, "contents").unwrap();

        let program = WhichProblem {
            program: name.clone(),
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(vec![file.clone()], program.found_files.bad_executables);

        let perms = std::fs::metadata(&file).unwrap().permissions();
        let mode = perms.mode() | 0o111;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode)).unwrap();

        assert!(file.is_executable());

        let program = WhichProblem {
            program: name,
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert!(program.found_files.bad_executables.is_empty());
        assert_eq!(vec![file.clone()], program.found_files.valid_files);
    }

    #[test]
    fn check_symlink_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());

        std::os::unix::fs::symlink(dir.join("nope"), &file).unwrap();

        assert_eq!(FileState::BadSymlink, file_state(&file));

        let program = WhichProblem {
            program: name,
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(vec![file.clone()], program.found_files.bad_symlinks);
    }

    #[test]
    fn check_parts_are_dirs() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());
        let expected = dir.join("nope");

        std::fs::write(&expected, "lol").unwrap();
        let program = WhichProblem {
            program: name,
            path_env: Some(vec![expected.as_os_str(), dir.as_os_str()].join(&OsString::from(":"))),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert!(!program
            .path_parts
            .iter()
            .any(|p| p.state == PathState::Missing));
        assert!(program
            .path_parts
            .iter()
            .any(|p| p.state == PathState::NotDir));
    }

    #[test]
    fn check_path_parts_exist() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());
        let expected = dir.join("nope");

        let program = WhichProblem {
            program: name,
            path_env: Some(vec![expected.as_os_str(), dir.as_os_str()].join(&OsString::from(":"))),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert!(program
            .path_parts
            .iter()
            .any(|p| p.state == PathState::Missing));
        assert!(!program
            .path_parts
            .iter()
            .any(|p| p.state == PathState::NotDir));
    }

    #[test]
    fn check_suggested_spelling() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());

        let actual = dir.join("rofl");
        std::fs::write(&actual, "contents").unwrap();
        make_executable(&actual);

        let program = WhichProblem {
            program: name,
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            program.suggested.unwrap(),
            vec![actual.file_name().unwrap()]
        );

        assert_eq!(program.name, file.file_name().unwrap());
    }
}
