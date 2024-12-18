use std::fmt;
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    fs,
    hash::{Hash, Hasher},
};

use clap::Parser;
use regex::Regex;

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Representation of a robot
#[derive(Debug, Clone, Copy)]
struct Robot {
    id: usize,
    x_pos: u64,
    y_pos: u64,
    x_vel: i64,
    y_vel: i64,
}

impl Robot {
    fn position(&self) -> (u64, u64) {
        (self.x_pos, self.y_pos)
    }
}

impl PartialEq for Robot {
    fn eq(&self, other: &Robot) -> bool {
        self.id == other.id
    }
}

impl Eq for Robot {}

impl Hash for Robot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Representation of the game map
#[derive(Debug, Clone)]
struct GameMap {
    robots: Vec<Robot>,
    width: u64,
    height: u64,
}

impl GameMap {
    /// Parses the map from the provided string
    fn parse(text: &str, width: u64, height: u64) -> Self {
        // Create a list for storing robots
        let mut robots = Vec::new();

        // Create the regex pattern for parsing robot information
        let re = Regex::new(r"p=(\d+),(\d+) v=(-?\d+),(-?\d+)").unwrap();

        // Iterate through the string line by line
        for (id, line) in text.trim().lines().enumerate() {
            // Parse the line of text for the robot informations
            let Some((_text, [x_pos, y_pos, x_vel, y_vel])) =
                re.captures(line).map(|x| x.extract())
            else {
                panic!("Could not parse text for robot information")
            };

            // Create the robot
            let robot = Robot {
                id,
                x_pos: x_pos.parse::<u64>().unwrap(),
                y_pos: y_pos.parse::<u64>().unwrap(),
                x_vel: x_vel.parse::<i64>().unwrap(),
                y_vel: y_vel.parse::<i64>().unwrap(),
            };

            // Add the robot to the list
            robots.push(robot);
        }

        // Return a new map with the given rows
        Self {
            robots,
            width,
            height,
        }
    }

    // Extrapolates the location of all the robots after n seconds
    fn extrapolate(&mut self, n: u64) {
        // Iterate through the robots
        for robot in &mut self.robots {
            // Get the extended X and Y travel position
            let extrapolated_x = robot.x_vel * n as i64 + robot.x_pos as i64;
            let extrapolated_y = robot.y_vel * n as i64 + robot.y_pos as i64;

            // Correct for the wrap-around teleporation
            let mut map_x = extrapolated_x % self.width as i64;
            let mut map_y = extrapolated_y % self.height as i64;

            // If the map X or Y is negative, put it back on the map by adding
            // the width or height respectively
            if map_x < 0 {
                map_x += self.width as i64;
            }
            if map_y < 0 {
                map_y += self.height as i64;
            }

            // Update the robots position
            robot.x_pos = map_x as u64;
            robot.y_pos = map_y as u64;
        }
    }

    /// Gets the neightbors for given robot, which is any robot within a single square
    fn neighbors(&self, robot: &Robot) -> HashSet<&Robot> {
        self.robots
            .iter()
            .filter(|x| robot.x_pos.abs_diff(x.x_pos) <= 1 && robot.y_pos.abs_diff(x.y_pos) <= 1)
            .collect()
    }

    /// Gathers a grouping of robots for the given robot, searching recursively if necessary
    fn gather_grouping(&self, robot: &Robot, grouping: &mut HashSet<Robot>) -> HashSet<Robot> {
        // Create a list for storing the robot grouping
        let mut discovered_robots = HashSet::new();

        // If the robot is not already grouping, it should be added
        if !grouping.contains(robot) {
            // Add the robot to the grouping and list of discovered robots
            grouping.insert(*robot);
            discovered_robots.insert(*robot);

            // Get the neighbors of the current robot
            let neighbors = self.neighbors(robot);

            // Recursively checking if the new robot is part of the grouping
            for new_neighbor in neighbors {
                let other_robots = self.gather_grouping(new_neighbor, grouping);
                discovered_robots.extend(other_robots);
            }
        }

        // Return the list of discovered robots in the grouping
        discovered_robots
    }

    /// Gets the groupings for the current state of the map
    fn get_groupings(&self) -> Vec<HashSet<Robot>> {
        // Create a list for groupings of robots
        let mut groupings = Vec::new();

        // Create a hash set for keeping track of checked robots
        let mut checked_robots = HashSet::new();

        // Iterate through the robots one by one
        for robot in &self.robots {
            // If the robot has already been checked, skip it
            if checked_robots.contains(robot) {
                continue;
            }

            // Create a hash set for storing groupings
            let mut grouping = HashSet::new();

            // Get the grouping for the given robot
            let explored = self.gather_grouping(robot, &mut grouping);

            // Add the robots from the grouping to the list of checked robots
            checked_robots.extend(explored);

            // Add the grouped robots to the list
            groupings.push(grouping);
        }

        // Return the completed list of grouped robots
        groupings
    }

    /// Calculates the safety factor for the current state of the map
    fn safety_factor(&self) -> usize {
        // Get the halfway marks
        let half_width = self.width / 2;
        let half_height = self.height / 2;

        // println!("Half width = {half_width}. half height = {half_height}");

        // Create lists for storing robots
        let mut topleft = Vec::new();
        let mut topright = Vec::new();
        let mut bottomright = Vec::new();
        let mut bottomleft = Vec::new();

        // Place robots in their respective quadrant groups
        for robot in &self.robots {
            // Get shorthand for X and Y coordinates
            let x = robot.x_pos;
            let y = robot.y_pos;

            // Place the robot in the appropriate quadrant group
            if x < half_width && y < half_height {
                topleft.push(robot);
            } else if x > half_width && y < half_height {
                topright.push(robot);
            } else if x > half_width && y > half_height {
                bottomright.push(robot);
            } else if x < half_width && y > half_height {
                bottomleft.push(robot);
            }
        }

        // Return the product of the group quantities
        topleft.len() * topright.len() * bottomright.len() * bottomleft.len()
    }

    /// Gets the state of the map as a unique vector
    fn as_state(&self) -> Vec<(u64, u64)> {
        let mut states = Vec::new();
        for robot in &self.robots {
            states.push((robot.x_pos, robot.y_pos));
        }
        states
    }
}

impl Display for GameMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Create a string for pushing map unformation
        let mut map_string = String::new();

        // Iterate through tha possible map coordinates
        for row_index in 0..self.height {
            for col_index in 0..self.width {
                let mut space_string = String::new();
                // space_string.push('[');

                let located_robots: Vec<&Robot> = self
                    .robots
                    .iter()
                    .filter(|x| x.position() == (col_index, row_index))
                    .collect();
                let num_robots = located_robots.len();

                if num_robots != 0 {
                    space_string.push_str(&num_robots.to_string());
                } else {
                    space_string.push('.');
                }
                // space_string.push(']');

                map_string.push_str(&space_string);
            }

            // Add a newline character to the end of the row
            map_string.push('\n');
        }

        write!(f, "{}", map_string)
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

    // Parse the inout file contents into the game map
    let mut gamemap = GameMap::parse(&contents, 101, 103);

    // Simulate 100 seconds
    gamemap.extrapolate(100);

    // Calculate and print the safety factor
    let safety_factor = gamemap.safety_factor();
    println!("{safety_factor}");
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the inout file contents into the game map
    let mut gamemap = GameMap::parse(&contents, 101, 103);

    // Get the original map and it's state for comparison
    let mut original_game = gamemap.clone();
    let orginal_state = gamemap.as_state();

    // Simulate until the Christmas tree is shown
    let mut secs_elapsed = 0;

    // Keep track of the amount of order in the map
    let mut entropy: Option<(u64, usize)> = None;

    // Print information about the search
    println!("Searching through game states until loop detected...");
    println!("The game with the lowest entropy will be displayed");

    // Simulate the robot's actions
    loop {
        // Simulate the next round of robots
        secs_elapsed += 1;
        gamemap.extrapolate(1);

        // Get the state of the current iteration
        let new_state = gamemap.as_state();

        // If the robots have looped into the same state again, stop searching
        if new_state == orginal_state {
            break;
        }

        // Get the number of groupings in this iteration
        let num_groupings = gamemap.get_groupings().len();

        // If the entropy is not set and the entropy is lower, save the information
        if entropy.is_none() || num_groupings < entropy.unwrap().1 {
            entropy = Some((secs_elapsed, num_groupings))
        }
    }

    // Get the time elapsed for the moment saved
    let (elapsed, ..) = entropy.expect("Entropy detection failed");

    // Create the state of the map with the lowest entropy
    original_game.extrapolate(elapsed);

    // Print the map and the number of seconds elapsed
    println!("{original_game}");
    println!("{elapsed}");
}
