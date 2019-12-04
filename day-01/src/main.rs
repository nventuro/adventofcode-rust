use std::fs;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    process(contents);
}

fn process(input: String) {
    let mut total_fuel = 0;

    for line in input.split_whitespace() {
        if line.is_empty() { continue }

        let mass = line.parse::<i32>().unwrap();
        total_fuel += get_fuel(mass);
    }

    println!("Total fuel required: {}", total_fuel);
}

fn get_fuel(mass: i32) -> i32 {
    let fuel = mass / 3 - 2;
    if fuel > 0 {
        fuel + get_fuel(fuel)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fuel_simple() {
        assert_eq!(get_fuel(14), 2);
    }


    #[test]
    fn test_get_fuel_complex() {
        assert_eq!(get_fuel(100756), 50346);
    }

    #[test]
    fn test_get_fuel_chain() {
        assert_eq!(get_fuel(5), 0);
        assert_eq!(get_fuel(21), 5);
        assert_eq!(get_fuel(70), 21 + 5);
        assert_eq!(get_fuel(216), 70 + 21 + 5);
        assert_eq!(get_fuel(654), 216 + 70 + 21 + 5);
        assert_eq!(get_fuel(654), 216 + 70 + 21 + 5);
        assert_eq!(get_fuel(1969), 654 + 216 + 70 + 21 + 5);
    }
}

