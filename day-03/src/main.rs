use std::fs;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    process(contents);
}

fn process(input: String) {
    // Parse input and get movements for each wire

    let mut wires_movements = Vec::<Vec::<Movement>>::new();
    for line in input.split_whitespace() {
        if !line.is_empty() {
            // New wire
            wires_movements.push(parse_wire_description(line));
        }
    }

    // Compute positions for each wire

    let mut wires_positions = Vec::<Vec::<Point>>::new();
    for wire_movements in wires_movements {
        // All wires start at 0,0
        let mut wire_positions = vec![Point::new(0, 0)];

        // Compute all positions for wire
        for movement in wire_movements {
            let start = wire_positions.last().unwrap().clone();

            match movement {
                Movement::Horizontal(value) => {
                    for i in 1..value {
                        wire_positions.push(Point::new(start.x + i, start.y));
                    }
                },
                Movement::Vertical(value) => {
                    for i in 1..value {
                        wire_positions.push(Point::new(start.x, start.y + i));
                    }
                },
            }
        }

        wires_positions.push(wire_positions);
    }

    // Find intersections - go over the first wire, and check in all other wires if they also have that position
    assert!(wires_positions.len() >= 2);
    println!("{}",  wires_positions[0].len());
    println!("{}",  wires_positions[1].len());

    for first_wire_position in wires_positions.first().unwrap() {
        for other_wire_position in wires_positions[1].clone() {
            if first_wire_position == other_wire_position {
                println!("Match in {:?}", first_wire_position);
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
#[derive(PartialEq)]
#[derive(Clone)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
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
