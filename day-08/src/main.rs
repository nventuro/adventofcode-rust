use std::io::{ self, Write };
use core::convert::{ TryInto, TryFrom };
use std::fs;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    process(&contents, 25, 6);
}

#[derive(PartialEq)]
#[derive(Debug)]
enum Color {
    Black,
    White,
    Transparent,
}

impl TryFrom<u8> for Color {
    type Error = u8;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        let character = x as char;
        match character {
            '0' => Ok(Color::Black),
            '1' => Ok(Color::White),
            '2' => Ok(Color::Transparent),
            _ => Err(x),
        }
    }
}

#[derive(Debug)]
struct Layer {
    contents: Vec<Color>,
}

impl Layer {
    fn new(bytes: &[u8]) -> Layer {
        Layer{ contents: bytes.iter().map(|byte| (*byte).try_into().unwrap()).collect() }
    }

    fn from_input(input: &str, width: usize, height: usize) -> Vec<Layer> {
        input
            .as_bytes()
            .chunks_exact(width * height)
            .map(|contents| Layer::new(contents))
            .collect()
    }

    fn color_count(&self, color: Color) -> usize {
        self.contents
            .iter()
            .filter(|pixel| **pixel == color)
            .count()
    }
}

fn process(input: &str, width: usize, height: usize) {
    let layers = Layer::from_input(input, width, height);

    let fewest_zeroes_layer = layers
        .iter()
        .min_by_key(|layer| layer.color_count(Color::Black))
        .unwrap();

    // Test correctness
    assert_eq!(
        fewest_zeroes_layer.color_count(Color::White) * fewest_zeroes_layer.color_count(Color::Transparent),
        2159
    );

    let picture = (0..width * height).
        map(|pixel_num| layers.iter().find_map(|layer| {
            let color = &layer.contents[pixel_num];
            if *color != Color::Transparent {
                Some(color)
            } else {
                None
            }
        }))
        .collect::<Vec::<Option::<&Color>>>();

    for row in picture.as_slice().chunks_exact(width) {
        for pixel in row {
            let draw = match pixel.unwrap() {
                Color::Black => " ",
                Color::White => "X",
                _ => unreachable!("Pictures cannot have transparent colors"),
            };

            print!("{}", draw);
        }
        print!("\n");
        io::stdout().flush().unwrap();
    }
}
