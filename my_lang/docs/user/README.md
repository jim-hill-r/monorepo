# my_lang user guide

## Who it's for

Programmers who want to write small programs in my_lang, a language built
around pure functions with side effects kept in their own declarations. You
write a `.my` file, and the `my_lang` command runs it straight away or
compiles it to a native executable through LLVM.

my_lang is early. Every value is a 64-bit integer, and there are no `if`
statements, loops, comments or strings yet. It's for trying the language
out, not for real work. The [README](../../README.md) describes how the
compiler is built, and [requirements.md](../requirements.md) lists what it
promises.

## Getting started

You need Rust, LLVM 18 and a C compiler (`cc`, which links the executables
`my_lang build` writes).

1. Install LLVM 18. On Debian or Ubuntu run
   `sudo apt-get install llvm-18-dev libpolly-18-dev libzstd-dev`. On macOS
   run `brew install llvm@18`, then
   `export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)`.
2. Get the repository and install the command from the `my_lang` folder:

   ```bash
   git clone https://github.com/jimhillr/monorepo.git
   cd monorepo/my_lang
   LLVM_SYS_181_PREFIX=${LLVM_SYS_181_PREFIX:-/usr/lib/llvm-18} cargo install --path .
   ```

3. Check that it works with `my_lang --version`. It prints `my_lang 0.1.0`.

## How to write and run a first program

1. Make a file called `answer.my` that holds this line:

   ```text
   function main() { 6 * 7 }
   ```

2. Run it with `my_lang run answer.my`. my_lang compiles the program, calls
   `main` and prints what it gives: `42`.
3. To get an executable instead, run `my_lang build answer.my`. It writes
   `answer` next to the source file (add `-o <path>` to choose another name).
   Run `./answer` and it prints `42`.

Every program needs a `main`. With `run`, `main` can take up to 8
parameters, and you pass them as numbers after the file name. Say `add.my`
holds:

```text
function add(a, b) { a + b }
function main(x, y) { add(x, y) * 2 }
```

Then `my_lang run add.my 3 4` prints `14`. Negative numbers work too, as in
`my_lang run add.my 3 -10`. An executable from `build` takes no arguments,
so its `main` must have no parameters.

## How to write functions

- A function is `function name(params) { body }`. Its body is statements
  separated by `;` or new lines.
- It gives the value of the `return` it reaches, or else the expression its
  body ends with. `function add(a, b) { return a + b }` and
  `function add(a, b) { a + b }` are the same function.
- A function must give a value. A body that is empty, or that ends in an
  assignment with no `return`, is a compile error.
- `x = 10` makes a variable. You can use it in the lines after it.
- The operators are `+`, `-`, `*` and `/` (integer division), plus `==`,
  `!=`, `<`, `>`, `<=` and `>=`, which give `1` for true and `0` for false.
  Use brackets to group, as in `(1 + 2) * 3`.
- Define a function before the functions that call it. A call to one
  defined further down fails with "undefined function".
- There are no negative literals: write `0 - 5` for -5.

## How to use pure functions and side effects

my_lang has three kinds of declaration, and the kind says what it may do:

| Keyword | For | Gives |
| --- | --- | --- |
| `function` | Pure work: no side effects | Its `return` or last expression |
| `input` | Reading the outside world | Always `0`; `return` isn't allowed |
| `output` | Writing to the outside world | Its `return` or last expression, like a `function` |

```text
input read_sensor() { 5 }
output log(v) { return v + 1 }
function main() { x = 10; y = x * 3; return y - 4 }
```

An `input` that uses `return` is a compile error. my_lang can't read or
write anything outside the program yet, so for now these keywords only mark
your intent. A `function` can still call an `input` or an `output`; the
compiler doesn't stop it yet.

## How to read a compile error

When something is wrong, my_lang prints one line starting `my_lang:` on
standard error and exits with status 1. The words after that say which step
failed:

- **`parse error: UnexpectedToken { expected: ..., found: ... }`**: the
  program isn't written the way my_lang expects. `found` is the first thing
  that doesn't fit, and `expected` is what should have been there. For
  example, `(1 + 2 }` gives `expected: "RightParen", found: RightBrace`
  because a `)` is missing. `found: Illegal('%')` means a character my_lang
  doesn't know, and `Illegal('0')` at the end of a long number means the
  number is too big for 64 bits.
- **`codegen error: ...`**: the program parses, but it can't be compiled.
  The message names the problem, such as `undefined variable: 'y'`,
  `undefined function: 'foo'`, or
  `function 'main' gives no value: add a 'return' or end its body with an expression`.
- **`no entry function 'main' in the program`**: add a `main`.
- **`'main' takes 2 arguments, but 1 were given`**: pass one number for
  each of `main`'s parameters after the file name.
- **`entry function 'main' must take no parameters, but takes 2`**: `build`
  needs a `main` with no parameters. Use `run` to pass arguments.
- **`could not read answer.my: ...`**: the file name or path is wrong.

## Getting help

Common problems:

- **`cargo install` fails with "No suitable version of LLVM was found"**:
  LLVM 18 isn't installed, or `LLVM_SYS_181_PREFIX` doesn't point at it.
  Install it (see Getting started) and set the variable to its folder.
- **`my_lang build` fails while linking**: there's no C compiler. Install
  one (`build-essential` on Debian or Ubuntu, Xcode's command line tools on
  macOS), or point `CC` at the one you have.
- **The program crashes with "Floating point exception"**: it divided by
  zero. my_lang doesn't catch that yet, so check what you divide by.
- **A `main` that ends in a string prints a huge number**: strings aren't
  supported yet. They parse, but what comes out is meaningless.
- **An `input` gives 0 when you expected another number**: that's how
  `input` works for now (see the table above). Use a `function` for a
  value you need.

my_lang has no feedback button. To report a problem or ask for a feature,
tell Jim the program, the command you ran and what my_lang printed. It
will be recorded as an issue in `my_lang/issues/`.
