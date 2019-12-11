use std::collections::HashSet;
use std::convert::TryInto;
use std::fs;
use std::hash::{Hash, Hasher};

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
struct LineOfSight {
    x: i32,
    y: i32,
}

impl LineOfSight {
    fn from_points(source: &Point, target: &Point) -> LineOfSight {
        LineOfSight {
            x: target.x - source.x,
            y: target.y - source.y,
        }
    }

    fn angle(&self) -> f64 {
        (self.y as f64).atan2(self.x as f64)
    }
}

impl PartialEq for LineOfSight {
    fn eq(&self, other: &Self) -> bool {
        (self.angle() - other.angle()).abs() < 0.0001
    }
}

impl Eq for LineOfSight {}

impl Hash for LineOfSight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.angle() * 10000_f64) as i32).hash(state);
    }
}

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Failed to read from file '{}'", filename));

    let asteroids = get_asteroids(&contents);

    println!("{:?}", get_max_asteroids_in_los(&asteroids));
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
        .map(|asteroid| LineOfSight::from_points(from, asteroid))
        .collect::<HashSet<_>>()
        .len()
}

fn get_max_asteroids_in_los(asteroids: &Vec<Point>) -> usize {
    asteroids
        .iter()
        .map(|asteroid| get_asteroids_in_los(asteroid, &asteroids))
        .max()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_a() {
        let map = ".#..#\n.....\n#####\n....#\n...##";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)), 8);
    }

    #[test]
    fn test_map_b() {
        let map = "......#.#.\n#..#.#....\n..#######.\n.#.#.###..\n.#..#.....\n..#....#.#\n#..#....#.\n.##.#..###\n##...#..#.\n.#....####";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)), 33);
    }

    #[test]
    fn test_map_c() {
        let map = "#.#...#.#.\n.###....#.\n.#....#...\n##.#.#.#.#\n....#.#.#.\n.##..###.#\n..#...##..\n..##....##\n......#...\n.####.###.";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)), 35);
    }

    #[test]
    fn test_map_d() {
        let map = ".#..#..###\n####.###.#\n....###.#.\n..###.##.#\n##.##.#.#.\n....###..#\n..#.#..#.#\n#..#.#.###\n.##...##.#\n.....#.#..";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)), 41);
    }

    #[test]
    fn test_map_e() {
        let map = ".#..##.###...#######\n##.############..##.\n.#.######.########.#\n.###.#######.####.#.\n#####.##.#.##.###.##\n..#####..#.#########\n####################\n#.####....###.#.#.##\n##.#################\n#####.##.###..####..\n..######..##.#######\n####.##.####...##..#\n.#####..#.######.###\n##...#.##########...\n#.##########.#######\n.####.#.###.###.#.##\n....##.##.###..#####\n.#.#.###########.###\n#.#.#.#####.####.###\n###.##.####.##.#..##";
        assert_eq!(get_max_asteroids_in_los(&get_asteroids(map)), 210);
    }
}
