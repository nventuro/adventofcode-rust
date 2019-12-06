use std::fs;
use std::cmp;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    process(contents);
}

fn process(input: String) {
    // Parse input and get movements for each wire

    let mut movements_per_wire = Vec::<Vec::<Movement>>::new();
    for line in input.split_whitespace() {
        if !line.is_empty() {
            // New wire
            movements_per_wire.push(parse_wire_description(line));
        }
    }

    // Create wire vectors
    let mut vectors_per_wire = Vec::<Vec::<Vector>>::new();
    for wire_movements in movements_per_wire {
        let mut wire_vectors = Vec::<Vector>::new();

        for movement in wire_movements {
            match wire_vectors.last() {
                None => wire_vectors.push(Vector::new(Point::new(0, 0), movement)),
                Some(last) => {
                    let last_end = last.end(); // prevent mutable_borrow_reservation_conflict
                    wire_vectors.push(Vector::new(last_end, movement))
                },
            }
        }

        vectors_per_wire.push(wire_vectors);
    }

    assert!(vectors_per_wire.len() >= 2);

    for first_wire_vector in &vectors_per_wire[0] {
        for other_wire_vectors in &vectors_per_wire[1..] {
            for other_wire_vector in other_wire_vectors {
                if first_wire_vector.intersects(other_wire_vector) {
                    println!("match!");
                }
            }
        }
    }
}

fn parse_wire_description(description: &str) -> Vec<Movement> {
    description.split(",")
        .map(|x| Movement::from_string(x.trim()))
        .collect()
}

#[derive(Debug)]
#[derive(PartialEq)]
enum Movement {
    Horizontal(i32),
    Vertical(i32),
}

impl Movement {
    fn from_string(text: &str) -> Movement {
        assert!(text.len() >= 2);

        let direction = text.chars().next().unwrap();
        let value = text[1..].parse::<i32>().unwrap();

        assert!(value >= 1);

        match direction {
            // First quadrant coordinates: right and up are positive
            'R' => Movement::Horizontal(value),
            'L' => Movement::Horizontal(-value),
            'U' => Movement::Vertical(value),
            'D' => Movement::Vertical(-value),
            _ => panic!("Invalid direction: '{}'", ),
        }
    }
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct Vector {
    start: Point,
    length: i32,
    direction: Direction,
}


impl Vector {
    fn new(start: Point, movement: Movement) -> Vector {
        match movement {
            Movement::Horizontal(length) => Vector{ start, length, direction: Direction::Horizontal },
            Movement::Vertical(length) => Vector{ start, length, direction: Direction::Vertical },
        }
    }

    fn end(&self) -> Point {
        match self.direction {
            Direction::Horizontal =>
                Point::new(self.start.x + self.length, self.start.y),

            Direction::Vertical =>
                Point::new(self.start.x, self.start.y + self.length),
        }
    }

    fn intersects(&self, other: &Vector) -> bool {
        use Direction::*;

        match (&self.direction, &other.direction) {
            (Horizontal, Horizontal) => {
                self.start.y == other.start.y && (
                    // We need to check for both containing one point of the
                    // other to account for scenarios where one vector contains
                    // the other one completely
                    self.contains_x(other.start.x) ||
                    self.contains_x(other.end().x) ||
                    other.contains_x(self.start.x) ||
                    other.contains_x(self.end().x)
                )
            },

            (Horizontal, Vertical) => {
                self.contains_x(other.start.x) && other.contains_y(self.start.y)
            },

            (Vertical, Horizontal) => {
                other.intersects(self)
            },

            (Vertical, Vertical) => {
                // Same as Horizontal, Horizontal
                self.start.x == other.start.x && (
                    self.contains_y(other.start.y) ||
                    self.contains_y(other.end().y) ||
                    other.contains_y(self.start.y) ||
                    other.contains_y(self.end().y)
                )
            },
        }
    }

    fn contains_x(&self, x: i32) -> bool {
        assert_eq!(self.direction, Direction::Horizontal);

        x >= cmp::min(self.start.x, self.end().x) &&
        x <= cmp::max(self.start.x, self.end().x)
    }

    fn contains_y(&self, y: i32) -> bool {
        assert_eq!(self.direction, Direction::Vertical);

        y >= cmp::min(self.start.y, self.end().y) &&
        y <= cmp::max(self.start.y, self.end().y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_from_string() {
        assert_eq!(Movement::from_string("U1"), Movement::Vertical(1));
        assert_eq!(Movement::from_string("D123"), Movement::Vertical(-123));
        assert_eq!(Movement::from_string("R9876"), Movement::Horizontal(9876));
        assert_eq!(Movement::from_string("L2"), Movement::Horizontal(-2));
    }

    #[test]
    fn test_parse_wire_description() {
        assert_eq!(
            parse_wire_description("U123,D14,R5"),
            vec![Movement::Vertical(123), Movement::Vertical(-14), Movement::Horizontal(5)]
        );
    }

    #[test]
    fn test_vector_cross_intersection_none() {
        let horizontal = Vector::new(Point::new(1, 0), Movement::Horizontal(3));
        let vertical = Vector::new(Point::new(0, 1), Movement::Vertical(3));
        assert_eq!(horizontal.intersects(&vertical), false);
    }

    #[test]
    fn test_vector_cross_intersection_single() {
        let horizontal = Vector::new(Point::new(0, 0), Movement::Horizontal(3));
        let vertical = Vector::new(Point::new(1, -1), Movement::Vertical(3));
        assert_eq!(horizontal.intersects(&vertical), true);
    }
}
