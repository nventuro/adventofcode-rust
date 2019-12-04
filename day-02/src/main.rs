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

    let noun = 12;
    let verb = 2;

    println!("With noun {} and verb {}, will run:\n{:?}", noun, verb, &program);

    let mut computer = Computer::new(program, noun, verb);
    computer.run();

    println!("Output:\n{:?}", computer.result());
}

fn to_address(value: i32) -> usize {
    value.try_into().unwrap()
}

struct OpcodeAdd {
    lhs_address: usize,
    rhs_address: usize,
    result_address: usize,
}

impl OpcodeAdd {
    fn new(computer: &Computer) -> OpcodeAdd {
        assert!(computer.memory.len() >= computer.counter + 3); // 3 arguments

        OpcodeAdd {
            lhs_address: to_address(computer.memory[computer.counter + 1]),
            rhs_address: to_address(computer.memory[computer.counter + 2]),
            result_address: to_address(computer.memory[computer.counter + 3]),
        }
    }

    fn exec(&self, computer: &mut Computer) {
        computer.memory[self.result_address] = computer.memory[self.lhs_address] + computer.memory[self.rhs_address];
    }
}

struct OpcodeMul {
    lhs_address: usize,
    rhs_address: usize,
    result_address: usize,
}

impl OpcodeMul {
    fn new(computer: &Computer) -> OpcodeMul {
        assert!(computer.memory.len() >= computer.counter + 3); // 3 arguments

        OpcodeMul {
            lhs_address: to_address(computer.memory[computer.counter + 1]),
            rhs_address: to_address(computer.memory[computer.counter + 2]),
            result_address: to_address(computer.memory[computer.counter + 3]),
        }
    }

    fn exec(&self, computer: &mut Computer) {
        computer.memory[self.result_address] = computer.memory[self.lhs_address] * computer.memory[self.rhs_address];
    }
}

struct Computer {
    memory: Vec<i32>,
    counter: usize,
}

impl Computer {
    fn new(mut program: Vec<i32>, noun: i32, verb: i32) -> Computer {
        program[1] = noun;
        program[2] = verb;

        Computer {
            memory: program,
            counter: 0,
        }
    }

    fn run(&mut self) {
        while self.memory[self.counter] != 99 {
            self.step();
        }
    }

    fn step(&mut self) {
        let opcode = self.memory[self.counter];
        if opcode == 1 {
            OpcodeAdd::new(self).exec(self)
        } else if opcode == 2 {
            OpcodeMul::new(self).exec(self)
        } else {
            panic!("Unknown opcode: '{}'", opcode);
        }

        self.counter += 4;
    }

    fn result(&self) -> i32 {
        assert!(self.memory[self.counter] == 99);
        self.memory[0]
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
