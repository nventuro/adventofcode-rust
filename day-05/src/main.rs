#[macro_use] extern crate text_io;

mod computer;

use std::fs;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    process(contents);
}

fn process(input: String) {
    let program: Vec<i32> = input.split(",")
        .map(|x| x.trim().parse::<i32>().unwrap())
        .collect();

    let mut computer = computer::Hardware::new(program);
    computer.run();
}

