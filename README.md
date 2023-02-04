# LYNE Solver

A Rust project to solve puzzles in the game LYNE.

## Usage

1. Clone the project from GitHub.
2. Run the project using the command `RUST_LOG=warn cargo run --release`.
3. Input the puzzle you want to solve, using lowercase rgb to represent red, green, and blue nodes, using uppercase RGB to represent red, green, and blue start/end points, using . to represent a space, and using numbers to represent points that can be passed through multiple times.

For example, B 15 input as follow:

| | |
| ------------- | ------------- |
| <img src="https://user-images.githubusercontent.com/17942323/216759636-00badbf1-0a8e-45ed-a4ee-840f6a0c9fd5.png" width="100">  | <pre>R2B<br>2Gr<br>gbR<br>.GB</pre>  |


## Future Plans

* Better display of solutions.
* Automatic recognition of images.

## License

This project is licensed under the [AGPL-3.0 License](/LICENSE).
