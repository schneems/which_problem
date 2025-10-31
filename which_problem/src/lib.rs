#![doc = include_str!("../README.md")]
#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]

/// Get detailed diagnostic information about executable lookups
///
/// Example:
///
/// ```rust,no_run
/// use std::process::Command;
/// use which_problem::Which;
///
/// let program = "bundle";
/// Command::new(program)
///     .arg("install")
///     .output()
///     .map_err(|error| {
///         eprintln!("Executing command '{program}' failed. Error: {error}");
///
///         match Which::new(program).diagnose() {
///             Ok(details) => println!("Diagnostic info: {details}"),
///             Err(error) => println!("Warning: Internal which_problem error: {error}"),
///         }
///         error
///     })
///     .unwrap();
/// ```
mod file_state;
mod path_part;
mod path_with_state;
mod program;
mod suggest;
mod which;

// Primary input interface
pub use crate::which::Which;

// Primary output interface
pub use crate::program::Program;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_state::{file_state, FileState};
    use crate::path_with_state::PathWithState;
    use crate::which::Which;
    use is_executable::IsExecutable;
    use path_part::PartState;
    use std::ffi::OsString;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

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

        let program = Which {
            program,
            path_env: Some([dir.as_os_str(), dir_two.as_os_str()].join(&OsString::from(":"))),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            vec![
                PathWithState {
                    path: file,
                    state: FileState::Valid,
                },
                PathWithState {
                    path: file_two,
                    state: FileState::Valid
                }
            ],
            program.found_files
        );
    }

    #[test]
    fn check_executable_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());

        std::fs::write(&file, "contents").unwrap();

        let program = Which {
            program: name.clone(),
            path_env: Some(dir.as_os_str().into()),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            vec![PathWithState {
                path: file.clone(),
                state: FileState::NotExecutable
            }],
            program.found_files
        );

        let perms = std::fs::metadata(&file).unwrap().permissions();
        let mode = perms.mode() | 0o111;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode)).unwrap();

        assert!(file.is_executable());

        let program = Which {
            program: name,
            path_env: Some(dir.as_os_str().into()),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            vec![PathWithState {
                path: file,
                state: FileState::Valid
            }],
            program.found_files
        );
    }

    #[test]
    fn check_symlink_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());

        std::os::unix::fs::symlink(dir.join("nope"), &file).unwrap();

        assert_eq!(FileState::BadSymlink, file_state(&file));

        let program = Which {
            program: name,
            path_env: Some(dir.as_os_str().into()),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            vec![PathWithState {
                path: file,
                state: FileState::BadSymlink
            }],
            program.found_files
        );
    }

    #[test]
    fn check_symlink_matching_program_name_is_reported_if_pointing_to_non_executable_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        // Create a symlink named "yarn" that points to "yarn-3.10.0.cjs"
        let symlink_path = dir.join("yarn");
        let target_executable = dir.join("yarn-3.10.0.cjs");
        let program_name = OsString::from("yarn");

        // Create the target executable file
        std::fs::write(&target_executable, "#!/usr/bin/env node\n").unwrap();

        // Create symlink pointing to the differently named executable
        std::os::unix::fs::symlink(&target_executable, &symlink_path).unwrap();

        let program = Which {
            program: program_name.clone(),
            path_env: Some(dir.as_os_str().into()),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(program.found_files.len(), 1);
        assert_eq!(program.found_files[0].path, symlink_path);
        assert_eq!(program.found_files[0].state, FileState::NotExecutable);
        assert!(program.to_string().contains(&format!(
            "Program \"yarn\" found at \"{}\" but is not executable",
            symlink_path.display()
        )));
    }

    #[test]
    fn check_parts_are_dirs() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());
        let expected = dir.join("nope");

        std::fs::write(&expected, "lol").unwrap();
        let program = Which {
            program: name,
            path_env: Some([expected.as_os_str(), dir.as_os_str()].join(&OsString::from(":"))),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert!(!program
            .path_parts
            .iter()
            .any(|p| p.state == PartState::Missing));
        assert!(program
            .path_parts
            .iter()
            .any(|p| p.state == PartState::NotDir));
    }

    #[test]
    fn check_path_parts_exist() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let dir = tmp_dir.path();
        let file = dir.join("lol");
        let name = OsString::from(file.file_name().unwrap());
        let expected = dir.join("nope");

        let program = Which {
            program: name,
            path_env: Some([expected.as_os_str(), dir.as_os_str()].join(&OsString::from(":"))),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert!(program
            .path_parts
            .iter()
            .any(|p| p.state == PartState::Missing));
        assert!(!program
            .path_parts
            .iter()
            .any(|p| p.state == PartState::NotDir));
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

        let program = Which {
            program: name,
            path_env: Some(dir.as_os_str().into()),
            ..Which::default()
        }
        .diagnose()
        .unwrap();

        assert_eq!(
            program.suggested.clone().unwrap(),
            vec![actual.file_name().unwrap()]
        );

        assert_eq!(program.name, file.file_name().unwrap());
    }
}
