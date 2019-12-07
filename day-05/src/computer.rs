use std::convert::{ TryFrom, TryInto };
use std::io::{ self, Write };

type Address = usize;

pub trait AddressLike {
  fn from_value(value: i32) -> Address;
}

impl AddressLike for Address {
  fn from_value(value: i32) -> Address {
    value.try_into().expect("Value is not an address")
  }
}

pub struct Hardware {
  memory: Vec<i32>,
  program_counter: Address,
}

impl Hardware {
  pub fn new(program: Vec<i32>) -> Hardware {
    Hardware {
      memory: program,
      program_counter: 0,
    }
  }

  fn read(&self, location: Address) -> i32 {
    self.memory[location]
  }

  fn write(&mut self, location: Address, value: i32) {
    self.memory[location] = value;
  }

  fn next_instruction(&self) -> Instruction {
    self.read(self.program_counter).try_into().expect("Unknown opcode")
  }

  // Reads the value of the nth argument for the current instruction (0-based)
  fn argument(&self, nth: usize) -> (i32, ArgumentMode) {
    let value = self.read(self.program_counter + nth + 1);

    // In a string representation, the last two digits of the instruction value
    // are the opcode (i.e. opcodes go from 0 to 99). We first remove those.
    let mode_indicators = self.read(self.program_counter) / 100;

    // Then, the mode for the 0-based nth argument is the nth digit from the
    // right: the first argument is the units, second tenths, and so on.
    let mode_indicator = (mode_indicators / 10_i32.pow(nth.try_into().unwrap())) % 10;

    let mode = mode_indicator.try_into().unwrap();

    (value, mode)
  }

  fn relative_jump_forward(&mut self, distance: usize) {
    self.program_counter += distance;
  }

  fn absolute_jump(&mut self, destination: usize) {
    self.program_counter = destination;
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
    let current_program_counter = self.program_counter;
    let instruction = self.next_instruction();

    // Collect arguments
    let arguments: Vec<Argument> = instruction.argument_types()
      .iter()
      .enumerate()
      .map(|(index, argument_type)| {
        let (value, mode) = self.argument(index);

        match argument_type {
          ArgumentType::In => {
            Argument::In(
              match mode {
                ArgumentMode::Immediate => value,
                ArgumentMode::Indexed => self.read(Address::from_value(value)),
              }
            )
          },
          ArgumentType::Out => {
            // Out arguments can only be indexed
            assert_eq!(mode, ArgumentMode::Indexed);

            Argument::Out(Address::from_value(value))
          },
        }
      }).collect();

    // Run instruction
    instruction.exec(&arguments, self);

    // Move forward by consuming opcode and its arguments, but only if the
    // instruction didn't already change the program counter
    if current_program_counter == self.program_counter {
      self.relative_jump_forward(1 + arguments.len());
    }

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
  JumpIfTrue,
  JumpIfFalse,
  LessThan,
  Equals,
  Halt,
}

// An instruction's arguments may be in-values (read from) or out-values (written to)
enum ArgumentType {
  In,
  Out,
}

#[derive(PartialEq)]
#[derive(Debug)]
enum ArgumentMode {
  Immediate, // The value itself is the argument
  Indexed, // The value is the address of the argument
}

// In-values are just values, while out-values are addresses (where a value is stored).
// Note that this means that in-arguments in indexed mode must first be dereferenced
// to be used as an Argument.
#[derive(Debug)]
enum Argument {
  In(i32),
  Out(Address),
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

impl Argument {
  // Extracts in-value or fails
  fn get_input(&self) -> i32 {
    match self {
      Argument::In(input) => *input,
      _ => panic!("Non-input argument: {:?}", self),
    }
  }

  // Extracts out-value or fails
  fn get_output(&self) -> Address {
    match self {
      Argument::Out(output) => *output,
      _ => panic!("Non-output argument: {:?}", self),
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
          5 => Ok(Instruction::JumpIfTrue),
          6 => Ok(Instruction::JumpIfFalse),
          7 => Ok(Instruction::LessThan),
          8 => Ok(Instruction::Equals),
          99 => Ok(Instruction::Halt),
          _ => Err(x),
        }
    }
}

impl Instruction {
  // Map each instruction to its argument types
  fn argument_types(&self) -> Vec<ArgumentType> {
    use ArgumentType::*;

    match self {
      Instruction::Add => vec![In, In, Out],
      Instruction::Mul => vec![In, In, Out],
      Instruction::Prompt => vec![Out],
      Instruction::Print => vec![In],
      Instruction::JumpIfTrue => vec![In, In],
      Instruction::JumpIfFalse => vec![In, In],
      Instruction::LessThan => vec![In, In, Out],
      Instruction::Equals => vec![In, In, Out],
      Instruction::Halt => vec![],
    }
  }

  // Execute an instruction on a hardware
  fn exec(&self, arguments: &[Argument], hardware: &mut Hardware) {
    use Instruction::*;

    match self {
      Add | Mul | LessThan | Equals => {
        // Implements an instruction that consists of a binary operation that
        // writes its result to memory
        let mut write_binary_operation = |operation: &dyn Fn(i32, i32) -> i32| {
          let lhs = arguments[0].get_input();
          let rhs = arguments[1].get_input();
          let destination = arguments[2].get_output();

          hardware.write(destination, operation(lhs, rhs));
        };

        match self {
          Add => write_binary_operation(&|lhs, rhs| lhs + rhs),
          Mul => write_binary_operation(&|lhs, rhs| lhs * rhs),
          LessThan => write_binary_operation(&|lhs, rhs| if lhs < rhs { 1 } else { 0 }),
          Equals => write_binary_operation(&|lhs, rhs| if lhs == rhs { 1 } else { 0 }),
          _ => unreachable!("Missing match for binary instruction {:?}", self),
        }
      },
      JumpIfTrue | JumpIfFalse => {
        // Implements an instruction that performs an absolute jump if a
        // function applied on a value is true
        let mut jump_if = |condition: &dyn Fn(i32) -> bool| {
          let value = arguments[0].get_input();
          let destination = Address::from_value(arguments[1].get_input());

          if condition(value) {
            hardware.absolute_jump(destination);
          }
        };

        match self {
          JumpIfTrue => jump_if(&|value| value != 0),
          JumpIfFalse => jump_if(&|value| value == 0),
          _ => unreachable!("Missing match for conditional jump instruction {:?}", self),
        }
      },
      Prompt => {
        print!("PROMPT: ");
        io::stdout().flush().unwrap();

        let mut raw_input = String::new();
        io::stdin().read_line(&mut raw_input).unwrap();
        let input = raw_input.trim().parse::<i32>().unwrap();

        hardware.write(arguments[0].get_output(), input);
      },
      Print => {
        println!("PRINT: {}", arguments[0].get_input());
      },
      Halt => {
        println!("HALT");
      },
    }
  }
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
