use core::convert::{ TryFrom, TryInto };
use std::io::{ self, Write };

pub type Address = usize;

pub trait AddressLike {
    fn from_value(value: i32) -> Address;
}

impl AddressLike for Address {
    fn from_value(value: i32) -> Address {
        value.try_into().expect("Value is not an address")
    }
}

pub struct Hardware<'a> {
    memory: Vec<i32>,
    input: Box<dyn 'a + FnMut() -> i32>,
    output: Box<dyn 'a + FnMut(i32)>,
}

impl<'a> Hardware<'a> {
    pub fn new_with_terminal(program: Vec<i32>) -> Hardware<'a> {
        Hardware::new(program, Hardware::prompt, Hardware::print)
    }

    pub fn new
        <OutputFn: 'a + FnMut(i32), InputFn: 'a + FnMut() -> i32>
        (program: Vec<i32>, input: InputFn, output: OutputFn) -> Hardware<'a>
    {
        Hardware {
            memory: program,
            input: Box::new(input),
            output: Box::new(output),
        }
    }

    pub fn read(&self, location: Address) -> i32 {
        self.memory[location]
    }

    pub fn write(&mut self, location: Address, value: i32) {
        self.memory[location] = value;
    }

    pub fn from_input(&mut self) -> i32 {
        (self.input)()
    }

    pub fn to_output(&mut self, value: i32) {
        (self.output)(value);
    }

    fn prompt() -> i32 {
        print!("PROMPT: ");
        io::stdout().flush().unwrap();

        let mut raw_input = String::new();
        io::stdin().read_line(&mut raw_input).unwrap();
        raw_input.trim().parse::<i32>().unwrap()
    }

    fn print(value: i32) {
        println!("PRINT: {}", value);
    }
}

pub struct Computer<'a> {
    hardware: Hardware<'a>,
    program_counter: Address,
}

impl<'a> Computer<'a> {
    pub fn new_with_terminal(program: Vec<i32>) -> Computer<'a> {
        Computer {
            hardware: Hardware::new_with_terminal(program),
            program_counter: 0,
        }
    }

    pub fn new
        <InputFn: 'a + FnMut() -> i32, OutputFn: 'a + FnMut(i32)>
        (program: Vec<i32>, input: InputFn, output: OutputFn) -> Computer<'a>
    {
        Computer {
            hardware: Hardware::new(program, input, output),
            program_counter: 0,
        }
    }

    fn next_instruction(&self) -> Instruction {
        self.hardware.read(self.program_counter).try_into().expect("Unknown opcode")
    }

    fn argument(&self, nth: usize) -> (i32, ArgumentMode) {
        (self.argument_value(nth), self.argument_mode(nth))
    }

    // Reads the value of the nth argument for the current instruction (0-based)
    fn argument_value(&self, nth: usize) -> i32 {
        self.hardware.read(self.program_counter + nth + 1)
    }

    fn argument_mode(&self, nth: usize) -> ArgumentMode {
        // In a string representation, the last two digits of the instruction value
        // are the opcode (i.e. opcodes go from 0 to 99). We first remove those.
        let mode_indicators = self.hardware.read(self.program_counter) / 100;

        // Then, the mode for the 0-based nth argument is the nth digit from the
        // right: the first argument is the units, second tenths, and so on.
        let mode_indicator = (mode_indicators / 10_i32.pow(nth.try_into().unwrap())) % 10;

        mode_indicator.try_into().unwrap()
    }

    fn relative_jump_forward(&mut self, distance: usize) {
        self.program_counter += distance;
    }

    fn absolute_jump(&mut self, destination: usize) {
        self.program_counter = destination;
    }

    pub fn run(&mut self) {
        loop {
            if self.step() == Instruction::Halt {
                break;
            }
        }
    }

    fn step(&mut self) -> Instruction {
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
                                ArgumentMode::Indexed => self.hardware.read(Address::from_value(value)),
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
        let jump = instruction.exec(&arguments, &mut self.hardware);

        // Change program counter, advancing over the opcode and its arguments if the instruction
        // didn't request a jump, or jumping if it did
        if jump.is_none() {
            self.relative_jump_forward(1 + arguments.len());
        } else {
            self.absolute_jump(jump.unwrap())
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
    fn exec(&self, arguments: &[Argument], hardware: &mut Hardware) -> Option<Address> {
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

#[cfg(test)]
mod instructions {
    use super::*;

    fn step(program: Vec<i32>, expected_instr: Instruction, expected_memory: Vec<i32>) {
        let mut computer = Computer::new_with_terminal(program);
        let instruction = computer.step();

        assert_eq!(instruction, expected_instr);
        assert_eq!(computer.hardware.memory, expected_memory);
    }

    #[test]
    fn add() { step(vec![1,0,0,0], Instruction::Add, vec![2,0,0,0]) }

    #[test]
    fn mul() { step(vec![2,3,0,3], Instruction::Mul, vec![2,3,0,6]) }
}

#[cfg(test)]
mod programs {
    use super::*;

    fn run(program: Vec<i32>, expected_memory: Vec<i32>) {
        let mut computer = Computer::new_with_terminal(program);
        computer.run();
        assert_eq!(computer.hardware.memory, expected_memory);
    }

    #[test]
    fn add() { run(vec![1,1,1,4,99,5,6,0,99], vec![30,1,1,4,2,5,6,0,99]) }

    #[test]
    fn negative_add() { run(vec![1101,100,-1,4,0], vec![1101,100,-1,4,99]) }

    #[test]
    fn multiply() { run(vec![2,4,4,5,99,0], vec![2,4,4,5,99,9801]) }

    #[test]
    fn multiply_with_argument_modes() { run(vec![1002,4,3,4,33], vec![1002,4,3,4,99]) }
}

#[cfg(test)]
mod programs_io {
    use super::*;

    fn run_io<InputFn: FnMut() -> i32>(program: &Vec<i32>, input: InputFn, expected_output: Vec<i32>) {
        let mut output_values = Vec::new();
        {
            // Scope mutable access to output_values
            let output = |value| output_values.push(value);

            let mut computer = Computer::new(program.clone(), input, output);
            computer.run();
        }
        assert_eq!(output_values, expected_output);
    }

    #[test]
    fn basic_io() { run_io(&vec![3,0,4,0,99], || 5, vec![5]) }

    #[test]
    fn io_equal_position_mode() {
        // Test if input equals 8
        let program = vec![3,9,8,9,10,9,4,9,99,-1,8];
        run_io(&program, || 7, vec![0]);
        run_io(&program, || 8, vec![1]);
        run_io(&program, || 9, vec![0]);
    }

    #[test]
    fn io_equal_immediate_mode() {
        // Test if input equals 8
        let program = vec![3,3,1108,-1,8,3,4,3,99];
        run_io(&program, || 7, vec![0]);
        run_io(&program, || 8, vec![1]);
        run_io(&program, || 9, vec![0]);
    }

    #[test]
    fn io_less_than_position_mode() {
        // Test if input is less than 8
        let program = vec![3,9,7,9,10,9,4,9,99,-1,8];
        run_io(&program, || 6, vec![1]);
        run_io(&program, || 7, vec![1]);
        run_io(&program, || 8, vec![0]);
    }

    #[test]
    fn io_less_than_immediate_mode() {
        // Test if input is less than 8
        let program = vec![3,3,1107,-1,8,3,4,3,99];
        run_io(&program, || 6, vec![1]);
        run_io(&program, || 7, vec![1]);
        run_io(&program, || 8, vec![0]);
    }

    #[test]
    fn io_jump_position_mode() {
        // Test if input is true
        let program = vec![3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9];
        run_io(&program, || 0, vec![0]);
        run_io(&program, || 1, vec![1]);
        run_io(&program, || 2, vec![1]);
    }

    #[test]
    fn io_jump_immediate_mode() {
        // Test if input is true
        let program = vec![3,3,1105,-1,9,1101,0,0,12,4,12,99,1];
        run_io(&program, || 0, vec![0]);
        run_io(&program, || 1, vec![1]);
        run_io(&program, || 2, vec![1]);
    }

    #[test]
    fn io_complex() {
        // Test if input is less, equal or greater to 8
        let program = vec![
            3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,
            1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,
            999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99
        ];

        run_io(&program, || 6, vec![999]);
        run_io(&program, || 7, vec![999]);
        run_io(&program, || 8, vec![1000]);
        run_io(&program, || 9, vec![1001]);
        run_io(&program, || 10, vec![1001]);
    }
}
