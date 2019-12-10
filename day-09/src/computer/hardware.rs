use std::collections::HashMap;
use std::io::{ self, Write };
use core::convert::{ TryInto };

pub type Address = usize;
pub type Value = i64;

pub trait AddressLike {
    fn from_value(value: Value) -> Address;
}

impl AddressLike for Address {
    fn from_value(value: Value) -> Address {
        value.try_into().expect("Value is not an address")
    }
}

pub struct Hardware<'a> {
    memory: HashMap<Address, Value>,
    input: Box<dyn 'a + FnMut() -> Value>,
    output: Box<dyn 'a + FnMut(Value)>,
}

impl<'a> Hardware<'a> {
    pub fn new_with_terminal(program: Vec<Value>) -> Hardware<'a> {
        Hardware::new(program, Hardware::prompt, Hardware::print)
    }

    pub fn new
        <OutputFn: 'a + FnMut(Value), InputFn: 'a + FnMut() -> Value>
        (program: Vec<Value>, input: InputFn, output: OutputFn) -> Hardware<'a>
    {
        Hardware {
            memory: program.into_iter().enumerate().collect(),
            input: Box::new(input),
            output: Box::new(output),
        }
    }

    pub fn read(&self, location: Address) -> Value {
        *self.memory.get(&location).unwrap_or(&0)
    }

    pub fn write(&mut self, location: Address, value: Value) {
        self.memory.insert(location, value);
    }

    pub fn from_input(&mut self) -> Value {
        (self.input)()
    }

    pub fn to_output(&mut self, value: Value) {
        (self.output)(value);
    }

    fn prompt() -> Value {
        print!("PROMPT: ");
        io::stdout().flush().unwrap();

        let mut raw_input = String::new();
        io::stdin().read_line(&mut raw_input).unwrap();
        raw_input.trim().parse::<Value>().unwrap()
    }

    fn print(value: Value) {
        println!("PRINT: {}", value);
    }
}
