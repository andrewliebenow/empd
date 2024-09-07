# `empd` ("empty")

`empd` is a tool for checking if a file or directory is empty, or if a symbolic link points to a non-existent path. I use it instead of paranoidly checking paths with `file`, `ls`, `stat`, etc. before deleting them.

`empd` has only been tested on Linux.

## Installation

```Shell
# TODO Publish to crates.io
cargo install --git https://github.com/andrewliebenow/empd
```

## Usage

If and only if the path passed to `empd` is an empty directory, empty file, or a symbolic link that points to a non-existent path, `empd` will terminate with an exit code of 0.

If the `-d`/`--delete-if-empty` flag is used, the file, directory, or symbolic link will be deleted IF it is empty/points to non-existent path AND confirmation is given at an interactive prompt. As such, `empd -d`/`empd --delete-if-empty` should be reasonably safe to use (i.e. can never delete a non-empty directory or file). However, no file locking is performed, so if the path is modified while the confirmation prompt is waiting for input, a non-empty file or directory could be deleted.

(Actual terminal output is colorized.)

```Shell
❯ empd --help
Checks if a directory or file is empty, or if a symbolic link points to a path that does not exist. Only supports UTF-8 paths

Usage: empd [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to test

Options:
  -d, --delete-if-empty  Delete the file or directory if it is empty
  -h, --help             Print help
  -V, --version          Print version
```

```Shell
❯ cd /mnt && empd .
Canonicalized input path "." to "/mnt"
 🗙  Path "/mnt" is a non-empty directory (directories: 24, files: 0, symlinks: 0, total items: 24)
Exiting with non-zero exit code 31
```

```Shell
❯ sudo touch /blankfile && cd / && empd blankfile
Canonicalized input path "blankfile" to "/blankfile"
 ✔  Path "/blankfile" is an empty file
```

```Shell
❯ ln -s ./does-not-exist ./new-symbolic-link && empd ./new-symbolic-link
Could not canonicalize input path "./new-symbolic-link" because it or the file it resolves to does not exist
 ✔  Path "./new-symbolic-link" (non-canonicalized) is a symbolic link to non-existent file "./does-not-exist" (non-canonicalized)
```

## License

MIT License, see <a href="LICENSE">LICENSE</a> file
