# README #

[![Crates.io](https://img.shields.io/crates/v/code-it-later-rs.svg)](https://crates.io/crates/code-it-later-rs)

- [Summary](#summary)
- [Install](#install)
  - [Installing the Emacs Interface Mode](#installing-the-emacs-interface-mode)
- [Features](#features)
- [Usage](#usage)
  - [Mulit-line](#mulit-line)
  - [Filter keyword](#filter-keyword)
  - [Ignore with keyword](#ignore-with-keyword)
  - [Excluding some folder](#excluding-some-folder)
  - [Expand json file](#expand-json-file)
  - [Local arguments](#local-arguments)
  - [Clean the crumbs](#clean-the-crumbs)
  - [Run format after clean the crumbs](#run-format-after-clean-the-crumbs)
  - [Output to different format of files](#output-to-different-format-of-files)

**Other Versions (have some features different):**

+ [code_it_later_ml](https://github.com/ccqpein/code_it_later_ml) ocaml version
+ [code-it-later](https://github.com/ccqpein/code-it-later) clojure version
+ [code_it_later_hs](https://github.com/ccqpein/code_it_later_hs) haskell version

## Summary ##

Make flags in source code where may have problems or can be optimized. codeitlater help you track this flags and fix them in future.

## Install ##

`cargo install code-it-later-rs`

### Installing the Emacs Interface Mode ###

I developed the [helm-code-it-later](https://github.com/ccqpein/helm-code-it-later), which serves as an interface mode for my Emacs.

## Features ##

* get comments in source code
* get comments depending on different key words
* get comments in special path of dir or files
* can expand to other languages

Languages support now:

+ rs
+ go
+ lisp
+ py
+ hs
+ el
+ clj
+ js

If you wanna expand other languages, check [expand json file](#expand-json-file)

## Usage ##

Write code as usual. The comment line that you want to leave mark in, left `:=` symbol after comment symbol.

Then run `codeitlater` command in terminal get those crumbs back. 

For example:

**Golang**:

```golang
// /user/src/main.go
// test codeitlater
//:= this line can be read by codeitlater
//:= MARK: you can left keyword to marked comment line
/*:= mutil lines comments
*/

```

then run `codeitlater` in code root path 

You will get:

```
|-- /user/src/main.go
  |-- Line 3: this line can be read by codeitlater
  |-- Line 4: MARK: you can left keyword to marked comment line
  |-- Line 5: mutil lines comments
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
  |-- Line 3: this line for codeitlater"
  |-- Line 4: this line can be read again"
```


**Give specify file type**

```
codeitlater -f clj
```

You will get result only from clojure.

```
codeitlater -f clj -f py
```

Then results both of clojure and python will return.

```
codeitlater -f clj -f py /path/to/file /path/to/dir
```

### Mulit-line ###

When one line ending with `...`, then, the **next** line will add to this crumb. Also, you can make tail chain for this.

For example:

```rust
//:= line1...
//:= , and line2...
//:= line3 with line2...

//:= line4 is diffrent...
//:= with line5
//:= line6
```

Will give you:

```
  |-- Line 1: line1 , and line2 line3 with line2...
  |-- Line 4: line4 is diffrent with line5
  |-- Line 6: line6
```

### Filter keyword ###

Keyword format is `Keyword:` with a space after.

Filter keyword (use -k be keyword flag, check out more flags by -h):

`codeitlater -k MARK`

You will get:

```
|-- /user/src/main.go
  |-- Line 4: MARK: you can left keyword to marked comment line
```

Same format as filetypes, if you want get two keywords together:

`codeitlater -k TODO --keywords MARK`

**CAUTION:** if keywords and multi-lines are mixed, multi-lines feature has higher priority. 

Example:

```
//:= TODO: aaaa...
//:= bbb...
//:= MARK: ccc
```

Both `codeitlater` and `codeitlater -k TODO` are showing 

> |-- Line 1: TODO: aaaa bbb MARK: ccc

`codeitlater -k MARK` will show nothing.

### Ignore with keyword ###

This is the special feature I use in my work. For example:

```rust
//:= !JIRA-123: hello world
//:= line2
```

The first line "hello world" will be ignore because it start with `'!'`. To show this line is give the keyword `JIRA-123` like `codeitlater -k JIRA-123`

Or give the `--show-ignored` true if you want to show everything, like `codeitlater --show-ignored true`.

### Excluding some folder ###

`codeitlater -x vendor` will ignore all files in vendor (recursively).

### Expand json file ###

Check `tests/test.json`, if you run `codeitlater -j ./tests/test.json`, the "rs" in codeitlater's dictionary will be covered by new value in `test.json`. Other languages are keep same.

### Local arguments ###

`codeitlater` will look for `{$PWD}/.codeitlater` file to pre-load arguments. If any arguments those been given in command line, also set inside the `.codeitlater` file, will be rewrote by command line arguments (**except ignore dirs (-x)**, ignore dirs configs located inside `.codeitlater` file and given in command line will merge together). 

### Clean the crumbs ###

`codeitlater -D target` gonna clean all crumbs inside the files in the target folder. Delete will give prompt interaction, which has `y/n/s/i` options. `y` means delete the bread/crumbs it just shows; `n` means ignore this; `s` means `show`, just re-print it again; `i` going to interact mode, show bread one by one or crumb one by one.

You can delete special keywords with `codeitlater -D -k TODO`. Generally, `-D` handle after normal `codeitlater` workflow done.

### Run format after clean the crumbs ###

After clean some crumbs inside files, you might need some format after it. You can give the `--fmt` options let `codeitlater` run the command given after clean. 

For example:

`codeitlater -D --fmt "go fmt" .` will delete your crumbs and run the `go fmt`. The command after `--fmt` has to be the standalone command.

As all other options, you can add it inside the local `{$PWD}/.codeitlater`.

### Output to different format of files ###

`-O/--output-format` can output the crumbs in specific format. 

Support format:

+ json
+ list

Example:

```shell
codeitlater -O json .
```
