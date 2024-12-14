use std::fs;

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    part: u8,
    filepath: String,
}

/// Representation of a magic stone
#[derive(Clone, Copy, Debug)]
struct Stone {
    value: u64,
}

impl Stone {
    fn new(value: u64) -> Self {
        Self { value }
    }

    fn one() -> Self {
        Self { value: 1 }
    }

    fn get_digit_count(&self) -> u64 {
        let mut factor = 10;
        let mut num_digits = 1;
        // let value = self.value;
        // println!("Value = {value}");
        // println!("Factor = {factor}");
        while self.value % factor != self.value {
            factor *= 10;
            num_digits += 1;
        }
        num_digits
    }

    fn has_even_digits(&self) -> bool {
        self.get_digit_count() % 2 == 0
    }

    // fn get_power_factor(&self) -> u64 {
    //     10_u64.pow(self.get_digit_count() as u32)
    // }

    fn split(&self) -> (Self, Self) {
        let value = self.value;

        let num_digits = self.get_digit_count();
        if num_digits % 2 != 0 {
            panic!("Cannot split stone of value {value}")
        }

        let splitter_power = self.get_digit_count() / 2;
        let splitter = 10_u64.pow(splitter_power as u32);

        let left_value = self.value / splitter;
        let right_value = self.value % splitter;

        (Stone::new(left_value), Stone::new(right_value))
    }

    fn is_zero(&self) -> bool {
        self.value == 0
    }

    fn grow(&self) -> Self {
        if self.has_even_digits() || self.value == 0 {
            panic!("Only stones not fitting the other rules should grow")
        }

        Stone::new(self.value * 2024)
    }

    fn unobserve(&self) -> StoneChange {
        if self.is_zero() {
            // self.to_one();
            return StoneChange::One(Stone::one())
        }
        else if self.has_even_digits() {
            let (left, right) = self.split();
            return StoneChange::Split(left, right);
        }
        else {
            self.grow();
            return StoneChange::Grow(self.grow());
        }
    }
}

enum StoneChange {
    One (Stone),
    Split (Stone, Stone),
    Grow (Stone),
}

/// Main entry function
fn main() {
    // Parse CLI arguments
    let cli = CliArgs::parse();

    // Run the code for the desired challenge part
    match cli.part {
        1 => main_part_one(cli.filepath),
        // 2 => main_part_two(cli.filepath),
        _ => panic!("Invalid selection part selection!"),
    }
}

/// Runs part one
fn main_part_one(filepath: String) {
    // Get the file contents
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Create the list of stones
    let mut stones = parse_input(&contents);

    stones = simulate_blinking(&stones, 25);

    // Print the number of stones
    let num_stones = stones.len();
    println!("{num_stones}");
}

fn simulate_blinking(stones: &[Stone], n: u8) -> Vec<Stone> {
    // Create list for storing new stones
    let mut stones = Vec::from(stones); // Vec::from_iter(stones).iter().copied().copied().collect();
    // Simulate blinking 25 times
    for _i in 0..n {
        stones = blink(&stones);
        // println!("{stones:?}");
    }

    stones
}

fn parse_input(input: &str) -> Vec<Stone> {
    let mut stones = Vec::new();
    for text in input.trim().split(" ") {
        let value = text.parse::<u64>().expect("Could not parse number");
        stones.push(Stone::new(value));
    }
    stones
}

fn blink(stones: &Vec<Stone>) -> Vec<Stone> {
    let mut new_stones = Vec::new();
    for stone in stones {
        match stone.unobserve() {
            StoneChange::Split(left, right) => {
                new_stones.push(left);
                new_stones.push(right);
            }
           StoneChange::One(new) => new_stones.push(new),
           StoneChange::Grow(new) => new_stones.push(new),
        }
    }
    new_stones
}
