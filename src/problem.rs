use crate::path_part::PathPart;
use std::{ffi::OsString, fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PlentyProblems(Vec<Problem>);

/// All known issues that could cause problems with invoking a program via `Command::new`
///
/// Ordered by severity
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Problem {
    /// Program name is empty i.e. `""`
    IsEmpty,

    /// PATH environment variable is empty or does not exist i.e. `export PATH=""`
    PathIsTotallyEmpty,

    /// Program name contains whitespace i.e. `"r uby"`
    ContainsWhitespace(OsString),

    /// Files that matched the given program name exist on disk but are not `chmod +x`
    FoundFilesNotExecutable(Vec<PathBuf>),

    /// Files that matched the given program name exist, but point to a broken symlink
    FoundFilesBadSymlink(Vec<PathBuf>),

    /// No exact matches were found, here's our best guesses
    NotFoundSuggestedSpelling(Vec<OsString>),

    /// More than one file exists that matches the given program name. Only the first is used
    FoundMultipleExecutables(Vec<PathBuf>),

    /// A part of the PATH points to a location that doesn't exist i.e. `export PATH="/usr/local/bin:/does/not/exist"
    PathPiecesDoNotExist(Vec<PathPart>),

    /// A part of the PATH points to a location that exists but is a file instead of a dir i.e. `export PATH="/usr/local/bin/which"
    PathPiecesNotDir(Vec<PathPart>),
}

impl Problem {
    #[must_use]
    pub fn header(&self) -> String {
        match self {
            Self::IsEmpty => String::from("Program cannot be empty"),
            Self::PathIsTotallyEmpty => String::from("Environment variable PATH is empty"),
            Self::ContainsWhitespace(name) => format!("Program contains whitespace: {name:?}."),
            Self::NotFoundSuggestedSpelling(_) => {
                String::from("Tip: These similarly named executables exist exist on PATH.")
            }
            Self::FoundFilesNotExecutable(_) => String::from(
                "Found program match(es) in PATH that do not have execution permission.",
            ),
            Self::FoundFilesBadSymlink(_) => {
                String::from("Found program matches in PATH that have invalid symlink(s).")
            }
            Self::FoundMultipleExecutables(_) => String::from(
                "Found multiple valid programsin PATH, ensure the one you expected is first.",
            ),
            Self::PathPiecesDoNotExist(_) => {
                String::from("Part(s) of the PATH do not exist on disk")
            }
            Self::PathPiecesNotDir(_) => {
                String::from("Part(s) of the PATH are file(s), expected to be directories.")
            }
        }
    }

    pub fn display(&self) -> String {
        format!("{self})")
    }
}

impl Display for &Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::NotFoundSuggestedSpelling(suggestions) => {
                f.write_str(&self.header())?;
                f.write_str("\n")?;
                f.write_str("\n")?;
                for suggestion in suggestions {
                    f.write_fmt(format_args!("  - '{}'\n", suggestion.to_string_lossy()))?;
                }
                Ok(())
            }
            Problem::FoundFilesNotExecutable(paths)
            | Problem::FoundFilesBadSymlink(paths)
            | Problem::FoundMultipleExecutables(paths) => {
                f.write_str(&self.header())?;
                f.write_str("\n")?;
                f.write_str("\n")?;
                for path in paths {
                    f.write_fmt(format_args!("  - '{}'\n", path.display()))?;
                }
                Ok(())
            }
            Problem::PathPiecesDoNotExist(parts) | Problem::PathPiecesNotDir(parts) => {
                let (relative, absolute): (Vec<_>, Vec<_>) =
                    parts.iter().partition(|p| p.is_relative());

                f.write_str(&self.header())?;
                f.write_str("\n")?;
                f.write_str("\n")?;
                for part in &absolute {
                    f.write_fmt(format_args!("- '{}'\n", part.original.display(),))?;
                }

                for part in &relative {
                    f.write_fmt(format_args!(
                        "- '{}' relative to '{}'\n",
                        part.original.display(),
                        part.cwd.display()
                    ))?;
                }
                Ok(())
            }
            Problem::IsEmpty | Problem::PathIsTotallyEmpty | Problem::ContainsWhitespace(_) => {
                f.write_str(&self.header())?;
                f.write_str("\n")
            } // Header contains all info
        }
    }
}
