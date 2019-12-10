use core::convert::{ TryInto, TryFrom };

mod hardware;
use hardware::*;

mod instruction;
use instruction::*;

pub struct Computer<'a> {
    hardware: Hardware<'a>,
    program_counter: Address,
    relative_base: Address,
}

impl<'a> Computer<'a> {
    pub fn new_with_terminal(program: Vec<i32>) -> Computer<'a> {
        Computer {
            hardware: Hardware::new_with_terminal(program),
            program_counter: 0,
            relative_base: 0,
        }
    }

    pub fn new
        <InputFn: 'a + FnMut() -> i32, OutputFn: 'a + FnMut(i32)>
        (program: Vec<i32>, input: InputFn, output: OutputFn) -> Computer<'a>
    {
        Computer {
            hardware: Hardware::new(program, input, output),
            program_counter: 0,
            relative_base: 0,
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

    fn move_relative_base(&mut self, change: i32) {
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
                                ArgumentMode::Relative => self.hardware.read(self.relative_base + Address::from_value(value)),
                            }
                        )
                    },
                    ArgumentType::Out => {
                        Argument::Out(
                            match mode {
                                ArgumentMode::Immediate => panic!("Out arguments cannot be immediate"),
                                ArgumentMode::Indexed => Address::from_value(value),
                                ArgumentMode::Relative => self.relative_base + Address::from_value(value),
                            }
                        )
                    },
                }
            }).collect();

        // Run instruction
        let register_change = instruction.exec(&arguments, &mut self.hardware);

        // An instruction may requests changes to certain registers as part of its operation, which
        // are now carried out

        if let Some(RegisterChange::ProgramCounter{ new_value }) = register_change {
            self.absolute_jump(new_value);
        } else {
            self.relative_jump_forward(1 + arguments.len());
        }

        if let Some(RegisterChange::RelativeBase{ change }) = register_change {
            self.move_relative_base(change);
        }


        instruction
    }
}

#[cfg(test)]
mod instructions {
    use super::*;

    fn step(program: Vec<i32>, expected_instr: Instruction, expected_memory: Vec<i32>) {
        let mut computer = Computer::new_with_terminal(program);
        let instruction = computer.step();

        assert_eq!(instruction, expected_instr);

        let actual_memory: Vec<_> = (0..expected_memory.len())
            .map(|index| computer.hardware.read(index)).collect();

        assert_eq!(actual_memory, expected_memory);
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

        let actual_memory: Vec<_> = (0..expected_memory.len())
            .map(|index| computer.hardware.read(index)).collect();

        assert_eq!(actual_memory, expected_memory);
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
