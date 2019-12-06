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
struct Vector {
    start: Point,
    length: i32,
}

#[derive(Debug)]
enum Segment {
    Horizontal(Vector),
    Vertical(Vector),
}

impl Segment {
    fn new(start: Point, movement: Movement) -> Segment {
        match movement {
            Movement::Horizontal(length) => Segment::Horizontal(Vector { start, length }),
            Movement::Vertical(length) => Segment::Vertical(Vector { start, length })
        }
    }

    fn end(self: &Segment) -> Point {
        match self {
            Segment::Horizontal(vector) =>
                Point::new(vector.start.x + vector.length, vector.start.y),

            Segment::Vertical(vector) =>
                Point::new(vector.start.x, vector.start.y + vector.length),
        }
    }

    fn intersects(self: &Segment, other: &Segment) -> bool {
        use Segment::*;
        match (self, other) {
            (Horizontal(a), Horizontal(b)) => {
                true
            },

            (Horizontal(a), Vertical(b)) => {
                a.start.y > std::cmp::min(b.start.y, b.end().y)
            },

            (Vertical(a), Horizontal(b)) => {
                true
            },

            (Vertical(a), Vertical(b)) => {
                true
            },
        }
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
}
