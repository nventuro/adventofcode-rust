use core::convert::{TryFrom, TryInto};
use std::io::{self, Write};

pub mod hardware;
use hardware::*;

mod instruction;
use instruction::*;

struct Console;

impl IO for Console {
    fn input(&mut self) -> Value {
        print!("PROMPT: ");
        io::stdout().flush().unwrap();

        let mut raw_input = String::new();
        io::stdin().read_line(&mut raw_input).unwrap();
        raw_input.trim().parse::<Value>().unwrap()
    }

    fn output(&mut self, value: Value) {
        println!("PRINT: {}", value);
    }
}

static mut CONSOLE: Console = Console;

pub struct Computer<'hw> {
    hardware: Hardware<'hw>,
    program_counter: Address,
    relative_base: Address,
}

impl Computer<'_> {
    pub fn new(program: Vec<Value>, io: Option<&mut dyn IO>) -> Computer {
        let hardware = Hardware::new(program, io.unwrap_or(
            // We need unsafe to get reference to the static console, but
            // multithreaded access to it is not an issue
            unsafe { &mut CONSOLE }
        ));

        Computer {
            hardware,
            program_counter: 0,
            relative_base: 0,
        }
    }

    fn next_instruction(&self) -> Instruction {
        self.hardware
            .read(self.program_counter)
            .try_into()
            .expect("Unknown opcode")
    }

    fn argument(&self, nth: usize) -> (Value, ArgumentMode) {
        (self.argument_value(nth), self.argument_mode(nth))
    }

    // Reads the value of the nth argument for the current instruction (0-based)
    fn argument_value(&self, nth: usize) -> Value {
        self.hardware.read(self.program_counter + nth + 1)
    }

    fn argument_mode(&self, nth: usize) -> ArgumentMode {
        // In a string representation, the last two digits of the instruction value
        // are the opcode (i.e. opcodes go from 0 to 99). We first remove those.
        let mode_indicators = self.hardware.read(self.program_counter) / 100;

        // Then, the mode for the 0-based nth argument is the nth digit from the
        // right: the first argument is the units, second tenths, and so on.
        let mode_indicator = (mode_indicators / 10_i64.pow(nth.try_into().unwrap())) % 10;

        mode_indicator.try_into().unwrap()
    }

    fn relative_jump_forward(&mut self, distance: usize) {
        self.program_counter += distance;
    }

    fn absolute_jump(&mut self, destination: usize) {
        self.program_counter = destination;
    }

    fn move_relative_base(&mut self, change: Value) {
        if change > 0 {
            self.relative_base += usize::try_from(change).unwrap();
        } else {
            self.relative_base -= usize::try_from(-change).unwrap();
        }
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
        let arguments: Vec<Argument> = instruction
            .argument_types()
            .iter()
            .enumerate()
            .map(|(index, argument_type)| {
                let (value, mode) = self.argument(index);

                match argument_type {
                    ArgumentType::In => Argument::In(match mode {
                        ArgumentMode::Immediate => value,
                        ArgumentMode::Indexed => self.hardware.read(Address::from_value(value)),
                        ArgumentMode::Relative => self.hardware.read(Address::from_value(
                            Value::try_from(self.relative_base).unwrap() + value,
                        )),
                    }),
                    ArgumentType::Out => Argument::Out(match mode {
                        ArgumentMode::Immediate => panic!("Out arguments cannot be immediate"),
                        ArgumentMode::Indexed => Address::from_value(value),
                        ArgumentMode::Relative => Address::from_value(
                            Value::try_from(self.relative_base).unwrap() + value,
                        ),
                    }),
                }
            })
            .collect();

        // Run instruction
        let register_change = instruction.exec(&arguments, &mut self.hardware);

        // An instruction may requests changes to certain registers as part of its operation, which
        // are now carried out

        if let Some(RegisterChange::ProgramCounter { new_value }) = register_change {
            self.absolute_jump(new_value);
        } else {
            self.relative_jump_forward(1 + arguments.len());
        }

        if let Some(RegisterChange::RelativeBase { change }) = register_change {
            self.move_relative_base(change);
        }

        instruction
    }
}

#[cfg(test)]
mod instructions {
    use super::*;

    fn step(program: Vec<Value>, expected_instr: Instruction, expected_memory: Vec<Value>) {
        let mut computer = Computer::new(program, None);
        let instruction = computer.step();

        assert_eq!(instruction, expected_instr);

        let actual_memory: Vec<_> = (0..expected_memory.len())
            .map(|index| computer.hardware.read(index))
            .collect();

        assert_eq!(actual_memory, expected_memory);
    }

    #[test]
    fn add() {
        step(vec![1, 0, 0, 0], Instruction::Add, vec![2, 0, 0, 0])
    }

    #[test]
    fn mul() {
        step(vec![2, 3, 0, 3], Instruction::Mul, vec![2, 3, 0, 6])
    }
}

#[cfg(test)]
mod programs {
    use super::*;

    fn run(program: Vec<Value>, expected_memory: Vec<Value>) {
        let mut computer = Computer::new(program, None);
        computer.run();

        let actual_memory: Vec<_> = (0..expected_memory.len())
            .map(|index| computer.hardware.read(index))
            .collect();

        assert_eq!(actual_memory, expected_memory);
    }

    #[test]
    fn add() {
        run(
            vec![1, 1, 1, 4, 99, 5, 6, 0, 99],
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99],
        )
    }

    #[test]
    fn negative_add() {
        run(vec![1101, 100, -1, 4, 0], vec![1101, 100, -1, 4, 99])
    }

    #[test]
    fn multiply() {
        run(vec![2, 4, 4, 5, 99, 0], vec![2, 4, 4, 5, 99, 9801])
    }

    #[test]
    fn multiply_with_argument_modes() {
        run(vec![1002, 4, 3, 4, 33], vec![1002, 4, 3, 4, 99])
    }
}

#[cfg(test)]
mod programs_io {
    use super::*;

    struct FixedIO {
        input_value: Value,
        output_values: Vec<Value>,
    }

    impl FixedIO {
        fn new(input: Value) -> FixedIO {
            FixedIO {
                input_value: input,
                output_values: Vec::<_>::new(),
            }
        }
    }

    impl IO for FixedIO {
        fn input(&mut self) -> Value {
            self.input_value
        }

        fn output(&mut self, value: Value) {
            self.output_values.push(value);
        }
    }

    fn run_io(program: &Vec<Value>, input_value: Value, expected_output: &Vec<Value>) {
        let mut fixed_io = FixedIO::new(input_value);
        let mut computer = Computer::new(program.clone(), Some(&mut fixed_io));
        computer.run();

        assert_eq!(fixed_io.output_values, *expected_output);
    }

    #[test]
    fn basic_io() {
        run_io(&vec![3, 0, 4, 0, 99], 5, &vec![5])
    }

    #[test]
    fn io_equal_position_mode() {
        // Test if input equals 8
        let program = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        run_io(&program, 7, &vec![0]);
        run_io(&program, 8, &vec![1]);
        run_io(&program, 9, &vec![0]);
    }

    #[test]
    fn io_equal_immediate_mode() {
        // Test if input equals 8
        let program = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];
        run_io(&program, 7, &vec![0]);
        run_io(&program, 8, &vec![1]);
        run_io(&program, 9, &vec![0]);
    }

    #[test]
    fn io_less_than_position_mode() {
        // Test if input is less than 8
        let program = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        run_io(&program, 6, &vec![1]);
        run_io(&program, 7, &vec![1]);
        run_io(&program, 8, &vec![0]);
    }

    #[test]
    fn io_less_than_immediate_mode() {
        // Test if input is less than 8
        let program = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];
        run_io(&program, 6, &vec![1]);
        run_io(&program, 7, &vec![1]);
        run_io(&program, 8, &vec![0]);
    }

    #[test]
    fn io_jump_position_mode() {
        // Test if input is true
        let program = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        run_io(&program, 0, &vec![0]);
        run_io(&program, 1, &vec![1]);
        run_io(&program, 2, &vec![1]);
    }

    #[test]
    fn io_jump_immediate_mode() {
        // Test if input is true
        let program = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        run_io(&program, 0, &vec![0]);
        run_io(&program, 1, &vec![1]);
        run_io(&program, 2, &vec![1]);
    }

    #[test]
    fn io_complex() {
        // Test if input is less, equal or greater to 8
        let program = vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        run_io(&program, 6, &vec![999]);
        run_io(&program, 7, &vec![999]);
        run_io(&program, 8, &vec![1000]);
        run_io(&program, 9, &vec![1001]);
        run_io(&program, 10, &vec![1001]);
    }

    #[test]
    fn large_memory_relative_base_quine() {
        let program = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];

        run_io(&program, 0, &program);
    }

    #[test]
    fn compute_large_number() {
        let program = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];

        run_io(&program, 0, &vec![1219070632396864]);
    }

    #[test]
    fn output_large_number() {
        let program = vec![104, 1125899906842624, 99];

        run_io(&program, 0, &vec![1125899906842624]);
    }
}
