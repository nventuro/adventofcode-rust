use std::fs;

mod computer;
use computer::*;

mod game;
use game::*;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Failed to read from file '{}'", filename));

    let program: Vec<_> = contents
        .split(',')
        .map(|x| x.trim().parse::<i64>().unwrap())
        .collect();

    process(program);
}

fn process(program: Vec<i64>) {
    let mut game_display = GameDisplay::new();
    let mut computer = Computer::new(program, Some(&mut game_display));
    computer.run();
}
