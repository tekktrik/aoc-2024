use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::{Hash, Hasher},
};

use clap::Parser;
use itertools::Itertools;

#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Representation of a given coordinate on a map, and whether an
/// antenna of a given frequency is at that location
#[derive(Clone, Copy, Eq)]
struct Coordinate {
    x: i64,
    y: i64,
    antenna: Option<char>,
}

impl Coordinate {
    /// Get the coordinate on the given game map representing an antinode location for this
    /// coordinate based on the location of a given coordinate
    fn get_antinode_for(&self, coordinate: &Coordinate, map: &GameMap) -> Option<Coordinate> {
        let (x_diff, y_diff) = self.get_distance_from(coordinate);
        let new_x = self.x + x_diff;
        let new_y = self.y + y_diff;
        map.at(new_x, new_y)
    }

    /// Gets the distance of this coordinate from another coordinate
    fn get_distance_from(&self, coordinate: &Coordinate) -> (i64, i64) {
        let x_diff = self.x - coordinate.x;
        let y_diff = self.y - coordinate.y;
        (x_diff, y_diff)
    }
}

impl PartialEq for Coordinate {
    fn eq(&self, other: &Coordinate) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Hash for Coordinate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

/// Representation of the game map
struct GameMap {
    spaces: Vec<Vec<Coordinate>>,
}

impl GameMap {
    /// Create a new blank game map
    fn new() -> Self {
        Self { spaces: Vec::new() }
    }

    /// Get the coordinate at a given X, Y coordinate
    ///
    /// Returns the Coordinate with the given location, or None if it's
    /// outside the bounds of the map
    fn at(&self, x: i64, y: i64) -> Option<Coordinate> {
        if x < 0 || y < 0 {
            return None;
        }

        match self.spaces.get(y as usize) {
            Some(row) => return row.get(x as usize).copied(),
            None => None,
        }
    }

    /// Gets all the antennas from the map, grouped by frequency (label) in a hash map
    fn get_antennas(&self) -> HashMap<char, HashSet<Coordinate>> {
        // Create a hash map for storing the antenna information
        let mut locations: HashMap<char, HashSet<Coordinate>> = HashMap::new();

        // Iterate through the map inspecting coordinates
        for row in &self.spaces {
            for coord in row {
                // If the coordinate has an antenna, add it to the stored hash set (adding one if
                // this is the first access)
                if let Some(label) = coord.antenna {
                    let set = locations.entry(label).or_default();
                    set.insert(*coord);
                }
            }
        }

        // Return the hash map of all the antenna locations
        locations
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
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the game map
    let map = parse_map(&contents);

    // Get all the antinodes for the antennas
    let mut all_antinodes: HashSet<Coordinate> = HashSet::new();
    for (_label, antenna_set) in map.get_antennas() {
        let antenna_set_antinodes = get_antinodes(&antenna_set, &map);
        all_antinodes.extend(&antenna_set_antinodes);
    }

    // Print the number of valid antinodes calculated
    let num_antinodes = all_antinodes.len();
    println!("{num_antinodes}");
}

// Runs part two
fn main_part_two(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the game map
    let map = parse_map(&contents);

    // Get all the antinodes for the antennas and add them to a running hash set
    let mut all_antinodes: HashSet<Coordinate> = HashSet::new();
    for (_label, antenna_set) in map.get_antennas() {
        let antenna_set_antinodes = get_resonant_antinodes(&antenna_set, &map);
        all_antinodes.extend(&antenna_set_antinodes);
    }

    // Print the number of valid antinodes calculated
    let num_antinodes = all_antinodes.len();
    println!("{num_antinodes}");
}

/// Parse the string to build a game map
fn parse_map(input: &str) -> GameMap {
    // Create a new map
    let mut map = GameMap::new();

    // Iterate through the file contents line by line
    for (row_index, line) in input.lines().filter(|x| !x.is_empty()).enumerate() {
        // Start a new row for the given line of data
        let mut row = Vec::new();

        // Iterate through the line character by character
        for (col_index, character) in line.chars().enumerate() {
            // Initialize a coordinate for the map
            let mut coordinate = Coordinate {
                x: col_index as i64,
                y: row_index as i64,
                antenna: None,
            };

            // If the character is an antenna, save it
            if character != '.' {
                coordinate.antenna = Some(character);
            }

            // Add the coordinate to the row
            row.push(coordinate);
        }

        // Add the row to the map space
        map.spaces.push(row);
    }

    // Return the completed map
    map
}

/// Gets the antinodes for a given set of antennas of the same frequency
fn get_antinodes(antennas: &HashSet<Coordinate>, map: &GameMap) -> HashSet<Coordinate> {
    // Create a new hash set for store antinodes that are found
    let mut antinodes = HashSet::new();

    // Iterate through all the permutations of the given set of antennas
    for antenna_pair in antennas.iter().permutations(2) {
        // Get the base antenna and paired antenna (paired <--> base <--> antinode)
        let base_antenna = antenna_pair[0];
        let paired_antenna = antenna_pair[1];

        // If an antinode can be found, add it to the hash set
        if let Some(antinode) = base_antenna.get_antinode_for(paired_antenna, map) {
            antinodes.insert(antinode);
        }
    }

    // Return all antinodes found
    antinodes
}

/// Gets the resonant antinodes for a given set of antennas of the same frequency
fn get_resonant_antinodes(antennas: &HashSet<Coordinate>, map: &GameMap) -> HashSet<Coordinate> {
    // Create a new hash set for store antinodes that are found
    let mut antinodes = HashSet::new();

    // Iterate through all the permutations of the given set of antennas
    for antenna_pair in antennas.iter().permutations(2) {
        // Get the original base antenna and paired antenna (paired <-> base <--> antinode)
        let mut base_antenna = *antenna_pair[0];
        let mut paired_antenna = *antenna_pair[1];

        // While antinodes can be found, add them to the hash set, and move down then line to detect again
        while let Some(antinode) = base_antenna.get_antinode_for(&paired_antenna, map) {
            antinodes.insert(antinode);
            paired_antenna = base_antenna;
            base_antenna = antinode;
        }

        // If there are two antennas, both are also antinodes, so add them to the hash set
        antinodes.extend(antenna_pair);
    }

    // Return the completed list of antinodes
    antinodes
}
