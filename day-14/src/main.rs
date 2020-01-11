use std::collections::HashMap;
use std::fs;

extern crate regex;
use regex::Regex;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Failed to read from file '{}'", filename));

    // let system = System::new(
    //     Regex::new(r"<x=(?P<x>-?\d+), y=(?P<y>-?\d+), z=(?P<z>-?\d+)>")
    //         .unwrap()
    //         .captures_iter(&contents)
    //         .map(|capture| {
    //             [
    //                 Particle::new(capture["x"].parse().unwrap()),
    //                 Particle::new(capture["y"].parse().unwrap()),
    //                 Particle::new(capture["z"].parse().unwrap()),
    //             ]
    //         })
    //         .collect(),
    // );

    // println!("Total period: {}", system.find_period());
}

struct Chemical {
    name: String,
    amount: u32,
}

struct Reaction {
    reactants: Vec<Chemical>,
    product: Chemical,
}

impl Reaction {
    fn new(description: String) -> Reaction {
        Regex::new(r"<x=(?P<x>-?\d+), y=(?P<y>-?\d+), z=(?P<z>-?\d+)>")
            .unwrap()
            .captures_iter(&contents)
            .map(|capture| {
                [
                    Particle::new(capture["x"].parse().unwrap()),
                    Particle::new(capture["y"].parse().unwrap()),
                    Particle::new(capture["z"].parse().unwrap()),
                ]
            })
            .collect(),
    }
}

struct Factory {
    reactions: HashMap<String, Reaction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy() {
    }
}
