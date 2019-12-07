mod computer;

use std::fs;
use std::io::{ self, Write };

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

    let mut computer = computer::Hardware::new(program, prompt, print);
    computer.run();
}

fn prompt() -> i32 {
    print!("PROMPT: ");
    io::stdout().flush().unwrap();

    let mut raw_input = String::new();
    io::stdin().read_line(&mut raw_input).unwrap();
    raw_input.trim().parse::<i32>().unwrap()
}

fn print(value: i32) {
    println!("PRINT: {}", value);
}
