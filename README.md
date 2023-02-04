# LYNE Solver

A Rust project to solve puzzles in the game LYNE.

## Usage

1. Clone the project from GitHub.
2. Run the project using the command `RUST_LOG=warn cargo run --release`.
3. Input the puzzle you want to solve, using lowercase rgb to represent red, green, and blue nodes, using uppercase RGB to represent red, green, and blue start/end points, using . to represent a space, and using numbers to represent points that can be passed through multiple times.

For example, B 15 input as follow:
```
R2B
2Gr
gbR
.GB
```

## Future Plans

* Better display of solutions.
* Automatic recognition of images.

## License

This project is licensed under the [AGPL-3.0 License](/LICENSE).
