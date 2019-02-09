# sudoku-rs

Sudoku solver and generator in Rust to explore benchmarking and profiling. It
is quite fast.

## Usage

```shell
$ cargo run --release -- --help
$ cargo run --release -- solve data/sudoku17.txt --parallel
$ cargo run --release -- generate --count 15000 --free-cells 60
```
