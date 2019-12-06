fn main() {
    process("172930", "683082");
}

fn process(start: &str, end: &str) {
    let total = password_range(start, end)
        .map(|v| v.to_string())
        .filter(|p| is_valid(p))
        .count();

    println!("Valid passwords: {}", total);
}

fn password_range(start: &str, end: &str) -> std::ops::Range<u32> {
    start.parse::<u32>().unwrap()..end.parse::<u32>().unwrap()
}

#[derive(PartialEq)]
enum Repeat {
    None,
    Double,
    Multi,
}

fn is_valid(password: &str) -> bool {
    let mut previous: Option<u32> = None;
    let mut current_repeat = Repeat::None;
    let mut found_double_repeat = false;

    for digit in password.chars().map(|c| c.to_digit(10).unwrap()) {
        if previous.is_some() {
            if previous.unwrap() > digit {
                // Decreasing sequence - invalid
                return false;
            }

            match current_repeat {
                Repeat::None => {
                    if previous.unwrap() == digit {
                        current_repeat = Repeat::Double;
                    }
                },
                Repeat::Double => {
                    if previous.unwrap() == digit {
                        current_repeat = Repeat::Multi;
                    } else {
                        current_repeat = Repeat::None;
                        found_double_repeat = true;
                    }
                },
                Repeat::Multi => {
                    if previous.unwrap() != digit {
                        current_repeat = Repeat::None;
                    }
                }
            }
        }


        previous = Some(digit)
    }

    found_double_repeat || current_repeat == Repeat::Double
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        assert_eq!(is_valid("112233"), true);
    }

    #[test]
    fn test_decreasing() {
        assert_eq!(is_valid("123450"), false);
    }

    #[test]
    fn test_no_double() {
        assert_eq!(is_valid("123789"), false);
    }

    #[test]
    fn test_repeated_long() {
        assert_eq!(is_valid("123444"), false);
    }

    #[test]
    fn test_double_repeat() {
        assert_eq!(is_valid("111122"), true);
    }

    #[test]
    fn test_quadruple_repeat() {
        assert_eq!(is_valid("111123"), false);
    }
}
