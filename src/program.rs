use crate::file_state::FileState;
use crate::path_part::PathPart;
use crate::path_with_state::PathWithState;
use itertools::Itertools;
use std::ffi::OsString;
use std::fmt::Display;
use std::fmt::Write;
use std::os::unix::ffi::OsStrExt;

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

        let name = format!("{name:?}");

        // Found/Not-found
        if let Some(found) = executable {
            let file = found.path.display();
            writeln!(f, r#"Program '{name}' found '{file}'"#)?;
        } else {
            writeln!(f, r#"Program '{name}' not found"#)?;

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
                writeln!(f, "  - {path:file_state_width$}")?;
            }
            f.write_char('\n')?;
            writeln!(
                f,
                "Help: Ensure the one you want comes first and is [{valid:file_state_width$}]'",
                valid = FileState::Valid
            )?;
            f.write_str("Explanation of keys:\n")?;
            for state in found_files.iter().map(|p| p.state.clone()).unique() {
                let details = state.details();
                writeln!(f, "  [{state:file_state_width$}] - {details}'\n")?;
            }
            f.write_char('\n')?;
        } else {
            f.write_str("Info: No other executables with the same name are found on the PATH\n")?;
            f.write_char('\n')?;
        }
        // Suggestions
        writeln!(
            f,
            "Info: These executables have the closest spelling to {name} but did not match:"
        )?;
        f.write_str("      ")?;
        let mut suggested = suggested.iter().peekable();
        while let Some(guess) = suggested.next() {
            write!(f, "'{guess:?}'")?;
            if suggested.peek().is_some() {
                f.write_str(", ")?;
            }
        }
        f.write_char('\n')?;

        // PATH parts
        if path_parts.is_empty() {
            f.write_str("Warning: The PATH is empty\n")?;
        } else {
            f.write_str(
                "Info: The following directories on PATH were searched (top to bottom):\n",
            )?;
            for part in path_parts {
                writeln!(f, "  - {part:part_width$}")?;
            }
            f.write_str("Explanation of keys:\n")?;
            for state in path_parts.iter().map(|p| p.state.clone()).unique() {
                let details = state.details();
                writeln!(f, "  [{state:part_width$}] - {details}'")?;
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
