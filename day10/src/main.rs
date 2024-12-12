use std::{
    collections::{HashMap, HashSet},
    fs,
};

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Representation of an X, Y coordinate pair
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Coordinate {
    x: i64,
    y: i64,
}

impl From<(i64, i64)> for Coordinate {
    fn from(value: (i64, i64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<Coordinate> for (i64, i64) {
    fn from(value: Coordinate) -> Self {
        (value.x, value.y)
    }
}

/// Representation of a location on the topography map, with coordinate and level
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Location {
    coord: Coordinate,
    level: u8,
}

/// Representation of the game map
struct GameMap {
    spaces: Vec<Vec<Location>>,
}

impl GameMap {
    // Creates a new blank map
    fn new(spaces: Vec<Vec<Location>>) -> Self {
        Self { spaces }
    }

    // Parses the map from the provided string
    fn parse(value: &str) -> Self {
        // Create a list for storing rows
        let mut rows = Vec::new();

        // Iterate through the string line by line
        for (row_index, line) in value.trim().lines().enumerate() {
            // Create a list for storing entries
            let mut row = Vec::new();

            // Iterate through the line character by character
            for (col_index, character) in line.chars().enumerate() {
                // Create the coordinate for the given position
                let coord = Coordinate {
                    x: col_index as i64,
                    y: row_index as i64,
                };

                // Add the location (with topography) to the row
                row.push(Location {
                    coord,
                    level: character.to_digit(10).expect("Could not parse level") as u8,
                });
            }

            // Add the row to the map
            rows.push(row);
        }

        // Return a new map with the given rows
        Self::new(rows)
    }

    /// Get the location at a given X, Y coordinate
    ///
    /// Returns the requested location if valid, or None if it's
    /// outside the bounds of the map
    fn get(&self, coord: &Coordinate) -> Option<&Location> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }

        match self.spaces.get(coord.y as usize) {
            Some(row) => row.get(coord.x as usize),
            None => None,
        }
    }

    /// Gets the valid neighboring squares in the cardinal directions
    fn neighbors(&self, coord: &Coordinate) -> Vec<&Location> {
        // Create a list to store the neighboring locations
        let mut neighbors = Vec::new();

        // Shorthands for x and y
        let x = coord.x;
        let y = coord.y;

        // Get the coordinates at the cardinal directions
        let north = Coordinate::from((x, y + 1));
        let east = Coordinate::from((x + 1, y));
        let south = Coordinate::from((x, y - 1));
        let west = Coordinate::from((x - 1, y));

        // Add the coordinates to the list of neighbors
        neighbors.push(self.get(&north));
        neighbors.push(self.get(&east));
        neighbors.push(self.get(&south));
        neighbors.push(self.get(&west));

        // Filter out invalid neighboring coordinates
        neighbors.iter().filter_map(|x| *x).collect()
    }

    /// Gets neighboring locations that are a single step up from the given location
    fn up_from(&self, loc: &Location) -> Vec<&Location> {
        self.neighbors(&loc.coord)
            .iter()
            .copied()
            .filter(|x| x.level == loc.level + 1)
            .collect()
    }

    /// Gets the trail ratings for a given start location
    fn find_complete_trails(&self, path: &[Location]) -> HashSet<Vec<Location>> {
        // Create a set of trails for the given start location
        let mut trails = HashSet::new();

        // If the first element isn't a 0, short circuit return the empty trail list
        if path.first().expect("Path is empty").level != 0 {
            return trails;
        }

        // Get the last location in the provided path
        let loc = path.last().expect("Path is empty");

        // Convert that path to a mutable list for try to find a complete trail
        let mut trial_path = Vec::from(path);

        // If the location is level 9, it's the end of the trail
        if loc.level == 9 {
            // Add the final point to the trail list
            trial_path.push(*loc);

            // Insert the final trail into the set of trails (only entry) and return it
            trails.insert(trial_path);
            return trails;
        }

        // Iterate through all valid next steps up from the current location
        for next_step in self.up_from(loc) {
            // Add the next step up to the trail list
            trial_path.push(*next_step);

            // Recursively search for completed paths and add them to the complete set of trails
            let additional_ends = self.find_complete_trails(&trial_path);
            trails.extend(additional_ends);
        }

        // Return the list of completed trails
        trails
    }

    // Get all of the trails, grouped by start location
    fn get_trails(&self) -> HashMap<Location, HashSet<Vec<Location>>> {
        // Create a hash map for storing trails
        let mut trails = HashMap::new();

        // Iterate through the map point by point
        for row in &self.spaces {
            for loc in row {
                // If the point isn't a start location (0), skip it
                if loc.level != 0 {
                    continue;
                }

                // Get the set of complete trails from this point
                let complete_trails = self.find_complete_trails(&[*loc]);

                // Insert the set of trails into the hash map
                trails.insert(*loc, complete_trails);
            }
        }

        // Return the completed hash map of trails
        trails
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

    // Create the game map from the file contents
    let map = GameMap::parse(&contents);

    // Calculate the scores for the map
    let ratings = map.get_trails();
    let scores = convert_ratings_to_scores(ratings);

    // Print the sum of the scores
    let mut total_score = 0;
    scores.iter().for_each(|(_i, x)| total_score += x.len());
    println!("{total_score}");
}

/// Runs part one
fn main_part_two(filepath: String) {
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Create the game map from the file contents
    let map = GameMap::parse(&contents);

    // Calculate the scores for the map
    let ratings = map.get_trails();

    // Print the sum of the rating
    let mut total_ratings = 0;
    ratings.iter().for_each(|(_i, x)| total_ratings += x.len());
    println!("{total_ratings}");
}

// Convert trail ratings into trail scores
fn convert_ratings_to_scores(
    ratings: HashMap<Location, HashSet<Vec<Location>>>,
) -> HashMap<Location, HashSet<Location>> {
    // Create a map of scores per trail start location
    let mut scores = HashMap::new();

    // Iterate through the ratings
    for (key, trails) in ratings {
        // Create a new set of trail endpoints for each start location
        let mut ends: HashSet<Location> = HashSet::new();

        // For each trail in the ratings (completed trail), insert the endpoint to the running set
        for trail in trails {
            ends.insert(*trail.last().expect("Path is empty"));
        }

        // Insert the set of trail endpoints into the score hash map
        scores.insert(key, ends);
    }

    // Return the hash map of scores
    scores
}
