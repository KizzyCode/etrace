[![License](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

# About
This library provides some structs and macros to create stackable/traceable errors for your own custom error-kinds.

The main goal of this library is to provide a flexible way to create and handle error-traces without relying on a huge
codebase. Since error handling is used everywhere in a project, it's usually pretty hard to change it afterwards. This
library wants to adress this problem by being as simple as possible so that if you want to switch away you don't have a
fuckload of dependent code that you'd need replace.

# Features
We provide some macros to create and "throw/rethrow" an error with a custom kind `T` and optionally a custom description
and/or a previous `Error<T>` (even for different types of `T`).

Additionally we provide macros to evaluate an expression (`try_err!`, `try_err_from!`, `ok_or!`, `some_or!`, ...) to
either __convert__ the resulting error into your `Error<T>` or to __include__ the resulting error as sub-error into a
new `Error<T>`. 

# Build Library and Documentation
To build and open the documentation, go into the projects root-directory and run `cargo doc --open`.

To build the library, go into the projects root-directory and run `cargo build --release`; you can find the build in
target/release.