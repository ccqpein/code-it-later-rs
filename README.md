# README #

**Other Versions:**

+ [code_it_later_ml](https://github.com/ccqpein/code_it_later_ml) ocaml version
+ [code-it-later](https://github.com/ccqpein/code-it-later) clojure version
+ [code_it_later_hs](https://github.com/ccqpein/code_it_later_hs) haskell version

## Summary ##

Make flags in source code where may have problems or can be optimized. codeitlater help you track this flags and fix them in future.

## Install ##

`cargo install code-it-later-rs`

## Features ##

* get comments in source code
* get comments depending on different key words
* get comments in special path
* can expand to other languages

## Usage ##

Write code as usual. The comment line that you want to leave mark in, left `:=` symbol after comment symbol.

For example:

**Golang**:

```golang
// /user/src/main.go
// test codeitlater
//:= this line can be read by codeitlater
//:= MARK: you can left keyword to marked comment line
/* mutil lines comments
*/

```

then run `codeitlater` in code root path 

You will get:

```
|-- /user/src/main.go
  |-- (3 "this line can be read by codeitlater")
  |-- (4 "MARK: you can left keyword to marked comment line")
  |-- (5 "mutil lines comments")
```

**Python**:

```python
# /src/main.py
# this line wont be read
#:= this line for codeitlater
print("aaa") ###:= this line can be read again
```

Run `codeitlater`

You will get:

```
|-- /src/main.py
  |-- (3 "this line for codeitlater")
  |-- (4 "this line can be read again")
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


#### Specific path ####

Run `codeitlater -d /user/src/` let codeitlater just scan specific path.

#### Filter keyword ####

Keyword format is `Keyword:` with a space after.

Filter keyword (use -k be keyword flag, check out more flags by -h):
`codeitlater -k MARK`

You will get:

```
|-- /user/src/main.go
  |-- (4 "MARK: you can left keyword to marked comment line")
```

Same format as filetypes, if you want get two keywords together:

`codeitlater -k TODO --keywords MARK`

#### Excluding some folder ####

`codeitlater -x vendor` will ignore all files in vendor (recursively).

#### Expand json file ####

Check `tests/test.json`, if you run `codeitlater -j ./tests/test.json`, the "rs" in codeitlater's dictionary will be covered by new value in `test.json`. Other languages are keep same.
