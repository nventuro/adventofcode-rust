use std::fs;
use std::io::{self, Write};

mod computer;
use computer::*;

mod robot;
use robot::*;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Failed to read from file '{}'", filename));

    let program: Vec<_> = contents
        .split(",")
        .map(|x| x.trim().parse::<i64>().unwrap())
        .collect();

    process(program);
}

fn process(program: Vec<i64>) {
    let mut robot = Robot::new();

    let mut computer = Computer::new(program, Some(&mut robot));
    computer.run();

    println!("Painted {:?} cells", robot.grid.colors.len());

    let max_x = robot.grid.colors.keys().map(|pos| pos.x).max().unwrap();
    let min_x = robot.grid.colors.keys().map(|pos| pos.x).min().unwrap();

    let max_y = robot.grid.colors.keys().map(|pos| pos.y).max().unwrap();
    let min_y = robot.grid.colors.keys().map(|pos| pos.y).min().unwrap();

    println!("X: {:?}, {:?}", min_x, max_x);
    println!("Y: {:?}, {:?}", min_y, max_y);

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            match robot.grid.get(&Position::new(x, y)) {
                Color::White => print!("#"),
                Color::Black => print!(" "),
            }
        }

        print!("\n");
        io::stdout().flush().unwrap();
    }
}
