use core::convert::{TryFrom, TryInto};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time;

use super::computer::*;

extern crate termion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub fn new(x: i64, y: i64) -> Position {
        Position { x, y }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Object {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl Object {
    fn draw(self) {
        match self {
            Object::Empty => print!(" "),
            Object::Wall => print!("|"),
            Object::Block => print!("#"),
            Object::Paddle => print!("="),
            Object::Ball => print!("o"),
        }
    }
}

impl TryFrom<hardware::Value> for Object {
    type Error = hardware::Value;

    fn try_from(x: hardware::Value) -> Result<Self, Self::Error> {
        use Object::*;
        match x {
            0 => Ok(Empty),
            1 => Ok(Wall),
            2 => Ok(Block),
            3 => Ok(Paddle),
            4 => Ok(Ball),
            _ => Err(x),
        }
    }
}

enum Data {
    Object(Object),
    Score(i64),
}

struct Buffer {
    data: Vec<hardware::Value>,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            data: Vec::with_capacity(3),
        }
    }

    fn insert(&mut self, value: hardware::Value) -> Option<(Position, Data)> {
        self.data.push(value);

        if self.data.len() == 3 {
            let position = Position::new(self.data[0], self.data[1]);

            let data = if position.x == -1 && position.y == 0 {
                Data::Score(self.data[2])
            } else {
                Data::Object(self.data[2].try_into().unwrap())
            };

            self.data.clear();

            Some((position, data))
        } else {
            None
        }
    }
}

pub struct GameDisplay {
    pub score: hardware::Value,
    objects: HashMap<Position, Object>,
    buffer: Buffer,
}

impl GameDisplay {
    pub fn new() -> GameDisplay {
        print!("{}", termion::cursor::Save);

        GameDisplay {
            buffer: Buffer::new(),
            score: 0,
            objects: HashMap::new(),
        }
    }

    fn draw(&self) {
        print!("{}", termion::cursor::Restore);

        let max_x = self.objects.keys().map(|pos| pos.x).max().unwrap();
        let min_x = self.objects.keys().map(|pos| pos.x).min().unwrap();

        let max_y = self.objects.keys().map(|pos| pos.y).max().unwrap();
        let min_y = self.objects.keys().map(|pos| pos.y).min().unwrap();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let object = self
                    .objects
                    .get(&Position::new(x, y))
                    .unwrap_or(&Object::Empty);

                object.draw();
            }

            println!("");
            io::stdout().flush().unwrap();
        }

        println!("Score: {:?}", self.score);
    }
}

impl hardware::IO for GameDisplay {
    fn input(&mut self) -> hardware::Value {
        self.draw();
        thread::sleep(time::Duration::from_millis(50));

        let (ball_position, _) = self
            .objects
            .iter()
            .find(|(_position, object)| **object == Object::Ball)
            .unwrap();

        let (paddle_position, _) = self
            .objects
            .iter()
            .find(|(_position, object)| **object == Object::Paddle)
            .unwrap();

        if ball_position.x > paddle_position.x {
            1
        } else if ball_position.x < paddle_position.x {
            -1
        } else {
            0
        }
    }

    fn output(&mut self, value: hardware::Value) {
        if let Some((position, data)) = self.buffer.insert(value) {
            match data {
                Data::Object(object) => {
                    self.objects.insert(position, object);
                }
                Data::Score(score) => {
                    self.score = score;
                }
            }
        }
    }
}
