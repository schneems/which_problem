## cargo-whichp

A CLI for dianosing executable lookup issues.

## Installation

```console
$ cargo install cargo-whichp
```

## Use

```console
$ cargo whichp bundle
Program "bundle" found at "/Users/rschneeman/.gem/ruby/3.1.3/bin/bundle"

Warning: Executables with the same name found on the PATH:
  > [OK] "/Users/rschneeman/.gem/ruby/3.1.3/bin/bundle"
  - [OK] "/Users/rschneeman/.rubies/ruby-3.1.3/bin/bundle"
  - [OK] "/usr/local/bin/bundle"
  - [OK] "/usr/local/bin/bundle"
  - [OK] "/usr/bin/bundle"
Help: Ensure the one you want comes first and is [OK]
Explanation of keys:
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
Explanation of keys:
    [OK     ] - Path part is a valid, non-empty, directory
    [MISSING] - Path part does not exist exist on disk, no such directory
```

For more options

```console
$ cargo whichp --help
```

## Dev execute

```console
$ cargo run -p cargo-whichp -- whichp --help
```
