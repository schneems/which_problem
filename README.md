# WhichProblem

This crate helps you find an executable when there's a problem i.e. `which <name>` fails.

## System support

- std::os::unix only (for now)

## Use

Use it to add diagnostic info to running a command:

```rust,no_run
use std::process::Command;
use which_problem::WhichProblem;

let program = "sh";
Command::new(program)
    .arg("-c")
    .arg("echo hello")
    .output()
    .map_err(|error| {
       eprintln!("Executing command failed: #{program}");
       eprintln!("Error: {error}");
       eprintln!("Diagnostic info:");
       eprintln!("{}", WhichProblem::new("cat").diagnose().unwrap_or_default());
       error
    })
    .unwrap();
```

Configure with custom options:

```rust,no_run
use std::ffi::OsString;
use which_problem::WhichProblem;

WhichProblem {
  program: OsString::from("cat"),
  path_env: std::env::var_os("CUSTOM_VALUE"),
  ..WhichProblem::default()
}.diagnose()
 .unwrap()
 .display();
```

## Cases

Here's a list of the known cases that can cause `which` to fail.

-Program name is empty i.e. `""`
-PATH environment variable is empty or does not exist i.e. `export PATH=""`
-Program name contains whitespace i.e. `"r uby"`
-Files that matched the given program name exist on disk but are not `chmod +x`
-Files that matched the given program name exist, but point to a broken symlink
-No exact matches were found, here's our best guesses. (Only supports UTF-8 programs)
-More than one file exists that matches the given program name. Only the first is used
-A part of the PATH points to a location that doesn't exist i.e. `export PATH="/usr/local/bin:/does/not/exist"
-A part of the PATH points to a location that exists but is a file instead of a dir i.e. `export PATH="/usr/local/bin/which"
