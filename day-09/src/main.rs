use std::fs;

extern crate itertools;
use itertools::Itertools;

mod amplifiers;

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

    let highest = (5..10).permutations(5)
        .map(|phase_sequence| amplifiers::run_phase_sequence(program.clone(), phase_sequence))
        .max();

    println!("Highest signal: {}", highest.unwrap());
}
