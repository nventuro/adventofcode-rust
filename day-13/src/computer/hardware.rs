use core::convert::TryInto;
use std::collections::HashMap;

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

pub trait IO {
    fn input(&mut self) -> Value;
    fn output(&mut self, _: Value);
}

pub struct Hardware<'hw> {
    memory: HashMap<Address, Value>,
    io: &'hw mut dyn IO,
}

impl Hardware<'_> {
    pub fn new(program: Vec<Value>, io: &mut dyn IO) -> Hardware {
        Hardware {
            memory: program.into_iter().enumerate().collect(),
            io,
        }
    }

    pub fn read(&self, location: Address) -> Value {
        *self.memory.get(&location).unwrap_or(&0)
    }

    pub fn write(&mut self, location: Address, value: Value) {
        self.memory.insert(location, value);
    }

    pub fn reset(&mut self) {
        self.memory.clear();
    }

    pub fn from_input(&mut self) -> Value {
        self.io.input()
    }

    pub fn to_output(&mut self, value: Value) {
        self.io.output(value);
    }
}
