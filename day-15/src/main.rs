use std::fs;
use std::io::{self, Write};

mod computer;
use computer::*;

mod robot;
use robot::*;

extern crate termion;

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
    let mut robot = RobotIO::new();
    let mut computer = Computer::new(program, Some(&mut robot));
    computer.run();

    let mut world = robot.get_world();

    let mut count = 0;

    let minutes_required = loop {
        print!("{}", termion::cursor::Save);

        let (min_x, max_x, min_y, max_y) = world.limits();
        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                world.get(&Position::new(x, y)).draw();
            }

            println!("");
            io::stdout().flush().unwrap();
        }
        print!("{}", termion::cursor::Restore);

        if world.oxygenate_step() {
            count += 1;
        } else {
            break count;
        }
    };

    println!("Full oxygenation after {:?}", minutes_required);
}
