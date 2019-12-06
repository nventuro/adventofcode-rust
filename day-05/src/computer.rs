use std::convert::{ TryFrom, TryInto };
use std::io::{ self, Write };

pub struct Hardware {
  memory: Vec<i32>,
  program_counter: usize,
}

impl Hardware {
  pub fn new(program: Vec<i32>) -> Hardware {
    Hardware {
      memory: program,
      program_counter: 0,
    }
  }

  fn read(&self, address: usize) -> i32 {
    self.memory[address]
  }

  fn write(&mut self, address: usize, value: i32) {
    self.memory[address] = value;
  }

  fn next_instruction(&self) -> Instruction {
    self.read(self.program_counter).try_into().expect("Unknown opcode")
  }

  // Reads the value of the nth parameter for the current instruction (0-based)
  fn parameter(&self, nth: usize) -> (i32, ArgumentMode) {
    let value = self.read(self.program_counter + nth + 1);

    // In a string representation, the last two digits of the instruction value
    // are the opcodes (i.e. opcodes go from 0 to 99). We first remove those.
    let mode_indicators = self.read(self.program_counter) / 100;

    // Then, the mode for the 0-based nth parameter is the nth digit from the
    // right: the first parameter is the units, second tenths, and so on.
    let mode_indicator = (mode_indicators / 10_i32.pow(nth.try_into().unwrap())) % 10;

    let mode = mode_indicator.try_into().unwrap();

    (value, mode)
  }

  fn relative_jump_forward(&mut self, distance: usize) {
    self.program_counter += distance;
  }

  pub fn run(&mut self) {
    println!("BEGIN");

    loop {
      if self.step() == Instruction::Halt {
        break;
      }
    }
  }

  fn step(&mut self) -> Instruction {
    let instruction = self.next_instruction();

    // Collect parameters
    let parameters: Vec<Parameter> = instruction.parameter_types()
      .iter()
      .enumerate()
      .map(|(index, parameter_type)| {
        let (argument, mode) = self.parameter(index);

        match parameter_type {
          ParameterType::In => {
            Parameter::In(
              match mode {
                ArgumentMode::Immediate => argument,
                ArgumentMode::Indexed => self.read(to_address(argument)),
              }
            )
          },
          ParameterType::Out => {
            // Out parameters can only be indexed
            assert_eq!(mode, ArgumentMode::Indexed);

            Parameter::Out(to_address(argument))
          },
        }
      }).collect();

    // Run instruction
    instruction.exec(&parameters, self);

    // Consume opcode and its parameters
    self.relative_jump_forward(1 + parameters.len());

    instruction
  }
}

// Different types of instructions
#[derive(PartialEq)]
#[derive(Debug)]
enum Instruction {
  Add,
  Mul,
  Prompt,
  Print,
  Halt,
}

// An instruction's parameters may be in-values (read from) or out-values (written to)
enum ParameterType {
  In,
  Out,
}

// In-values are just values, while out-values are addresses (where a value is stored)
#[derive(Debug)]
enum Parameter {
  In(i32),
  Out(usize),
}

#[derive(PartialEq)]
#[derive(Debug)]
enum ArgumentMode {
  Immediate, // The value itself is the argument
  Indexed, // The value is the address of the argument
}

// Map indicators to argument modes
impl TryFrom<i32> for ArgumentMode {
    type Error = i32;

    fn try_from(x: i32) -> Result<Self, Self::Error> {
        match x {
          0 => Ok(ArgumentMode::Indexed),
          1 => Ok(ArgumentMode::Immediate),
          _ => Err(x),
        }
    }
}

impl Parameter {
  // Extracts in-value or fails
  fn get_input(&self) -> i32 {
    match self {
      Parameter::In(input) => *input,
      _ => panic!("Non-input parameter: {:?}", self),
    }
  }

  // Extracts out-value or fails
  fn get_output(&self) -> usize {
    match self {
      Parameter::Out(output) => *output,
      _ => panic!("Non-output parameter: {:?}", self),
    }
  }
}

// Map value (opcode) to instruction
impl TryFrom<i32> for Instruction {
    type Error = i32;

    fn try_from(x: i32) -> Result<Self, Self::Error> {
        // Opcodes go from 0 to 99, the rest of the value sets the argument modes
        match x % 100 {
          1 => Ok(Instruction::Add),
          2 => Ok(Instruction::Mul),
          3 => Ok(Instruction::Prompt),
          4 => Ok(Instruction::Print),
          99 => Ok(Instruction::Halt),
          _ => Err(x),
        }
    }
}

impl Instruction {
  // Map each instruction to its parameter types
  fn parameter_types(&self) -> Vec<ParameterType> {
    use ParameterType::*;

    match self {
      Instruction::Add => vec![In, In, Out],
      Instruction::Mul => vec![In, In, Out],
      Instruction::Prompt => vec![Out],
      Instruction::Print => vec![In],
      Instruction::Halt => vec![],
    }
  }

  // Execute an instruction on a hardware
  fn exec(&self, parameters: &[Parameter], hardware: &mut Hardware) {
    match self {
      Instruction::Add => {
        let lhs = parameters[0].get_input();
        let rhs = parameters[1].get_input();
        let result = parameters[2].get_output();

        hardware.write(result, lhs + rhs);
      },
      Instruction::Mul => {
        let lhs = parameters[0].get_input();
        let rhs = parameters[1].get_input();
        let result = parameters[2].get_output();

        hardware.write(result, lhs * rhs);
      },
      Instruction::Prompt => {
        print!("PROMPT: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        hardware.write(parameters[0].get_output(), input.trim().parse::<i32>().unwrap());
      },
      Instruction::Print => {
        println!("PRINT: {}", parameters[0].get_input());
      },
      Instruction::Halt => {
        println!("HALT");
      },
    }
  }
}

fn to_address(value: i32) -> usize {
    value.try_into().expect("Value is not an address")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add_instruction() {
      let mut computer = Hardware::new(vec![1,0,0,0]);
      let instruction = computer.step();

      assert_eq!(instruction, Instruction::Add);
      assert_eq!(computer.memory, vec![2,0,0,0]);
      assert_eq!(computer.program_counter, 4);
    }

    #[test]
    fn test_simple_mul_instruction() {
      let mut computer = Hardware::new(vec![2,3,0,3]);
      let instruction = computer.step();

      assert_eq!(instruction, Instruction::Mul);
      assert_eq!(computer.memory, vec![2,3,0,6]);
      assert_eq!(computer.program_counter, 4);
    }

    #[test]
    fn test_add_program() {
        let mut computer = Hardware::new(vec![1,1,1,4,99,5,6,0,99]);
        computer.run();
        assert_eq!(computer.memory, vec![30,1,1,4,2,5,6,0,99]);
    }

    #[test]
    fn test_mul_program() {
        let mut computer = Hardware::new(vec![2,4,4,5,99,0]);
        computer.run();
        assert_eq!(computer.memory, vec![2,4,4,5,99,9801]);
    }

    #[test]
    fn test_mul_program_argument_modes() {
        let mut computer = Hardware::new(vec![1002,4,3,4,33]);
        computer.run();
        assert_eq!(computer.memory, vec![1002,4,3,4,99]);
    }
}
