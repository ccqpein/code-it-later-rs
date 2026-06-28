# README #

[![Crates.io](https://img.shields.io/crates/v/code-it-later-rs.svg)](https://crates.io/crates/code-it-later-rs)

- [Summary](#summary)
- [Install](#install)
  - [Installing the Emacs Interface Mode](#installing-the-emacs-interface-mode)
- [Features](#features)
- [Usage](#usage)
  - [Multi-line](#multi-line)
  - [Filter keyword](#filter-keyword)
  - [Ignore with keyword](#ignore-with-keyword)
  - [Excluding some folder](#excluding-some-folder)
  - [Expand json file](#expand-json-file)
  - [Local arguments](#local-arguments)
  - [Delete the crumbs](#delete-the-crumbs)
  - [Restore the crumbs](#restore-the-crumbs)
  - [Run format after clean the crumbs](#run-format-after-clean-the-crumbs)
  - [Output to different format of files](#output-to-different-format-of-files)
  - [Output the range of context](#output-the-range-of-context)
  - [Google Antigravity Integration](#google-antigravity-integration)

**Other Versions (have some features different):**

+ [code_it_later_ml](https://github.com/ccqpein/code_it_later_ml) ocaml version
+ [code-it-later](https://github.com/ccqpein/code-it-later) clojure version
+ [code_it_later_hs](https://github.com/ccqpein/code_it_later_hs) haskell version

## Summary ##

Create flags in your source code where there may be problems or potential optimizations. `codeitlater` helps you track these flags (crumbs) and fix them in the future.

## Install ##

`cargo install code-it-later-rs`

### Installing the Emacs Interface Mode ###

I developed the [code-it-later-mode](https://github.com/ccqpein/code-it-later-mode), which serves as an interface mode for Emacs.

## Features ##

* Extract crumbs (comments containing `:=`) in source code.
* Filter comments depending on different keywords.
* Scan specific directory paths or files.
* Support custom languages via configuration.

Languages supported by default:

+ rs
+ go
+ lisp
+ py
+ hs
+ el
+ clj
+ js

If you want to support other languages, check [expand json file](#expand-json-file).

## Usage ##

Write code as usual. For any comment line you want to track, add the `:=` symbol immediately after the comment prefix.

Then run the `codeitlater` command in your terminal to retrieve those crumbs.

For example:

**Golang**:

```golang
// /user/src/main.go
// test codeitlater
//:= this line can be read by codeitlater
//:= MARK: you can leave a keyword to mark the comment line
/*:= multiline comments
*/

```

Then running `codeitlater` in the root path:

```
|-- /user/src/main.go
  |-- Line 3: this line can be read by codeitlater
  |-- Line 4: MARK: you can leave a keyword to mark the comment line
  |-- Line 5: multiline comments
```

**Python**:

```python
# /src/main.py
# this line wont be read
#:= this line for codeitlater
print("aaa") ###:= this line can be read again
```

Run `codeitlater /path/to/this_py.py`.

You will get:

```
|-- /src/main.py
  |-- Line 3: this line for codeitlater
  |-- Line 4: this line can be read again
```


**Specify file types**

```
codeitlater -f clj
```

You will get results only from Clojure files.

```
codeitlater -f clj -f py
```

Then results from both Clojure and Python files will return.

```
codeitlater -f clj -f py /path/to/file /path/to/dir
```

### Multi-line ###

When a crumb line ends with `...`, the content of the **next** line (which must also start with a crumb marker) is appended to it. You can chain multiple lines this way.

For example:

```rust
//:= line1...
//:= , and line2...
//:= line3 with line2...

//:= line4 is different...
//:= with line5
//:= line6
```

Will give you:

```
  |-- Line 1: line1 , and line2 line3 with line2...
  |-- Line 4: line4 is different with line5
  |-- Line 6: line6
```

### Filter keyword ###

Keyword format is `Keyword:` followed by a space.

Filter by keyword (use the `-k` or `--keywords` flags; check out more flags with `-h`):

`codeitlater -k MARK`

You will get:

```
|-- /user/src/main.go
  |-- Line 4: MARK: you can leave a keyword to mark the comment line
```

Similar to file types, if you want to query multiple keywords together:

`codeitlater -k TODO --keywords MARK`

**CAUTION:** If keywords and multi-line crumbs are mixed, the multi-line feature takes priority.

Example:

```
//:= TODO: aaaa...
//:= bbb...
//:= MARK: ccc
```

Both `codeitlater` and `codeitlater -k TODO` will show:

> |-- Line 1: TODO: aaaa bbb MARK: ccc

`codeitlater -k MARK` will show nothing.

### Ignore with keyword ###

To exclude specific crumbs from default runs:

```rust
//:= !JIRA-123: hello world
//:= line2
```

The first line "hello world" will be ignored by default because it starts with `'!'`. To display it, query for its specific keyword:

`codeitlater -k JIRA-123`

Or pass the `--show-ignored` flag to display all ignored crumbs:

`codeitlater --show-ignored`

### Excluding some folder ###

`codeitlater -x vendor` will ignore all files in the `vendor` directory (recursively).

### Expand json file ###

Check [tests/testcases/test.json](file:///Users/ccQ/Code/code-it-later-rs/tests/testcases/test.json). If you run `codeitlater -j ./tests/testcases/test.json`, the `"rs"` entry in the built-in dictionary is overridden by the value in the JSON file. Other languages remain unchanged.

### Local arguments ###

`codeitlater` will look for a `.codeitlater` file in the current working directory (`$PWD/.codeitlater`) to pre-load configuration arguments. Any arguments passed directly on the command line override those in `.codeitlater`, **except for ignore directories (`-x`)**, which are merged together.

### Delete the crumbs ###

`codeitlater -D target` removes crumb comments from files in the targeted folder. Deletion triggers an interactive prompt with `y/n/s/i` options:
- `y`: delete all crumbs shown.
- `n`: do nothing.
- `s`: show (re-print the list of crumbs).
- `i`: interactive mode (allows you to review and confirm deletion file-by-file or crumb-by-crumb).

You can delete special keywords with `codeitlater -D -k TODO`. Generally, `-D` is handled after the normal scan workflow completes.

### Restore the crumbs ###

Similar to the delete feature, but this option restores the crumb markers inside the code back to regular comments.

```golang
//:= here
```

will restore to:

```golang
// here
```

### Run format after clean the crumbs ###

After cleaning crumbs from your files, you might want to run a formatter. The `--fmt` option specifies a formatter command to run automatically after cleaning.

For example:

`codeitlater -D --fmt "go fmt" .` will delete your crumbs and run `go fmt` on the files. The command passed to `--fmt` must be a standalone executable command.

As with all other options, you can add this inside your local `.codeitlater` configuration file.

### Output to different format of files ###

`-O/--output-format` outputs the crumbs in a specific format.

Supported formats:

+ json
+ list

Example:

```shell
codeitlater -O json .
```

### Output the range of context ###

`-r/--range <N>` outputs `N` lines above and below the crumb to provide more context (useful for feeding context to LLM agents).

## Google Antigravity Integration ##

If you use Google Antigravity (AGY) as your AI coding assistant, you can install the custom `code-it-later` skill to let the assistant scan and manage crumbs in your codebase.

To install or update the skill, copy `SKILL.md` to your local Antigravity skills directory:

```shell
cp ./SKILL.md ~/.gemini/antigravity-cli/skills/code-it-later/SKILL.md
```
