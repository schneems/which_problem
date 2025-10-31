use crate::file_state::FileState;
use crate::path_part::PathPart;
use crate::path_with_state::PathWithState;
use itertools::Itertools;
use std::ffi::OsString;
use std::fmt::Display;
use std::fmt::Write;
use std::os::unix::ffi::OsStrExt;

/// Holds the results of a `Which::diagnose` call
///
/// The main interface is intended to output diagnostic
/// information to an end user.
///
/// See the `Display` implementation.
#[derive(Clone, Debug, Default)]
pub struct Program {
    pub(crate) name: OsString,
    pub(crate) suggested: Option<Vec<OsString>>,
    pub(crate) path_parts: Vec<PathPart>,
    pub(crate) found_files: Vec<PathWithState>,
}

pub(crate) fn contains_whitespace(name: &OsString) -> bool {
    (name).as_bytes().iter().any(u8::is_ascii_whitespace)
}

impl Display for Program {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Program {
            name,
            suggested,
            path_parts,
            found_files,
        } = &self;

        let executable = found_files
            .iter()
            .find(|p| matches!(p.state, FileState::Valid));

        let file_state_width = found_files
            .iter()
            .map(|p| format!("{}", p.state).len())
            .max()
            .unwrap_or_default();

        let part_width = self
            .path_parts
            .iter()
            .map(|part| format!("{}", part.state).len())
            .max()
            .unwrap_or(0);

        let name = name.display();

        // Found/Not-found
        if let Some(found) = executable {
            let file = &found.path.display();
            writeln!(f, r"Program {name:?} found at {file:?}")?;
        } else if let Some(found) = found_files
            .iter()
            .find(|p| matches!(p.state, FileState::NotExecutable))
        {
            let file = found.path.display();
            writeln!(
                f,
                "Program {name:?} found at {file:?} but is not executable",
            )?;
        } else {
            writeln!(f, r"Program {name:?} not found")?;

            if self.name.is_empty() {
                writeln!(f, "Warning: Program is blank")?;
            }
            if contains_whitespace(&self.name) {
                writeln!(f, "Warning: Program contains whitespace")?;
            }
        }
        f.write_char('\n')?;

        // Files in order they were found
        if found_files.len() > 1 {
            f.write_str("Warning: Executables with the same name found on the PATH:\n")?;
            for path in found_files {
                write!(f, "  ")?;
                if executable
                    .map(|found| &found.path)
                    .and_then(|p| (p == &path.path).then_some(()))
                    .is_some()
                {
                    write!(f, "> ")?;
                } else {
                    write!(f, "- ")?;
                }

                writeln!(f, "{path:file_state_width$}")?;
            }
            writeln!(
                f,
                "Help: Ensure the one you want comes first and is [{valid:file_state_width$}]",
                valid = FileState::Valid
            )?;
            f.write_str("Explanation:\n")?;
            for state in found_files.iter().map(|p| p.state.clone()).unique() {
                let details = state.details();
                writeln!(
                    f,
                    "    [{:file_state_width$}] - {details}",
                    &format!("{state}")
                )?;
            }
            f.write_char('\n')?;
        } else {
            f.write_str("Info: No other executables with the same name are found on the PATH\n")?;
            f.write_char('\n')?;
        }
        // Suggestions
        writeln!(
            f,
            "Info: These executables have the closest spelling to {name:?} but did not match:"
        )?;
        f.write_str("      ")?;

        if let Some(suggested) = suggested {
            let out = suggested
                .iter()
                .map(|s| format!(r#""{}""#, s.display()))
                .collect::<Vec<String>>()
                .join(", ");

            writeln!(f, "{out}")?;
            f.write_char('\n')?;
        }

        // PATH parts
        if path_parts.is_empty() {
            f.write_str("Warning: The PATH is empty\n")?;
        } else {
            f.write_str(
                "Info: The following directories on PATH were searched (top to bottom):\n",
            )?;
            for part in path_parts {
                write!(f, "  ")?;
                if executable
                    .map(|found| &found.path)
                    .and_then(|p| p.parent())
                    .and_then(|parent| (parent == part.absolute).then_some(()))
                    .is_some()
                {
                    write!(f, "> ")?;
                } else {
                    write!(f, "- ")?;
                }

                writeln!(f, "{part:part_width$}")?;
            }
            f.write_str("Explanation:\n")?;
            for state in path_parts.iter().map(|p| p.state.clone()).unique() {
                let details = state.details();
                writeln!(f, "    [{:part_width$}] - {details}", &format!("{state}"))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_program_name() {
        assert!(OsString::new().is_empty());
    }

    #[test]
    fn check_whitespace_program() {
        assert!(contains_whitespace(&OsString::from("lol ")));
    }
}
