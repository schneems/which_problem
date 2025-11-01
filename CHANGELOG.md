# Changelog

## Unreleased

## 0.1.2

- Use `fsaccess` for executable check. [#12](https://github.com/schneems/which_problem/pull/12)

The `fsaccess` crate uses https://pubs.opengroup.org/onlinepubs/9699919799/functions/access.html on unix systems which effectively delegates the question of if it's executable or not to the OS. This handles edge cases like where a file might have executable permissions, but a parent directory does not. From https://github.com/schneems/path_facts/blob/3400c1020a074713bc72a7193d23cf1a5d8f4317/README.md:

> Permissions of a path depend not just on the permissions of the specific file/directory but also on other things, such as inherited permissions from parent directories.
>
> This means that to know the "effective" permissions of a file, you need to know the permissions of all its parent directories (we use the faccess crate for this)
>
> More permissions info at https://www.redhat.com/sysadmin/linux-file-permissions-explained and https://www.redhat.com/sysadmin/suid-sgid-sticky-bit

## 0.1.1

- Fix symlink not reported if target file is not executable [#3](https://github.com/schneems/which_problem/pull/3)

## 0.1.0

- First release
