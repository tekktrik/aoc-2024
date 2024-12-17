use std::fs;

use clap::Parser;
use regex::Regex;

/// Representation of a system of equations for both x and y
type SystemOfEquations = ((u64, u64, u64), (u64, u64, u64));

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Representation of the effects of a button press
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct ButtonPress {
    label: char,
    x: u64,
    y: u64,
}

/// Represenation of the prize coordinates
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct PrizeLocation {
    x: u64,
    y: u64,
}

/// Representation of the machine game
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct MachineGame {
    a: ButtonPress,
    b: ButtonPress,
    prize: PrizeLocation,
}

impl MachineGame {
    /// Corrent the input so that the prize location is much larger
    fn correct_prize(&mut self) {
        self.prize.x += 10000000000000;
        self.prize.y += 10000000000000;
    }
    /// Returns the machine game as as system of equations
    fn as_system(&self) -> SystemOfEquations {
        let x_eq = (self.a.x, self.b.x, self.prize.x);
        let y_eq = (self.a.y, self.b.y, self.prize.y);
        (x_eq, y_eq)
    }

    /// Solve the system of equations for the cost to win
    fn solve_for_cost(&self) -> Option<u128> {
        if let Some((a_presses, b_presses)) = self.solve_system() {
            return Some((3 * a_presses) + b_presses);
        }
        None
    }

    /// Solves the independent system of equations
    ///
    /// Note that this is ONLY for independent systems of equations.
    fn solve_system(&self) -> Option<(u128, u128)> {
        // Get the x and y equations of the system of equations
        let (x_eq, y_eq) = self.as_system();

        // Get the components for solving the system of equations for Button B
        let dividend = (x_eq.0 * y_eq.2) as i128 - (y_eq.0 * x_eq.2) as i128;
        let divisor = -(y_eq.0 as i128) * x_eq.1 as i128 + (x_eq.0 * y_eq.1) as i128;

        // Check if the result is a positive integer number of presses
        if dividend % divisor != 0 || dividend / divisor < 0 {
            return None;
        }

        // The number of button presses for B is a positive interger, calculate it
        let b_presses = dividend / divisor;

        // Check whether the number of A presses is a positive integer
        let a_presses_dividend = x_eq.2 as i128 - (x_eq.1 as i128 * b_presses);
        let a_presses_divisor = x_eq.0 as i128;
        if a_presses_dividend % a_presses_divisor != 0 || a_presses_dividend / a_presses_divisor < 0
        {
            return None;
        }

        // Get the number of A button presses
        let a_presses = a_presses_dividend / a_presses_divisor;

        // Return the number of A and B button presses
        Some((a_presses as u128, b_presses as u128))
    }
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
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the game from the input text
    let games = parse_input(&contents);

    // Get the minimum total cost to win the maximum number of games
    let mut total_cost = 0;
    for game in &games {
        if let Some(cost) = game.solve_for_cost() {
            total_cost += cost;
        }
    }

    // Print the total cost
    println!("{total_cost}")
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the game from the input text
    let mut games = parse_input(&contents);
    games.iter_mut().for_each(|x| x.correct_prize());

    // Get the minimum total cost to win the maximum number of games
    let mut total_cost = 0;
    for game in &games {
        if let Some(cost) = game.solve_for_cost() {
            total_cost += cost;
        }
    }

    // Print the total cost
    println!("{total_cost}")
}

/// Parse the input string into a list of machine games
fn parse_input(text: &str) -> Vec<MachineGame> {
    // Create a list for storing the machine games
    let mut all_games = Vec::new();

    // Create the regex patterns for finding infomration about button presses
    // and prize locations
    let a_re = Regex::new(r"Button A: X\+(\d+), Y\+(\d+)").unwrap();
    let b_re = Regex::new(r"Button B: X\+(\d+), Y\+(\d+)").unwrap();
    let prize_re = Regex::new(r"Prize: X=(\d+), Y=(\d+)").unwrap();

    // Iterate through the individual game texts
    for game_text in text.split("\n\n") {
        // Parse the information regarding A button presses
        let a_capture = a_re.captures(game_text).unwrap();
        let a_x = a_capture.get(1).unwrap().as_str().parse::<u64>().unwrap();
        let a_y = a_capture.get(2).unwrap().as_str().parse::<u64>().unwrap();
        let a = ButtonPress {
            label: 'a',
            x: a_x,
            y: a_y,
        };

        // Parse the information regarding B button presses
        let b_capture = b_re.captures(game_text).unwrap();
        let b_x = b_capture.get(1).unwrap().as_str().parse::<u64>().unwrap();
        let b_y = b_capture.get(2).unwrap().as_str().parse::<u64>().unwrap();
        let b = ButtonPress {
            label: 'b',
            x: b_x,
            y: b_y,
        };

        // Parse the information regarding the prize location
        let prize_capture = prize_re.captures(game_text).unwrap();
        let prize_x = prize_capture
            .get(1)
            .unwrap()
            .as_str()
            .parse::<u64>()
            .unwrap();
        let prize_y = prize_capture
            .get(2)
            .unwrap()
            .as_str()
            .parse::<u64>()
            .unwrap();
        let prize = PrizeLocation {
            x: prize_x,
            y: prize_y,
        };

        // Create the matchine game
        let game = MachineGame { a, b, prize };

        // Add the game to the list of games
        all_games.push(game);
    }

    // Return the list of all parsed games
    all_games
}
