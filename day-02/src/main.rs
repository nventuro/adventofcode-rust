use std::fs;
use std::convert::TryInto;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    process(contents);
}

fn process(input: String) {
    let mut program: Vec<i32> = input.split(",")
        .map(|x| x.trim().parse::<i32>().unwrap())
        .collect();

    println!("Original program:\n{:?}", &program);
    println!("Performing replacements...", );

    program[1] = 12;
    program[2] = 2;

    println!("Running program:\n{:?}", &program);

    println!("Output:\n{:?}", run_program(&program));
}

fn run_program(initial_program: &Vec<i32>) -> Vec<i32> {
    let mut run_index: usize = 0;
    let mut program = initial_program.clone();

    while program[run_index] != 99 {
        program = run_opcode(&program, run_index);
        run_index += 4;
    }

    program
}

fn run_opcode(program: &Vec<i32>, run_index: usize) -> Vec<i32> {
    assert!(program.len() >= run_index + 3); // each opcode has 3 arguments

    if program[run_index] == 1 {
        let lhs_index: usize = program[run_index + 1].try_into().unwrap();
        let rhs_index: usize = program[run_index + 2].try_into().unwrap();
        let result_index: usize = program[run_index + 3].try_into().unwrap();

        let mut new_program = program.clone();
        new_program[result_index] = program[lhs_index] + program[rhs_index];
        new_program

    } else if program[run_index] == 2 {
        let lhs_index: usize = program[run_index + 1].try_into().unwrap();
        let rhs_index: usize = program[run_index + 2].try_into().unwrap();
        let result_index: usize = program[run_index + 3].try_into().unwrap();

        let mut new_program = program.clone();
        new_program[result_index] = program[lhs_index] * program[rhs_index];
        new_program
    } else {
        panic!("Unknown opcode: '{}'", program[run_index]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add_opcode() {
        assert_eq!(run_opcode(&[1,0,0,0].to_vec(), 0), [2,0,0,0].to_vec());
    }

    #[test]
    fn test_simple_mul_opcode() {
        assert_eq!(run_opcode(&[2,3,0,3].to_vec(), 0), [2,3,0,6].to_vec());
    }

    #[test]
    fn test_complex_add_program() {
        assert_eq!(run_program(&[1,1,1,4,99,5,6,0,99].to_vec()), [30,1,1,4,2,5,6,0,99].to_vec());
    }

    #[test]
    fn test_complex_mul_program() {
        assert_eq!(run_program(&[2,4,4,5,99,0].to_vec()), [2,4,4,5,99,9801].to_vec());
    }
}
