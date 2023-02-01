use crate::file_state::FileState;
use crate::path_part::PathPart;
use crate::path_with_state::PathWithState;
use crate::program::Program;
use crate::suggest;
use std::ffi::OsStr;
use std::{ffi::OsString, path::PathBuf};

/// Find problems with executable lookup
///
/// Example:
///
/// ```rust,no_run
/// use which_problem::Which;
///
/// let problems = Which::new("bundle").diagnose().unwrap();
/// eprintln!("{problems}");
/// ```
///
/// Selectively configure fields using `Which::default()`:
///
/// ```rust
/// use std::ffi::OsString;
/// use which_problem::Which;
///
/// let which = Which {
///     program: OsString::from("bundle"),
///     guess_limit: 5,
///     path_env: Some(OsString::from("alternate:path:here")),
///     ..Which::default()
/// };
/// eprintln!("{}", which.diagnose().unwrap());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Which {
    /// The current working directory, affects PATHs with relative parts
    pub cwd: Option<PathBuf>,

    /// The program name you're trying to execute i.e.
    /// for `bundle install` would be "bundle".
    pub program: OsString,

    /// The contents of PATH environment variable
    /// i.e. OsString::new("different:path:here")
    pub path_env: Option<OsString>,

    /// How many guesses to suggest if the command could not be found
    /// set to 0 to disable.
    pub guess_limit: usize,
}

impl Which {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        let program = program.as_ref().into();
        Self {
            program,
            ..Self::default()
        }
    }

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

impl Default for Which {
    fn default() -> Self {
        Self {
            program: OsString::new(),
            path_env: std::env::var_os("PATH"),
            guess_limit: 3,
            cwd: None,
        }
    }
}

struct ResolvedWhich {
    program: OsString,
    path_parts: Vec<PathPart>,
    guess_limit: usize,
}

impl ResolvedWhich {
    fn check(&self) -> Program {
        Program {
            name: self.program.clone(),
            suggested: suggest::spelling(&self.program, &self.path_parts, self.guess_limit),
            path_parts: self.path_parts.clone(),
            found_files: files_on_path(&self.program, &self.path_parts),
        }
    }
}

fn files_on_path(name: &OsString, path_parts: &[PathPart]) -> Vec<PathWithState> {
    path_parts
        .iter()
        .map(|p| p.absolute.join(name))
        .map(PathWithState::new)
        .filter(|p| !matches!(p.state, FileState::Missing))
        .collect()
}
