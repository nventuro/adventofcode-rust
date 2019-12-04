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
    let program: Vec<i32> = input.split(",")
        .map(|x| x.trim().parse::<i32>().unwrap())
        .collect();

    println!("Original program:\n{:?}", &program);
    println!("Performing replacements...", );

    let initialized = initialize(program, 12, 2);

    println!("Running program:\n{:?}", &initialized);

    println!("Output:\n{:?}", run_program(initialized));
}

fn initialize(mut program: Vec<i32>, noun: i32, verb: i32) -> Vec<i32> {
    program[1] = noun;
    program[2] = verb;

    program
}

fn run_program(initial_program: Vec<i32>) -> Vec<i32> {
    let mut instr_ptr: usize = 0;
    let mut program = initial_program.clone();

    while program[instr_ptr] != 99 {
        program = run_opcode(program, instr_ptr);
        instr_ptr += 4;
    }

    program
}

fn run_opcode(program: Vec<i32>, instr_ptr: usize) -> Vec<i32> {
    assert!(program.len() >= instr_ptr + 3); // each opcode has 3 arguments

    if program[instr_ptr] == 1 {
        let lhs_index: usize = program[instr_ptr + 1].try_into().unwrap();
        let rhs_index: usize = program[instr_ptr + 2].try_into().unwrap();
        let result_index: usize = program[instr_ptr + 3].try_into().unwrap();

        let mut new_program = program.clone();
        new_program[result_index] = program[lhs_index] + program[rhs_index];
        new_program

    } else if program[instr_ptr] == 2 {
        let lhs_index: usize = program[instr_ptr + 1].try_into().unwrap();
        let rhs_index: usize = program[instr_ptr + 2].try_into().unwrap();
        let result_index: usize = program[instr_ptr + 3].try_into().unwrap();

        let mut new_program = program.clone();
        new_program[result_index] = program[lhs_index] * program[rhs_index];
        new_program
    } else {
        panic!("Unknown opcode: '{}'", program[instr_ptr]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add_opcode() {
        assert_eq!(run_opcode([1,0,0,0].to_vec(), 0), [2,0,0,0].to_vec());
    }

    #[test]
    fn test_simple_mul_opcode() {
        assert_eq!(run_opcode([2,3,0,3].to_vec(), 0), [2,3,0,6].to_vec());
    }

    #[test]
    fn test_complex_add_program() {
        assert_eq!(run_program([1,1,1,4,99,5,6,0,99].to_vec()), [30,1,1,4,2,5,6,0,99].to_vec());
    }

    #[test]
    fn test_complex_mul_program() {
        assert_eq!(run_program([2,4,4,5,99,0].to_vec()), [2,4,4,5,99,9801].to_vec());
    }
}
