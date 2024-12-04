# whitespace-format

[![Build, lint and test](https://github.com/DavidPal/whitespace-format-rust/actions/workflows/build.yaml/badge.svg)](https://github.com/DavidPal/whitespace-format-rust/actions/workflows/build.yaml)

Whitespace formatter and linter for text files and source code files.

The purpose of this tool is to normalize source code files (e.g. Python, Java,
C/C++, Rust, Ruby, Go, JavaScript, etc.) and text files (HTML, JSON, YAML, CSV,
MarkDown, LaTeX) before they are checked into a version control system.

The features include:

* Auto-detection of new line markers (Linux `\n`, Windows `\r\n`, Mac `\r`).
* Add a new line marker at the end of the file if it is missing.
* Make new line markers consistent.
* Remove empty lines at the end of the file.
* Remove whitespace at the end of each line.
* Replace tabs with spaces.
* Remove/replace non-standard whitespace characters.

The formatting changes are
[idempotent](https://en.wikipedia.org/wiki/Idempotence), i.e., running the tool
second time (with the same parameters) has no effect.

Currently, the tool assumes that the files are encoded in either
[UTF-8](https://en.wikipedia.org/wiki/UTF-8),
[ASCII](https://en.wikipedia.org/wiki/ASCII) or [Extended
ASCII](https://en.wikipedia.org/wiki/Extended_ASCII). The results on other
types of files are undefined.

The tool is implemented in Rust. This ensures the necessary speed to handle
many and/or large files. For example, formatting the whole [Linux
Kernel](https://github.com/torvalds/linux) code base (1.4 GB, 87k files, 4k
modified files) on a modern computer takes less than 3 seconds.

## Installation

### Installation using `cargo`

The package is published to
[crates.io](https://crates.io/crates/whitespace-format).  If you have `cargo`
installed, you can install the package by running:

```shell
cargo install whitespace-format
```

### Debian package

If you are using a Debian-based system (Ubuntu, Debian, Mint, etc.), you can
download the Debian package from the [release
page](https://github.com/DavidPal/whitespace-format-rust/releases) and install
it using `dpkg` with the command:
```shell
sudo dpkg --install whitespace-format*.deb
```

## Usage

A sample command that formats source code files:
```shell
whitespace-format \
    --exclude ".git/|.idea/|.pyc$" \
    --new-line-marker=linux \
    --normalize-new-line-markers \
    foo.txt  my_project/
```
The command above formats `foo.txt` and all files contained in `my_project/`
directory and its subdirectories. Files that contain `.git/` or `.idea/` in
their (relative) path are excluded. For example, files in `my_project/.git/`
and files in `my_project/.idea/` are excluded. Likewise, files ending with
`*.pyc` are excluded.

If you want to know only if any changes **would be** made, add `--check-only`
option:
```shell
whitespace-format \
    --check-only \
    --exclude ".git/|.idea/|.pyc$" \
    --new-line-marker=linux \
    --normalize-new-line-markers \
    foo.txt  my_project/
```
This command can be used as a validation step before checking the source files
into a version control system. The command outputs a non-zero exit code if any
of the files would be formatted.

### Options

* `--check-only` -- Do not format files. Only report which files would be formatted.
  Exit code is zero if input is formatted correctly. Exit code is non-zero if formatting is required.
* `--follow-symlinks` -- Follow symbolic links when searching for files.
* `--exclude=REGEX` -- Regular expression that specifies which files to exclude.
  The regular expression is evaluated on the path of each file.
* `--color=MODE` -- This options specifies color output:
    * `auto` -- Determine whether to enable color output automatically based on the terminal used.
    * `on` -- Turn on color output.
    * `off` -- Turn off color output.

### Formatting options

* `--add-new-line-marker-at-end-of-file` -- Add a new line marker at the end of the file if it is missing.
* `--remove-new-line-marker-from-end-of-file` -- Remove all new line marker(s) from the end of each file.
  This option conflicts with `--add-new-line-marker-at-end-of-file`.
  This option implies `--remove-trailing-empty-lines` option, i.e., all empty lines at the end of the file are removed.
* `--normalize-new-line-markers` -- Make new line markers consistent in each file
  by replacing `\r\n`, `\n`, and `\r` with a consistent new line marker.
* `--remove-trailing-whitespace` -- Remove whitespace at the end of each line.
* `--remove-trailing-empty-lines` -- Remove empty lines at the end of each file.
* `--new-line-marker=MARKER` -- This option specifies what new line marker to use.
  `MARKER` must be one of the following:
    * `auto` -- Use new line marker that is the most common in each individual file.
      If no new line marker is present in the file, Linux `\n` is used.
      This is the default option.
    * `linux` -- Linux new line marker `\n`.
    * `mac` -- Mac new line marker `\r`.
    * `windows` -- Windows new line marker `\r\n`.

Note that input files can contain an arbitrary mix of new line markers `\n`,
`\r`, `\r\n` even within the same file. The option `--new-line-marker`
specifies the character that will be written in the formatted file.

An opinionated combination of options is:
```shell
whitespace-format \
    --new-line-marker=linux \
    --add-new-line-marker-at-end-of-file \
    --normalize-new-line-markers \
    --remove-trailing-whitespace \
    --remove-trailing-empty-lines \
    foo.txt  my_project/
```
This should work well for common programming languages (e.g. Python, Java,
C/C++, Rust, Ruby, Go, JavaScript, etc.) and common text file formats (HTML,
JSON, YAML, CSV, MarkDown, LaTeX).

### Empty files

There are separate options for handling empty files and files consisting of
whitespace characters only:

* `--normalize-empty-files=MODE` -- Replace files of zero length.
* `--normalize-whitespace-only-files=MODE` -- Replace files consisting of whitespace only.

The value `MODE` is one of the following:

* `ignore` -- Leave the file as is. This is the default option.
* `empty` -- Replace the file with an empty file.
* `one-line` -- Replace the file with a file consisting of a single new line marker.

Note that `--normalize-empty-files=ignore` and `--normalize-empty-files=empty` are equivalent.

The combination `--normalize-whitespace-only-files=empty` and
`--normalize-empty-files=one-line` is not allowed, since it would lead to
behavior that is not idempotent.

An opinionated combination for these two options is:
```shell
whitespace-format \
    ...
    --normalize-empty-files=empty \
    --normalize-whitespace-only-files=empty
```

### Special characters

Additional options are available for handling tab (`\t`), vertical tab (`\v`),
and form feed (`f`) characters:

* `--replace-tabs-with-spaces=N` -- Replace tabs with spaces.
  The value `N` is the number of spaces used to replace each tab character.
  If `N` is zero, tab characters are removed. If `N` is negative, tabs are not
  replaced. Default value is `-1`, i.e., tabs are not replaced.

* `--normalize-non-standard-whitespace=MODE` -- Replace or remove
  non-standard whitespace characters (`\v` and `\f`). `MODE` must be one of the following:
    * `ignore` -- Leave `\v` and `\f` as is. This is the default option.
    * `replace` -- Replace any occurrence of `\v` or `\f` with a single space.
    * `remove` -- Remove all occurrences of `\v` and `\f`.

## License

[MIT](LICENSE)
