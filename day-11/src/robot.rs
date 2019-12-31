use core::convert::{TryFrom, TryInto};
use std::collections::HashMap;

use super::computer::*;

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

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black = 0,
    White,
}

impl TryFrom<hardware::Value> for Color {
    type Error = hardware::Value;

    fn try_from(x: hardware::Value) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(Color::Black),
            1 => Ok(Color::White),
            _ => Err(x),
        }
    }
}

pub struct Grid {
    pub colors: HashMap<Position, Color>,
}

impl Grid {
    fn new() -> Grid {
        Grid {
            colors: HashMap::new(),
        }
    }

    pub fn get(&self, position: &Position) -> Color {
        *self.colors.get(position).unwrap_or(&Color::White)
    }

    fn paint(&mut self, position: &Position, color: Color) {
        self.colors.insert(*position, color);
    }
}

enum State {
    ReadyToPaint,
    ReadyToMove,
}

struct Heading {
    x: i64,
    y: i64,
}

enum RotateDirection {
    Clockwise,
    CounterClockwise,
}

impl TryFrom<hardware::Value> for RotateDirection {
    type Error = hardware::Value;

    fn try_from(x: hardware::Value) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(RotateDirection::CounterClockwise),
            1 => Ok(RotateDirection::Clockwise),
            _ => Err(x),
        }
    }
}

impl Heading {
    fn rotate(&mut self, direction: RotateDirection) {
        let (cos, sin) = match direction {
            RotateDirection::Clockwise => {
                (0, -1) // cos(-pi/2), sin(-pi/2)
            }
            RotateDirection::CounterClockwise => {
                (0, 1) // cos(pi/2), sin(pi/2)
            }
        };

        let new_x = self.x * cos - self.y * sin;
        let new_y = self.x * sin + self.y * cos;

        self.x = new_x;
        self.y = new_y;
    }
}

pub struct Robot {
    position: Position,
    heading: Heading,
    pub grid: Grid,
    state: State,
}

impl Robot {
    pub fn new() -> Robot {
        Robot {
            position: Position::new(0, 0),
            grid: Grid::new(),
            state: State::ReadyToPaint,
            heading: Heading { x: 0, y: 1 },
        }
    }

    fn scan(&self) -> Color {
        self.grid.get(&self.position)
    }

    fn rotate(&mut self, direction: RotateDirection) {
        self.heading.rotate(direction);
    }

    fn advance(&mut self, speed: i64) {
        self.position.x += self.heading.x * speed;
        self.position.y += self.heading.y * speed;
    }
}

impl hardware::IO for Robot {
    fn input(&mut self) -> hardware::Value {
        self.scan() as hardware::Value
    }

    fn output(&mut self, value: hardware::Value) {
        match &self.state {
            State::ReadyToPaint => {
                self.grid.paint(&self.position, value.try_into().unwrap());

                self.state = State::ReadyToMove;
            }
            State::ReadyToMove => {
                let direction = value.try_into().unwrap();
                self.rotate(direction);
                self.advance(1);

                self.state = State::ReadyToPaint;
            }
        }
    }
}
