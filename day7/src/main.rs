use std::{
    collections::{HashSet, VecDeque},
    fs,
};

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Possible operations that can be performed
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Operation {
    Addition,
    Multiplication,
    Concatenation,
}

/// Possible equations representation, including results and inputs
#[derive(Clone, Debug)]
struct PossibleEquation {
    result: i64,
    inputs: VecDeque<i64>,
    // operations: Vec<Operation>,
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

/// Runs the main functions with the specified operations
fn run_main_with_operations(filepath: String, operations: &HashSet<Operation>) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Get the list of possible equations
    let equations = parse_data(&contents);

    // Get the sum of the valid equations
    let solvable_total: i64 = equations
        .iter()
        .filter(|x| x.is_solvable(operations))
        .map(|x| x.result)
        .sum();
    println!("{solvable_total}");
}

/// Runs part one
fn main_part_one(filepath: String) {
    let operations_list = [Operation::Multiplication, Operation::Addition];
    let operations = HashSet::from_iter(operations_list.iter().copied());
    run_main_with_operations(filepath, &operations);
}

// Runs part two
fn main_part_two(filepath: String) {
    let operations_list = [
        Operation::Multiplication,
        Operation::Addition,
        Operation::Concatenation,
    ];
    let operations = HashSet::from_iter(operations_list.iter().copied());
    run_main_with_operations(filepath, &operations);
}

impl PossibleEquation {
    /// Checks whether the equation is solvable, recursively if needed
    fn is_solvable(&self, operations_allowed: &HashSet<Operation>) -> bool {
        // Get the first two inputs at the top of the inputs list
        let mut inputs = self.inputs.clone();
        let x = inputs.pop_front().expect("Missing first number");
        let y = inputs.pop_front().expect("Missing second nunber");

        // If there are no remaining inputs other than these two, operate on them directly
        if inputs.is_empty() {
            // If they are valid via multiplication or addition, return true
            let mult = x * y == self.result;
            let add = x + y == self.result;
            if mult || add {
                return true;
            }

            // If concatenation is not allowed, return false
            if !operations_allowed.contains(&Operation::Concatenation) {
                return false;
            }

            // Check the actual result of trying to concatenate the numbers
            return combine_numbers(x, y) == self.result;
        }

        // Recursively check possible values for multiplication, assuming the answer is still even possible
        let z_mult = x * y;
        if z_mult <= self.result
            && self
                .as_if_next_operation(Operation::Multiplication)
                .is_solvable(operations_allowed)
        {
            return true;
        }

        // Recursively check possible values for addition, assuming the answer is still even possible
        let z_add = x + y;
        if z_add <= self.result
            && self
                .as_if_next_operation(Operation::Addition)
                .is_solvable(operations_allowed)
        {
            return true;
        }

        // If concatenation is not allowed, return false
        if !operations_allowed.contains(&Operation::Concatenation) {
            return false;
        }

        // Recursively check the actual results of trying to concatenate the numbers
        self.as_if_next_operation(Operation::Concatenation)
            .is_solvable(operations_allowed)
    }

    /// Creates a new equation from an existing one, where the given operation is performed
    /// to the leading inputs
    fn as_if_next_operation(&self, operation: Operation) -> Self {
        // Get the first two inputs
        let mut inputs = self.inputs.clone();
        let x = inputs.pop_front().expect("Could not get first number");
        let y = inputs.pop_front().expect("Could not get second number");

        // Get the result of the operation on the two numbers
        let z = match operation {
            Operation::Multiplication => x * y,
            Operation::Addition => x + y,
            Operation::Concatenation => combine_numbers(x, y),
        };

        // Create and return a new equation with the new inputs
        inputs.push_front(z);
        Self {
            result: self.result,
            inputs,
        }
    }
}

/// Gets the "reverse factor" for a given number
///
/// I originally implemented this using logarithm base 10 and the ceiling
/// operation, but I was a little concerned about using floats given that
/// Rust says it's non-deterministic, and this method is also much simpler
/// to program
fn reverse_factor_for(x: i64) -> i64 {
    let mut factor = 10;
    while x % factor != x {
        factor *= 10;
    }
    factor
}

/// Combines two numbers (concatenation)
fn combine_numbers(x: i64, y: i64) -> i64 {
    let mult = reverse_factor_for(y);
    x * mult + y
}

/// Parse a string input into a list of possible equations
fn parse_data(input: &str) -> Vec<PossibleEquation> {
    // Create a list of possible equations to populate;
    let mut equations = Vec::new();

    // Iterate through the input string line by line
    for line in input.lines().filter(|x| !x.is_empty()) {
        // Split the line by the colon to get and parse the result on the left
        let result_split: Vec<&str> = line.split(':').collect();
        let result = result_split[0]
            .parse::<i64>()
            .expect("Could not parse result");

        // Split the right by spaces and parse to get the inputs
        let inputs = result_split[1]
            .trim()
            .split(' ')
            .map(|x| x.parse::<i64>().expect("Could not parse input"))
            .collect();

        // Create the possible equation and add it to the list of equations
        let equation = PossibleEquation { result, inputs };
        equations.push(equation);
    }

    // Return the list of possible equations
    equations
}
