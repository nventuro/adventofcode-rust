use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq)]
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
struct Segment {
    x: f64,
    y: f64,
}

struct Angle {
    value: f64,
}

impl Segment {
    fn from_points(source: &Point, target: &Point) -> Segment {
        Segment {
            x: (target.x - source.x) as f64,
            y: (target.y - source.y) as f64,
        }
    }

    fn angle(&self) -> Angle {
        Angle {
            value: self.y.atan2(self.x),
        }
    }

    fn len_sq(&self) -> f64 {
        self.x.powf(2_f64) + self.y.powf(2_f64)
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        (self.value - other.value).abs() < 0.0001
    }
}

impl Eq for Angle {}

impl Hash for Angle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.value * 10000_f64) as i32).hash(state);
    }
}

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Failed to read from file '{}'", filename));

    let asteroids = get_asteroids(&contents);

    let (station_asteroid, in_los) = get_max_asteroids_in_los(&asteroids);

    println!(
        "Asteroid at {:?} has {:?} asterioids in LoS",
        station_asteroid, in_los
    );

    let other_asteroids = asteroids
        .iter()
        .filter(|asteroid| *asteroid != station_asteroid)
        .map(|asteroid| Segment::from_points(station_asteroid, &asteroid))
        .collect

//    let by_angle = ::new();
//    for asteroid in other_asteroids {
//        let los = Segment::from_points(station_asteroid, &asteroid);
//    }
}

fn get_asteroids(map: &str) -> Vec<Point> {
    let rows = map
        .split_whitespace()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let mut asteroids = Vec::new();

    for (row_number, row) in rows.iter().enumerate() {
        for (line_number, value) in row.chars().enumerate() {
            if value == '#' {
                asteroids.push(Point::new(
                    line_number.try_into().unwrap(),
                    row_number.try_into().unwrap(),
                ));
            }
        }
    }

    asteroids
}

fn get_asteroids_in_los(from: &Point, asteroids: &Vec<Point>) -> usize {
    asteroids
        .iter()
        .map(|asteroid| Segment::from_points(from, asteroid).angle())
        .collect::<HashSet<_>>()
        .len()
}

fn get_max_asteroids_in_los(asteroids: &Vec<Point>) -> (&Point, usize) {
    asteroids
        .iter()
        .map(|asteroid| (asteroid, get_asteroids_in_los(asteroid, asteroids)))
        .max_by_key(|(_asteroid, in_los)| *in_los)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_a() {
        let map = ".#..#\n.....\n#####\n....#\n...##";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)).1, 8);
    }

    #[test]
    fn test_map_b() {
        let map = "......#.#.\n#..#.#....\n..#######.\n.#.#.###..\n.#..#.....\n..#....#.#\n#..#....#.\n.##.#..###\n##...#..#.\n.#....####";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)).1, 33);
    }

    #[test]
    fn test_map_c() {
        let map = "#.#...#.#.\n.###....#.\n.#....#...\n##.#.#.#.#\n....#.#.#.\n.##..###.#\n..#...##..\n..##....##\n......#...\n.####.###.";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)).1, 35);
    }

    #[test]
    fn test_map_d() {
        let map = ".#..#..###\n####.###.#\n....###.#.\n..###.##.#\n##.##.#.#.\n....###..#\n..#.#..#.#\n#..#.#.###\n.##...##.#\n.....#.#..";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)).1, 41);
    }

    #[test]
    fn test_map_e() {
        let map = ".#..##.###...#######\n##.############..##.\n.#.######.########.#\n.###.#######.####.#.\n#####.##.#.##.###.##\n..#####..#.#########\n####################\n#.####....###.#.#.##\n##.#################\n#####.##.###..####..\n..######..##.#######\n####.##.####...##..#\n.#####..#.######.###\n##...#.##########...\n#.##########.#######\n.####.#.###.###.#.##\n....##.##.###..#####\n.#.#.###########.###\n#.#.#.#####.####.###\n###.##.####.##.#..##";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)).1, 210);
    }
}
