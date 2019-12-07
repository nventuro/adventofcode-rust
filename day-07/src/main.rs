mod computer;
use computer::Computer as Computer;

use std::fs;

extern crate itertools;
use itertools::Itertools;

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


    let highest = (0..5).permutations(5)
        .map(|phase_sequence| run_phase_sequence(program.clone(), phase_sequence))
        .max();

    println!("Highest signal: {}", highest.unwrap());
}

fn run_phase_sequence(program: Vec<i32>, phase_sequence: Vec<i32>) -> i32 {
    let mut previous_output_signal = 0;

    for phase in phase_sequence {
        let mut output_signal = 0;
        {
            let mut phase_queried = false;
            let input = || {
                if !phase_queried {
                    phase_queried = true;
                    phase
                } else {
                    previous_output_signal
                }
            };

            let mut computer = Computer::new(program.clone(), input, |value| output_signal = value);
            computer.run();
        }

        println!("{:?}", output_signal);
        previous_output_signal = output_signal;
    }

    previous_output_signal
}

#[cfg(test)]
mod sequencer {
    use super::*;

    #[test]
    fn test_sequences() {
        assert_eq!(
            run_phase_sequence(
                vec![3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0],
                vec![4,3,2,1,0]
            ), 43210
        );

        assert_eq!(
            run_phase_sequence(
                vec![
                    3,23,3,24,1002,24,10,24,1002,23,-1,23,
                    101,5,23,23,1,24,23,23,4,23,99,0,0
                ],
                vec![0,1,2,3,4]
            ), 54321
        );

        assert_eq!(
            run_phase_sequence(
                vec![
                    3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,
                    1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0
                ],
                vec![1,0,4,3,2]
            ), 65210
        );
    }
}
