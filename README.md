# lambda-rust

A lambda calculus interpreter I wrote as a toy project in Rust. Most of the functionality is written from scratch
(i.e. manually written lexer and parser) as an exercise, save for `readline` capabilities using
[rustyline](https://github.com/kkawakam/rustyline).

It supports normal order evaluation of lambda terms and definition of bindings for ease of usage.
See [the prelude](prelude.lmd) for examples.

# TODO

- [x] implement better control of when reductions happen
- [x] add interpreter commands for output control, displaying bindings, ...
- [ ] move the calculation to a separate thread and make it interruptible
- [ ] implement lazy evaluation of reductions - hardest
