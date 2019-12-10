use std::collections::HashMap;
use std::io::{ self, Write };
use core::convert::{ TryInto };

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
    memory: HashMap<Address, i32>,
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
            memory: program.into_iter().enumerate().collect(),
            input: Box::new(input),
            output: Box::new(output),
        }
    }

    pub fn read(&self, location: Address) -> i32 {
        *self.memory.get(&location).unwrap_or(&0)
    }

    pub fn write(&mut self, location: Address, value: i32) {
        self.memory.insert(location, value);
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
