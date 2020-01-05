use core::convert::{TryFrom, TryInto};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{self, Write};

use super::computer::*;
extern crate termion;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    Up = 1,
    Down = 2,
    Left = 3,
    Right = 4,
}

impl Direction {
    fn opposite(&self) -> Direction {
        use Direction::*;

        match self {
            Up => Down,
            Down => Up,
            Right => Left,
            Left => Right,
        }
    }

    fn directions() -> Vec<Direction> {
        use Direction::*;
        vec![Left, Down, Right, Up]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    x: i64,
    y: i64,
}

impl Position {
    pub fn new(x: i64, y: i64) -> Position {
        Position { x, y }
    }

    fn plus_direction(&self, direction: Direction) -> Position {
        use Direction::*;

        match direction {
            Up => Position::new(self.x, self.y + 1),
            Down => Position::new(self.x, self.y - 1),
            Left => Position::new(self.x - 1, self.y),
            Right => Position::new(self.x + 1, self.y),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PositionState {
    Unknown,
    Empty,
    Wall,
    OxygenTank,
    Oxygenated,
}

impl PositionState {
    pub fn draw(&self) {
        match self {
            PositionState::Unknown => print!(" "),
            PositionState::Empty => print!("."),
            PositionState::Wall => print!("#"),
            PositionState::OxygenTank => print!("O"),
            PositionState::Oxygenated => print!("o"),
        }
    }
}

pub struct World(HashMap<Position, PositionState>);

impl World {
    fn new() -> World {
        World(
            [(Position::new(0, 0), PositionState::Empty)]
                .iter()
                .cloned()
                .collect(),
        )
    }

    pub fn get(&self, position: &Position) -> PositionState {
        *self.0.get(position).unwrap_or(&PositionState::Unknown)
    }

    fn set(&mut self, position: Position, state: PositionState) {
        self.0.insert(position, state);
    }

    pub fn limits(&self) -> (i64, i64, i64, i64) {
        let min_x = self.0.keys().map(|pos| pos.x).min().unwrap();
        let max_x = self.0.keys().map(|pos| pos.x).max().unwrap();

        let min_y = self.0.keys().map(|pos| pos.y).min().unwrap();
        let max_y = self.0.keys().map(|pos| pos.y).max().unwrap();

        (
            std::cmp::min(min_x, -20),
            std::cmp::max(max_x, 20),
            std::cmp::min(min_y, -20),
            std::cmp::max(max_y, 20),
        )
    }

    pub fn oxygenate_step(&mut self) -> bool {
        let oxygen_positions = self
            .0
            .iter()
            .filter(|(_position, state)| {
                **state == PositionState::OxygenTank || **state == PositionState::Oxygenated
            })
            .map(|(position, _state)| *position)
            .collect::<Vec<_>>();

        let mut oxygenated = false;

        for position in oxygen_positions {
            for adjacent in Direction::directions()
                .iter()
                .map(|direction| position.plus_direction(*direction))
            {
                if self.get(&adjacent) == PositionState::Empty {
                    self.set(adjacent, PositionState::Oxygenated);
                    oxygenated = true;
                }
            }
        }

        oxygenated
    }
}

#[derive(Debug, PartialEq)]
enum SensorReading {
    Wall,
    Empty,
    Goal,
}

impl TryFrom<hardware::Value> for SensorReading {
    type Error = hardware::Value;

    fn try_from(x: hardware::Value) -> Result<Self, Self::Error> {
        use SensorReading::*;
        match x {
            0 => Ok(Wall),
            1 => Ok(Empty),
            2 => Ok(Goal),
            _ => Err(x),
        }
    }
}

enum RobotState {
    Probing {
        directions: Vec<Direction>,
        backtrack: Option<Direction>,
    },
    Moving {
        path: Vec<Direction>,
    },
}

struct Robot {
    world: World,
    position: Position,

    state: RobotState,
    path: Vec<Direction>,
    plan: VecDeque<Vec<Direction>>,
    done: bool,
}

impl Robot {
    fn new() -> Robot {
        Robot {
            world: World::new(),
            position: Position::new(0, 0),

            state: RobotState::Probing {
                directions: Direction::directions(),
                backtrack: None,
            },

            path: Vec::new(),
            plan: VecDeque::new(),
            done: false,
        }
    }

    fn draw(&self) {
        print!("{}", termion::cursor::Save);

        let (min_x, max_x, min_y, max_y) = self.world.limits();
        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                let position = Position::new(x, y);

                if position != self.position {
                    self.world.get(&position).draw();
                } else {
                    print!("@");
                }
            }

            println!("");
            io::stdout().flush().unwrap();
        }
        print!("{}", termion::cursor::Restore);
    }

    fn next_plan(&mut self) -> Vec<Direction> {
        while !self.plan.is_empty() {
            let plan = self.plan.pop_front().unwrap();
            if self.is_valid_plan(&plan) {
                return plan;
            }
        }

        self.done = true;
        return vec![];
    }

    fn is_valid_plan(&self, plan: &[Direction]) -> bool {
        let probe_position = plan
            .iter()
            .fold(Position::new(0, 0), |position, direction| {
                position.plus_direction(*direction)
            });

        Direction::directions()
            .iter()
            .map(|direction| probe_position.plus_direction(*direction))
            .filter(|position| self.world.get(position) == PositionState::Unknown)
            .count()
            > 0
    }

    fn process(&mut self, movement: Direction, reading: SensorReading) -> Direction {
        if reading != SensorReading::Wall {
            self.update_position(movement);
        }

        self.process_reading(movement, reading);
        self.evaluate_state();
        self.next_direction()
    }

    fn update_position(&mut self, movement: Direction) {
        if !self.path.is_empty() && *self.path.last().unwrap() == movement.opposite() {
            self.path.pop();
        } else {
            self.path.push(movement);
        }

        self.position = self.position.plus_direction(movement);
    }

    fn process_reading(&mut self, movement: Direction, reading: SensorReading) {
        match &mut self.state {
            RobotState::Moving { path } => {
                assert_eq!(reading, SensorReading::Empty);

                path.pop().unwrap();
            }
            RobotState::Probing {
                directions,
                backtrack,
            } => {
                if backtrack.is_some() {
                    *backtrack = None;
                    return;
                }

                directions.pop().unwrap();

                match reading {
                    SensorReading::Wall => {
                        self.world
                            .set(self.position.plus_direction(movement), PositionState::Wall);
                    }
                    SensorReading::Empty | SensorReading::Goal => {
                        if self.world.get(&self.position) == PositionState::Unknown {
                            if reading == SensorReading::Empty {
                                self.world.set(self.position, PositionState::Empty);
                            } else {
                                self.world.set(self.position, PositionState::OxygenTank);
                                println!("Found tank! {:?} steps required", self.path.len());
                            }

                            self.plan.push_back(self.path.clone());
                        }

                        *backtrack = Some(movement.opposite())
                    }
                }
            }
        }
    }

    fn evaluate_state(&mut self) {
        match &mut self.state {
            RobotState::Moving { path } => {
                if path.is_empty() {
                    let directions = Direction::directions()
                        .into_iter()
                        .filter(|direction| {
                            self.world.get(&self.position.plus_direction(*direction))
                                == PositionState::Unknown
                        })
                        .collect::<Vec<_>>();

                    self.state = RobotState::Probing {
                        directions,
                        backtrack: None,
                    };
                }
            }
            RobotState::Probing {
                directions,
                backtrack,
            } => {
                if directions.is_empty() && backtrack.is_none() {
                    let next_plan = self.next_plan();

                    let return_path = self.path.clone();
                    let advance_path = next_plan.clone();

                    let mut match_count = 0;
                    for i in 0..std::cmp::min(return_path.len(), advance_path.len()) {
                        if return_path[i] != advance_path[i] {
                            break;
                        }

                        match_count += 1;
                    }

                    self.state = RobotState::Moving {
                        path: return_path[match_count..]
                            .iter()
                            .map(|direction| direction.opposite())
                            .rev()
                            .chain(advance_path[match_count..].to_vec().into_iter())
                            .rev()
                            .collect(),
                    };
                }
            }
        }
    }

    fn next_direction(&self) -> Direction {
        match &self.state {
            RobotState::Moving { path } => *path.last().unwrap(),
            RobotState::Probing {
                directions,
                backtrack,
            } => {
                if let Some(direction) = backtrack {
                    *direction
                } else {
                    *directions.last().unwrap()
                }
            }
        }
    }
}

pub struct RobotIO {
    robot: Robot,
    movement: Direction,
}

impl RobotIO {
    pub fn new() -> RobotIO {
        let robot = Robot::new();
        let movement = match &robot.state {
            RobotState::Probing {
                directions,
                backtrack: _,
            } => *directions.last().unwrap(),
            _ => panic!(),
        };

        RobotIO { robot, movement }
    }

    pub fn get_world(self) -> World {
        self.robot.world
    }
}

impl hardware::IO for RobotIO {
    fn input(&mut self) -> hardware::Value {
        if self.robot.done {
            0
        } else {
            self.movement as hardware::Value
        }
    }

    fn output(&mut self, value: hardware::Value) {
        //std::thread::sleep(std::time::Duration::from_millis(10));
        self.robot.draw();

        let reading = value.try_into().unwrap();
        self.movement = self.robot.process(self.movement, reading);
    }
}
