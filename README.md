# Compy - Shell agnostic command completion

## A rust implementation of Bash's compgen

`compy` is a shell agonstic re-implementation of bash's `compgen`.Since one of the
goals is to be shell agnostic, certain `compgen` features like `-F` and some of
it's actions like `arrayvar` are omitted.

The project is split into a binary and a library so that `compy` can be used in
other projects.

## Building and Installing

Run `cargo build` to build the project and `cargo install --path .` to install it.
