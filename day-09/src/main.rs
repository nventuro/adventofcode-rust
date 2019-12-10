use std::fs;

mod computer;
use computer::*;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    process(contents);
}

fn process(input: String) {
    let program: Vec<_> = input.split(",")
        .map(|x| x.trim().parse::<i64>().unwrap())
        .collect();

    let mut computer = Computer::new_with_terminal(program.clone());
    computer.run();
}
