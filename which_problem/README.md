## Which Problem (rust)

Tools to help you find problems when looking up executable files.

## Cli

Use the CLI to dianose problems:

```console
$ cargo install cargo-whichp
```

```
$ cargo whichp bundle

Program "bundle" found at "/Users/rschneeman/.gem/ruby/3.1.3/bin/bundle"

Warning: Executables with the same name found on the PATH:
  > [OK] "/Users/rschneeman/.gem/ruby/3.1.3/bin/bundle"
  - [OK] "/Users/rschneeman/.rubies/ruby-3.1.3/bin/bundle"
  - [OK] "/usr/local/bin/bundle"
  - [OK] "/usr/local/bin/bundle"
  - [OK] "/usr/bin/bundle"
Help: Ensure the one you want comes first and is [OK]
Explanation:
    [OK] - File found matching program name with executable permissions. Valid executable.

Info: These executables have the closest spelling to "bundle" but did not match:
      "uname", "bundler", "uuname"

Info: The following directories on PATH were searched (top to bottom):
  > [OK     ] "/Users/rschneeman/.gem/ruby/3.1.3/bin"
  - [MISSING] "/Users/rschneeman/.rubies/ruby-3.1.3/lib/ruby/gems/3.1.0/bin"
  - [OK     ] "/Users/rschneeman/.rubies/ruby-3.1.3/bin"
  - [OK     ] "/Users/rschneeman/.cargo/bin"
  - [OK     ] "/usr/local/bin"
  - [OK     ] "/usr/local/sbin"
  - [OK     ] "/usr/local/bin"
  - [OK     ] "/System/Cryptexes/App/usr/bin"
  - [OK     ] "/usr/bin"
  - [OK     ] "/bin"
  - [OK     ] "/usr/sbin"
  - [OK     ] "/sbin"
  - [OK     ] "/Users/rschneeman/.cargo/bin"
Explanation:
    [OK     ] - Path part is a valid, non-empty, directory
    [MISSING] - Path part does not exist exist on disk, no such directory
```

## Rust library

```console
$ cargo add which_problem
```

```rust,no_run
use std::process::Command;
use which_problem::Which;

let program = "bundle";
Command::new(program)
    .arg("install")
    .output()
    .map_err(|error| {
        eprintln!("Executing command '{program}' failed. Error: {error}");

        match Which::new(program).diagnose() {
            Ok(details) => println!("Diagnostic info: {details}"),
            Err(error) => println!("Warning: Internal which_problem error: {error}"),
        }
        error
    })
    .unwrap();
```
