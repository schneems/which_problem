#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]

pub mod path_part;
pub mod problem;

use is_executable::IsExecutable;
use itertools::Itertools;
use path_part::PathPart;
use problem::Problem;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::DirEntry;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{ffi::OsString, path::PathBuf};

/// Use it to add diagnostic info to running a command:
///
/// ```rust,no_run
/// use std::process::Command;
/// use which_problem::WhichProblem;
///
/// let program = "sh";
/// Command::new(program)
///     .arg("-c")
///     .arg("echo hello")
///     .output()
///     .map_err(|error| {
///        eprintln!("Executing command failed: #{program}");
///        eprintln!("Error: {error}");
///        eprintln!("Diagnostic info:");
///        eprintln!("{}", WhichProblem::new("cat").diagnose().unwrap_or_default());
///        error
///     })
///     .unwrap();
/// ```
///
/// Configure with custom options:
///
/// ```rust,no_run
/// use std::ffi::OsString;
/// use which_problem::WhichProblem;
///
/// WhichProblem {
///   program: OsString::from("cat"),
///   path_env: std::env::var_os("CUSTOM_VALUE"),
///   ..WhichProblem::default()
/// }.diagnose()
///  .unwrap()
///  .display();
/// ```
///

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

// Returns true if the symlink destination exists on disk
fn symlink_file_exists(path: &Path) -> bool {
    path.canonicalize() // Resolves symlink to path
        .ok()
        .map_or(false, |c| c.exists())
}

fn bad_symlinks(program: &OsString, parts: &Vec<PathPart>) -> Option<Vec<PathBuf>> {
    let found = parts
        .iter()
        .map(|p| p.absolute.join(&program))
        .filter(|file| file.is_symlink())
        .filter(|f| !symlink_file_exists(&f))
        .collect::<Vec<PathBuf>>();

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

fn find_files(program: &OsString, parts: &Vec<PathPart>) -> Option<Vec<PathBuf>> {
    let found = parts
        .iter()
        .map(|p| p.absolute.join(&program))
        .filter(|file| file.exists())
        .filter(|file| !file.is_dir())
        .collect::<Vec<PathBuf>>();

    if found.is_empty() {
        None
    } else {
        Some(found)
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

impl ResolvedWhich {
    fn check_program(&self, problems: &mut Vec<Problem>) {
        if self.program.is_empty() {
            problems.push(Problem::IsEmpty);
        }

        if (self.program)
            .as_bytes()
            .iter()
            .any(u8::is_ascii_whitespace)
        {
            problems.push(Problem::ContainsWhitespace(self.program.clone()));
        }

        // ## Symlinks
        if let Some(files) = bad_symlinks(&self.program, &self.path_parts) {
            problems.push(Problem::FoundFilesBadSymlink(files));
        }

        // ## Found files
        if let Some(files) = find_files(&self.program, &self.path_parts) {
            let (valid, cannot_execute): (Vec<_>, Vec<_>) = files
                .iter()
                .map(std::clone::Clone::clone)
                .partition(|f| f.is_executable());

            if !cannot_execute.is_empty() {
                problems.push(Problem::FoundFilesNotExecutable(cannot_execute));
            }

            if valid.len() > 1 {
                problems.push(Problem::FoundMultipleExecutables(valid));
            }
        } else {
            if let Some(guesses) =
                suggest_spelling(&self.program, &self.path_parts, self.guess_limit)
            {
                problems.push(Problem::NotFoundSuggestedSpelling(guesses))
            }
        }
    }

    fn check_path(&self, problems: &mut Vec<Problem>) {
        // PATH is empty and program is not a valid path
        if self.path_parts.is_empty() && !Into::<PathBuf>::into(&self.program).exists() {
            problems.push(Problem::PathIsTotallyEmpty);
        }

        // Parts exist, but are not directories
        if let Some(parts) = parts_not_dir(&self.path_parts) {
            problems.push(Problem::PathPiecesNotDir(parts));
        };

        // Parts do not exist on disk
        if let Some(parts) = parts_do_not_exist(&self.path_parts) {
            assert!(!parts.is_empty());
            problems.push(Problem::PathPiecesDoNotExist(parts));
        }
    }
}

fn parts_do_not_exist(parts: &Vec<PathPart>) -> Option<Vec<PathPart>> {
    let found = parts
        .iter()
        .filter(|p| !p.absolute.exists())
        .map(|p| p.clone())
        .collect::<Vec<PathPart>>();

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

fn parts_not_dir(parts: &Vec<PathPart>) -> Option<Vec<PathPart>> {
    let found = parts
        .iter()
        .filter(|p| p.absolute.exists())
        .filter(|p| !p.absolute.is_dir())
        .map(|p| p.clone())
        .collect::<Vec<PathPart>>();

    if found.is_empty() {
        None
    } else {
        Some(found)
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
    pub fn diagnose(&self) -> Result<Problems, std::io::Error> {
        let which = self.resolve()?;
        let mut problems = Vec::new();

        which.check_program(&mut problems);
        which.check_path(&mut problems);

        Ok(Problems { problems })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Problems {
    pub problems: Vec<Problem>,
}

impl Problems {
    pub fn iter(&self) -> <Problems as IntoIterator>::IntoIter {
        self.clone().into_iter()
    }

    pub fn display(&self) -> String {
        format!("{self}")
    }
}

impl IntoIterator for Problems {
    type Item = Problem;
    type IntoIter = <Vec<Problem> as IntoIterator>::IntoIter; // so that you don't have to write std::vec::IntoIter, which nobody remembers anyway

    fn into_iter(self) -> Self::IntoIter {
        self.problems.into_iter()
    }
}

impl Display for Problems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self
            .problems
            .iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join("\n");
        f.write_str(&data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn empty_program_name() {
        let problems = WhichProblem {
            program: OsString::new(),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            Some(Problem::IsEmpty),
            problems.iter().find(move |p| p == &Problem::IsEmpty)
        );
    }

    #[test]
    fn check_whitespace_program() {
        let program = OsString::from("lol ");
        let problems = WhichProblem {
            program: program.clone(),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            Some(Problem::ContainsWhitespace(program.clone())),
            problems
                .iter()
                .find(move |p| p == &Problem::ContainsWhitespace(program.clone()))
        );
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

        let problems = WhichProblem {
            program: program,
            path_env: Some(vec![dir.as_os_str(), dir_two.as_os_str()].join(&OsString::from(":"))),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            Some(vec![
                dir.join(file.file_name().unwrap()),
                dir_two.join(file.file_name().unwrap())
            ]),
            problems.iter().find_map(|f| match f {
                Problem::FoundMultipleExecutables(files) => Some(files),
                _ => None,
            })
        );
    }

    #[test]
    fn check_executable_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let program = OsString::from(file.file_name().unwrap());

        std::fs::write(&file, "contents").unwrap();

        let problems = WhichProblem {
            program: program.clone(),
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        let expected = Problem::FoundFilesNotExecutable(vec![file.clone()]);
        assert_eq!(
            Some(expected.clone()),
            problems.iter().find(move |p| p == &expected)
        );

        let perms = std::fs::metadata(&file).unwrap().permissions();
        let mode = perms.mode() | 0o111;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode)).unwrap();

        assert!(file.is_executable());

        let problems = WhichProblem {
            program: program,
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            None,
            problems.iter().find(|f| match f {
                Problem::FoundFilesNotExecutable(_) => true,
                _ => false,
            })
        );
    }

    #[test]
    fn check_symlink_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let program = OsString::from(file.file_name().unwrap());

        std::os::unix::fs::symlink(dir.join("nope"), &file).unwrap();

        let problems = WhichProblem {
            program: program,
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            Some(vec![file]),
            problems.iter().find_map(|f| match f {
                Problem::FoundFilesBadSymlink(files) => Some(files),
                _ => None,
            })
        );
    }
    #[test]
    fn check_parts_are_dirs() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let program = OsString::from(file.file_name().unwrap());
        let expected = dir.join("nope");

        std::fs::write(&expected, "lol").unwrap();
        let problems = WhichProblem {
            program: program,
            path_env: Some(vec![expected.as_os_str(), dir.as_os_str()].join(&OsString::from(":"))),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            Some(vec![expected]),
            problems.iter().find_map(|f| match f {
                Problem::PathPiecesNotDir(files) =>
                    Some(files.iter().map(|p| p.absolute.clone()).collect()),
                _ => None,
            })
        );
    }

    #[test]
    fn check_path_parts_exist() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let program = OsString::from(file.file_name().unwrap());
        let expected = dir.join("nope");

        let problems = WhichProblem {
            program: program,
            path_env: Some(vec![expected.as_os_str(), dir.as_os_str()].join(&OsString::from(":"))),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            Some(vec![expected]),
            problems.iter().find_map(|f| match f {
                Problem::PathPiecesDoNotExist(files) =>
                    Some(files.iter().map(|p| p.absolute.clone()).collect()),
                _ => None,
            })
        );
    }

    #[test]
    fn check_suggested_spelling() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let program = OsString::from(file.file_name().unwrap());

        let actual = dir.join("rofl");
        std::fs::write(&actual, "contents").unwrap();
        make_executable(&actual);

        let problems = WhichProblem {
            program: program,
            path_env: Some(dir.as_os_str().into()),
            ..WhichProblem::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            Some(vec![actual.file_name().unwrap().to_os_string()]),
            problems.iter().find_map(|f| match f {
                Problem::NotFoundSuggestedSpelling(files) => Some(files),
                _ => None,
            })
        );
    }
}
