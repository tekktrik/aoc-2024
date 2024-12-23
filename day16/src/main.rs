use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::hash::Hash;

use clap::Parser;

/// Type for the reindeer scores
type Score = u64;

/// Type for the transit nodes
type Transit = (Coordinate, Direction);

// Type for the transit node information (score and previous node)
type NodeInfo = (Score, Transit);

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Representation of a map coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate {
    x: isize,
    y: isize,
}

impl Coordinate {
    /// Gets the coordinate in a specific direction relative to this one
    fn coordinate_for(&self, direction: &Direction) -> Coordinate {
        match direction {
            Direction::North => Coordinate::from((self.x, self.y - 1)),
            Direction::South => Coordinate::from((self.x, self.y + 1)),
            Direction::East => Coordinate::from((self.x + 1, self.y)),
            Direction::West => Coordinate::from((self.x - 1, self.y)),
        }
    }

    // Gets the coordinates in the cardinal directions from the given coordinate
    fn cardinals(&self) -> Vec<(Coordinate, Direction)> {
        let mut coords = Vec::new();
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            coords.push((self.coordinate_for(&direction), direction));
        }
        coords
    }
}

impl From<(isize, isize)> for Coordinate {
    fn from(value: (isize, isize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<Coordinate> for (isize, isize) {
    fn from(value: Coordinate) -> Self {
        (value.x, value.y)
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// The vaarious directions in which entities can move
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let character = match self {
            Direction::North => '^',
            Direction::East => '>',
            Direction::South => 'v',
            Direction::West => '<',
        };
        write!(f, "{}", character)
    }
}

/// Representation of the game map
#[derive(Debug)]
struct GameMap {
    start: Coordinate,
    end: Coordinate,
    visited: HashMap<Transit, NodeInfo>,
    unvisited: HashMap<Transit, NodeInfo>,
    spaces: HashSet<Coordinate>,
    width: usize,
    height: usize,
}

impl Display for GameMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut map_string = String::new();

        for row_index in 0..self.height {
            for col_index in 0..self.width {
                let coord = Coordinate::from((col_index as isize, row_index as isize));
                if coord == self.start {
                    map_string.push('S');
                } else if coord == self.end {
                    map_string.push('E');
                } else if self.check_free(&coord).unwrap() {
                    map_string.push('.');
                } else {
                    map_string.push('#');
                }
            }

            map_string.push('\n');
        }

        write!(f, "{}", map_string)
    }
}

impl GameMap {
    /// Checks whether a given coordinate is empty
    ///
    /// Returns an error if the space is off the map.
    fn check_free(&self, coord: &Coordinate) -> Result<bool, ()> {
        //
        if coord.x < 0
            || coord.y < 0
            || coord.x >= self.width as isize
            || coord.y >= self.height as isize
        {
            return Err(());
        }

        Ok(self.spaces.contains(coord))
    }

    /// Gets the valid moves in cardinal directions
    fn valid_cardinal_moves(&self, transit: &Transit) -> Vec<Transit> {
        let (coord, _direction) = transit;
        let cardinal_moves = coord.cardinals();
        cardinal_moves
            .iter()
            .filter(|m| self.check_free(&m.0).expect("Invalid space"))
            .copied()
            .collect()
    }

    /// Gets the next available moves for a given transit node
    ///
    /// This is needed for Dijkstra's algorithm, for calculating the new scores for connections
    /// from the current node being used (the given transit node).
    ///
    /// Returns a list of new coordinates the given transit node connects to, the direction
    /// to travel in order to reach that coordinate, and the score associated with that move.
    fn next_moves(&self, transit: &Transit) -> Vec<(Coordinate, Direction, Score)> {
        // Break up the current transit node into its base components for ease of use
        let (.., direction) = transit;

        // Get all the valid moves that can be performed from the given transit node
        let valid_moves: Vec<Transit> = self.valid_cardinal_moves(transit);

        // Get the list of valid moves that would go to coordinates not yet visited
        let new_moves: Vec<Transit> = valid_moves
            .iter()
            .filter(|m| !self.visited.contains_key(m))
            .copied()
            .collect();

        // Get the score of the current transit node
        let (current_score, ..) = self
            .unvisited
            .get(transit)
            .expect("Could not get visited node");

        // Create a list for new moves with associated scores
        let mut new_scored_moves = Vec::new();

        // Iterate through the new moves
        for (new_coordinate, new_direction) in new_moves {
            // Add one to the score for the associated move forward
            let mut new_score = current_score + 1;

            // If the direction of the move is not the current direction, add 1000 points for the necessary turn
            if *direction != new_direction {
                new_score += 1000;
            }

            // Add the new move set to the list
            new_scored_moves.push((new_coordinate, new_direction, new_score));
        }

        // Return the list of moves
        new_scored_moves
    }

    /// Gets the closet (score-wise) unvisited node
    fn get_closest_unvisited(&self) -> (Coordinate, Direction) {
        // If there is only one unvisited node, return that one
        if self.unvisited.len() == 1 {
            return *self
                .unvisited
                .iter()
                .last()
                .expect("Could not get last element")
                .0;
        }

        // Get return the unvisited node with the lowest score
        *self
            .unvisited
            .iter()
            .min_by(|x, y| x.1 .0.cmp(&y.1 .0))
            .expect("No items to sort out minimum")
            .0
    }

    /// Performs a single iteration of Dijkstra's algorithm
    fn perform_dijkstra_iteration(&mut self) {
        // Get the closest (score-wise) node from the start
        let closest_transit = self.get_closest_unvisited();

        // Get the neighbor connections/moves from the closest node
        let next_moves = self.next_moves(&closest_transit);

        // Iterate through the connections/moves
        for (next_coordinate, next_direction, next_score) in next_moves {
            // Create the new transit node for the given coordinate and direction of the move
            let next_transit = (next_coordinate, next_direction);

            // Update the set of unvisited nodes
            match self.unvisited.get_mut(&next_transit) {
                // This transit nodes has been visited before
                Some(info) => {
                    // If new score would be at least as large as the stored one, ignore
                    if next_score >= info.0 {
                        continue;
                    }

                    // Otherwise, update the score for this node
                    info.0 = next_score;
                }
                // This transit node is being visied for the first time
                None => {
                    // Add an entry for this transit node
                    self.unvisited
                        .insert(next_transit, (next_score, closest_transit));
                }
            }
        }

        // Remove the current transit node from the unvisited set, and add it to the visited set
        let closest_entry = self
            .unvisited
            .remove(&closest_transit)
            .expect("Could not complete marking as visited");
        self.visited.insert(closest_transit, closest_entry);
    }

    /// Visit all nodes in the maze, using Dijkstra's algorithm
    fn visit_nodes(&mut self) {
        loop {
            if self.unvisited.is_empty() {
                break;
            }
            self.perform_dijkstra_iteration();
        }
    }

    /// Gets the end node entry with the minimum score
    fn get_best_end_node(&self) -> (&Transit, &(Score, Transit)) {
        self.visited
            .iter()
            .filter(|m| m.0 .0 == self.end)
            .min_by(|x, y| x.1 .0.cmp(&y.1 .0))
            .expect("Could not get end score")
    }

    /// Rewinds a completed map to find all best possible routes
    fn rewind_route(&self, current_transit: Transit, best_locations: &mut HashSet<Coordinate>) {
        // Add this coordinate to the list of best locations
        best_locations.insert(current_transit.0);

        // If the current transit node being analyzed is the start, no need to continuing searching
        if current_transit.0 == self.start {
            return;
        }

        // Get the information for the current transit node
        let current_info = self
            .visited
            .get(&current_transit)
            .expect("Could not get current info");

        // Get the valid cardinal moves from the current transit node
        let valid_cardinal_moves = self.valid_cardinal_moves(&current_transit);

        // Iterate through each coordinate in the valid cardinal moves
        for (coordinate, ..) in valid_cardinal_moves {
            // If the coordinate is already a best location, skip further analysis
            if best_locations.contains(&coordinate) {
                continue;
            }

            // Get the applicable visit nodes with the coordinate of the move
            let applicable_visits: HashMap<&Transit, &(Score, Transit)> = self
                .visited
                .iter()
                .filter(|v| v.0 .0 == coordinate)
                .collect();

            // Iterate through each applicable visit node
            for (applicable_visit_transit, applicable_visit_info) in applicable_visits {
                // The directions of the current transit node and the applicable visit node are the same
                if applicable_visit_transit.1 == current_transit.1 {
                    // If the score is different by 1, it is valid, and the rewind can continue via this node
                    if applicable_visit_info.0 == current_info.0 - 1 {
                        self.rewind_route(*applicable_visit_transit, best_locations);
                    }
                }
                // The directions of the current transit node and applicable visit node are different
                else {
                    // If the score is different by 1001, it is valid, and the rewind can continue via this node
                    if applicable_visit_info.0 == current_info.0 - 1001 {
                        self.rewind_route(*applicable_visit_transit, best_locations);
                    }
                }
            }
        }
    }

    /// Backtracks from the end node to the start node to find all coordinates associated
    /// with a best possible route
    fn backtrack(&self) -> HashSet<Coordinate> {
        // Create a hash set for storing the best locations
        let mut best_locations = HashSet::new();

        // Get all end transit nodes with the lowest score
        let best_end_node = self.get_best_end_node();
        let end_nodes: HashMap<&Transit, &(Score, Transit)> = self
            .visited
            .iter()
            .filter(|m| m.0 .0 == self.end && m.1 .0 == best_end_node.1 .0)
            .collect();

        // Rewind through the applicable end nodes
        for end_node in end_nodes {
            self.rewind_route(*end_node.0, &mut best_locations);
        }

        // Return the set of best locations
        best_locations
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

    // Parse the input file contents into the game map
    let mut gamemap = parse_game(&contents);

    // Visit all possible nodes in the game map
    gamemap.visit_nodes();

    // Get the best possible score for reaching the end
    let final_entry = gamemap.get_best_end_node();
    let final_score = final_entry.1 .0;
    println!("{final_score}");
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the input file contents into the game map
    let mut gamemap = parse_game(&contents);

    // Visit all possible nodes in the game map
    gamemap.visit_nodes();

    // Backtrack from the end node to find all possible best locations
    let best_locations = gamemap.backtrack();

    // Print the number of best locations
    let num_locations = best_locations.len();
    println!("{num_locations}");
}

/// Parses the given string into the game map
fn parse_game(text: &str) -> GameMap {
    // Create default start and end nodes
    let mut start = Coordinate::from((0, 0));
    let mut end = Coordinate::from((0, 0));

    // Create a list for storing empty spaces
    let mut spaces = HashSet::new();

    // Iterate through the string character by character
    for (row_index, line) in text.trim().lines().enumerate() {
        for (col_index, character) in line.chars().enumerate() {
            // Get the current coordinate based on the iteration
            let coord = Coordinate::from((col_index as isize, row_index as isize));

            // Handle each space on the map
            match character {
                '.' => {
                    spaces.insert(coord);
                }
                'S' => {
                    start = coord;
                    spaces.insert(coord);
                }
                'E' => {
                    end = coord;
                    spaces.insert(coord);
                }
                '#' => continue,
                _char => panic!("Encountered unrecognized character: {_char}"),
            }
        }
    }

    // Calculate the height and width of the map from the given string
    let height = text.trim().lines().count();
    let width = text.trim().lines().last().unwrap().len();

    // Create the set of unvisited nodes, seeding the start node into it
    let mut unvisited = HashMap::new();
    let start_transit = (start, Direction::East);
    let start_node = (0, start_transit);
    unvisited.insert(start_transit, start_node);

    // Return the finalized game map
    GameMap {
        start,
        end,
        visited: HashMap::new(),
        unvisited,
        spaces,
        width,
        height,
    }
}
