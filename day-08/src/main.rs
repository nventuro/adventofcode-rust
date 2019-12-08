use std::fs;

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename).unwrap_or_else(
        |_| panic!("Failed to read from file '{}'", filename)
    );

    let result = process(&contents, 25, 6);
    println!("Result: {}", result);
}

#[derive(Debug)]
struct Layer<'a> {
    contents: &'a[u8],
}

impl<'a> Layer<'a> {
    fn from_input(input: &'a str, width: usize, height: usize) -> Vec<Layer<'a>> {
        input
            .as_bytes()
            .chunks_exact(width * height)
            .map(|contents| Layer { contents })
            .collect()
    }

    fn char_count(&self, character: char) -> usize {
        self.contents
            .iter()
            .filter(|value| **value as char == character)
            .count()
    }
}

fn process(input: &str, width: usize, height: usize) -> usize {
    let layers = Layer::from_input(input, width, height);

    let fewest_zeroes_layer = layers
        .iter()
        .min_by_key(|layer| layer.char_count('0'))
        .unwrap();

    fewest_zeroes_layer.char_count('1') * fewest_zeroes_layer.char_count('2')
}
