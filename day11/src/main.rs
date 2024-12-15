use std::{collections::HashMap, fs};

use clap::Parser;

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u8,
    filepath: String,
}

/// Representation of data for blinking that should be pre-saved
#[derive(Debug)]
struct PreSaveBlinking {
    counts: HashMap<usize, u128>,
}

/// Representation of a magic stone
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Stone {
    value: u64,
}

impl Stone {
    /// Creates a new stone with a given value
    fn new(value: u64) -> Self {
        Self { value }
    }

    /// Creates a stone of value one
    fn one() -> Self {
        Self { value: 1 }
    }

    /// Gets the number of digits for this stone
    fn get_digit_count(&self) -> u64 {
        // Initialize the base factor and number of digits
        let mut factor = 10;
        let mut num_digits = 1;

        // While the number can still be divided by 10, increase the tally of number
        // of digits, as power of ten used to check
        while self.value % factor != self.value {
            factor *= 10;
            num_digits += 1;
        }

        // Return the number of digits
        num_digits
    }

    /// Checks whether the stone has an even number of digits
    fn has_even_digits(&self) -> bool {
        self.get_digit_count() % 2 == 0
    }

    /// Split the stone into two different stones by seperating the digits in half
    fn split(&self) -> (Self, Self) {
        // Check whether the stone is eligible to be split
        let value = self.value;
        let num_digits = self.get_digit_count();
        if num_digits % 2 != 0 {
            panic!("Cannot split stone of value {value}")
        }

        // Get the divisor needed for splitting
        let splitter_power = self.get_digit_count() / 2;
        let splitter = 10_u64.pow(splitter_power as u32);

        // Split the stone and return the two resulting stones
        let left_value = self.value / splitter;
        let right_value = self.value % splitter;
        (Stone::new(left_value), Stone::new(right_value))
    }

    /// Checks whether the value of the stone is zero
    fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Grow the stone by multiplying by 2024
    fn grow(&self) -> Self {
        if self.has_even_digits() || self.value == 0 {
            panic!("Only stones not fitting the other rules should grow")
        }
        Stone::new(self.value * 2024)
    }

    /// Unobserves the stone, resulting in a change according to the game rules
    fn unobserve(&self) -> StoneChange {
        if self.is_zero() {
            StoneChange::One(Stone::one())
        } else if self.has_even_digits() {
            let (left, right) = self.split();
            return StoneChange::Split(left, right);
        } else {
            self.grow();
            return StoneChange::Grow(self.grow());
        }
    }
}

/// The various actions a stone can take upon blinking
enum StoneChange {
    /// Turn the stone to a stone of value one
    One(Stone),
    /// Split the stone into two
    Split(Stone, Stone),
    /// Multiply the stone by 2024
    Grow(Stone),
}

/// Main entry function
fn main() {
    // Parse CLI arguments
    let cli = CliArgs::parse();

    // Run the code for the desired challenge part
    match cli.part {
        1 => main_part_one(cli.filepath),
        2 => main_part_two(cli.filepath),
        _ => panic!("Invalid selection part selection!"),
    }
}

/// Runs part one
fn main_part_one(filepath: String) {
    // Get the file contents
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Create the list of stones
    let mut stones = parse_input(&contents);

    // Simulate the blinking process 25 times
    stones = simulate_blinking_saving(&stones, 10);

    // Print the number of stones
    let num_stones = stones.len();
    println!("{num_stones}");
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the file contents
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Create the list of stones
    let stones = parse_input(&contents);

    // Get the total number of stones by "lazy solving"
    let total = lazy_solver(&stones, 38, 75);

    // Print the number of stones
    println!("{total}");
}

/// Simulate blinking n times, returning the resulting state of the stones
fn simulate_blinking_saving(stones: &[Stone], n: u8) -> Vec<Stone> {
    let mut stones = Vec::from(stones);
    for _i in 0..n {
        stones = blink_save(&stones);
    }
    stones
}

/// Create the list of pre-saved blinks for 0-9 for up to n iterations to be used
/// for lazy solving
fn preload_blinks(n: usize) -> HashMap<Stone, PreSaveBlinking> {
    let mut presaves = HashMap::new();

    for num in 0..10 {
        let mut counts = HashMap::new();
        let stone = Stone { value: num };
        for i in 0..n {
            let iter_count = i + 1;
            let count = blink_count(&stone, 0, iter_count);
            counts.insert(iter_count, count);
        }
        let presave = PreSaveBlinking { counts };
        presaves.insert(stone, presave);
    }

    presaves
}

/// "Lazily" solve for the given stone.  This is done by checking whether the stone is in
/// the pre-save hash map for the given number of iterations.  If it is, then that value is
/// returned for the current state; otherwise, the next generation of stones is generated,
/// and solved lazily, and that answer is returned.
fn lazy_solve(
    stone: &Stone,
    remaining_i: usize,
    presaves: &HashMap<Stone, PreSaveBlinking>,
) -> u128 {
    // No more stones to be generated, so this simply returns a single stone (this one)
    if remaining_i == 0 {
        return 1;
    }

    // If the pre-save map has the current stone for the remaining iterations, use it
    if let Some(presave) = presaves.get(stone) {
        if let Some(count) = presave.counts.get(&remaining_i) {
            return *count;
        }
    }

    // The stone doesn't exist in the pre-save map for the number of generations needed,
    // so the next generation is generated and lazily solved
    let mut total = 0;
    for next_stone in blink_save(&[*stone]) {
        total += lazy_solve(&next_stone, remaining_i - 1, presaves)
    }
    total
}

/// Solves the problem for a given set of stones for n iterations by
/// pre-saving s number of generations for numbers 0-9, which cyclically
/// result in other single digit stones.  The state for n-s iterations is
/// then created and the remaining iterations are "lazily" solved
fn lazy_solver(stones: &[Stone], s: usize, n: usize) -> u128 {
    // Pre-save the given number of blinks
    println!("Preparing presaves...");
    let presaves = preload_blinks(s);

    // Create a running total of stones
    let mut total = 0;

    // Calculate the start state from the save and target iterations
    let start_state = n - s;

    // Iterate through the given stones individually
    println!("Iterating through stones...");
    for stone in stones {
        // Create the start state for the stone
        println!("Creating start state and solving for stone {stone:?}");
        let state = simulate_blinking_saving(&[*stone], start_state as u8);

        // Lazily solve for each stone in the pre-generated state
        for state_stone in &state {
            total += lazy_solve(state_stone, s, &presaves);
        }
    }

    // Return the number of stones generated
    total
}

/// Parse the input text into a list of stones
fn parse_input(input: &str) -> Vec<Stone> {
    let mut stones = Vec::new();
    for text in input.trim().split(" ") {
        let value = text.parse::<u64>().expect("Could not parse number");
        stones.push(Stone::new(value));
    }
    stones
}

/// Perform a blink action for the given stones, and return the next generation of stones
fn blink_save(stones: &[Stone]) -> Vec<Stone> {
    // Create a list for storing new stones
    let mut new_stones = Vec::new();

    // Iterate through the given stones
    for stone in stones {
        // Perform the blink and push the resulting stones to the list
        match stone.unobserve() {
            StoneChange::Split(left, right) => {
                new_stones.push(left);
                new_stones.push(right);
            }
            StoneChange::One(new) => new_stones.push(new),
            StoneChange::Grow(new) => new_stones.push(new),
        }
    }

    // Return the created list of stones
    new_stones
}

/// Perform a blink action for a given stone, and return the number of stones in the
/// next n generations (and starting at index = i)
fn blink_count(stone: &Stone, i: usize, n: usize) -> u128 {
    // If iteration is complete, the path yields a single stone
    if i == n {
        return 1;
    }

    // Get the new index of the iteration
    let new_i = i + 1;

    // Create a total for adding recursive results
    let mut total = 0;

    // Perform the blink and add the resulting total to the running count
    match stone.unobserve() {
        StoneChange::Split(left, right) => {
            total += blink_count(&left, new_i, n);
            total += blink_count(&right, new_i, n);
        }
        StoneChange::One(new) => total += blink_count(&new, new_i, n),
        StoneChange::Grow(new) => total += blink_count(&new, new_i, n),
    }

    // Return the total number of blinks
    total
}
