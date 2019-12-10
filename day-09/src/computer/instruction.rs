use core::convert::{ TryFrom };

use super::hardware::*;

// Different types of instructions
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Instruction {
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
pub enum ArgumentType {
    In,
    Out,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ArgumentMode {
    Immediate, // The value itself is the argument
    Indexed, // The value is the address of the argument
}

// In-values are just values, while out-values are addresses (where a value is stored).
// Note that this means that in-arguments in indexed mode must first be dereferenced
// to be used as an Argument.
#[derive(Debug)]
pub enum Argument {
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
    pub fn argument_types(&self) -> Vec<ArgumentType> {
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
    pub fn exec(&self, arguments: &[Argument], hardware: &mut Hardware) -> Option<Address> {
        use Instruction::*;

        let mut jump = None;

        match self {
            Add | Mul | LessThan | Equals => {
                // Implements an instruction that consists of a binary operation that
                // writes its result to memory
                let mut write_binary_operation = |operation: fn(i32, i32) -> i32| {
                    let lhs = arguments[0].get_input();
                    let rhs = arguments[1].get_input();
                    let destination = arguments[2].get_output();

                    hardware.write(destination, operation(lhs, rhs));
                };

                match self {
                    Add => write_binary_operation(|lhs, rhs| lhs + rhs),
                    Mul => write_binary_operation(|lhs, rhs| lhs * rhs),
                    LessThan => write_binary_operation(|lhs, rhs| if lhs < rhs { 1 } else { 0 }),
                    Equals => write_binary_operation(|lhs, rhs| if lhs == rhs { 1 } else { 0 }),
                    _ => unreachable!("Missing match for binary instruction {:?}", self),
                }
            },
            JumpIfTrue | JumpIfFalse => {
                // Implements an instruction that performs an absolute jump if a
                // function applied on a value is true
                let mut jump_if = |condition: fn(i32) -> bool| {
                    let value = arguments[0].get_input();
                    let destination = Address::from_value(arguments[1].get_input());

                    if condition(value) {
                        jump = Some(destination);
                    }
                };

                match self {
                    JumpIfTrue => jump_if(|value| value != 0),
                    JumpIfFalse => jump_if(|value| value == 0),
                    _ => unreachable!("Missing match for conditional jump instruction {:?}", self),
                }
            },
              Prompt => {
                    let input = hardware.from_input();
                    hardware.write(arguments[0].get_output(), input);
              },
              Print => {
                    hardware.to_output(arguments[0].get_input());
              },
              Halt => {},
        }

        jump
    }
}
