use std::{collections::HashSet, fs};

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Directions of travel for the guard
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

/// Possible actions the guard can take
#[derive(Clone, Copy, PartialEq)]
enum Action {
    /// Check to see if the next action is a move, turn, or removal
    Check,
    /// Save the guard's current locaion
    Save,
    /// Turn the guard 90 degrees
    Turn,
    /// Move the guard one space
    Move,
    /// Remove the guard from the game map
    Remove,
}

/// A coordinate on the game map
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Coordinate {
    x: usize,
    y: usize,
    blockage: bool,
}

/// A guard movement, consisting of both the location and direction of movement
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Movement {
    coordinate: Coordinate,
    direction: Direction,
}

/// Guard to play the game on the map
#[derive(Clone)]
struct Guard {
    location: Option<Coordinate>,
    direction: Direction,
    next_action: Action,
    history: HashSet<Movement>,
}

impl Guard {
    /// Create a new guard
    ///
    /// This guard MUST be initialized with information such as location
    /// before they can play the game.
    fn new() -> Self {
        Self {
            location: None,
            direction: Direction::North,
            next_action: Action::Save,
            history: HashSet::new(),
        }
    }

    /// Turn the guard
    fn turn(&mut self) {
        self.direction = get_turn_direction(&self.direction);
        self.next_action = Action::Save;
    }

    /// Get the next movement space for the guard
    fn get_next_move_space(&self) -> (i64, i64) {
        get_next_move_space(&self.location.expect("Location not set!"), &self.direction)
    }

    /// Check what the next "movement" action should be for the guard
    fn check(&mut self, map: &GameMap) {
        // Get the coordinates of the next move space
        let (next_x, next_y) = self.get_next_move_space();

        // If the movement is not to a valid space, remove the guard
        if !map.is_valid_space(next_x, next_y) {
            self.next_action = Action::Remove;
            return;
        }

        // If the space is free then move, otherwise turn
        if map.is_free(next_x, next_y) {
            self.next_action = Action::Move;
        } else {
            self.next_action = Action::Turn;
        }
    }

    // Save the current location of the guard
    fn save_location(&mut self) {
        let movement = Movement {
            coordinate: self.location.expect("Location is not set"),
            direction: self.direction,
        };

        self.history.insert(movement);
        self.next_action = Action::Check;
    }

    // Move the guard a single space forward
    fn move_space(&mut self) {
        match self.next_action {
            Action::Move => {
                let (x, y) = self.get_next_move_space();
                let coordinate = Coordinate {
                    x: x as usize,
                    y: y as usize,
                    blockage: false,
                };
                self.location = Some(coordinate);
                self.next_action = Action::Save;
            }
            _ => panic!("Cannot perform guard movement if Action::Move is not set!"),
        }
    }
}

/// The game map on which the guard plays
#[derive(Clone)]
struct GameMap {
    space_map: Vec<Vec<Coordinate>>,
    start_location: Option<Coordinate>,
}

impl GameMap {
    /// Create a new map
    ///
    /// This MUST be initialized with data before the game can be played.
    fn new() -> Self {
        Self {
            space_map: Vec::new(),
            start_location: None,
        }
    }

    /// Checks whether the space requested is a valid map coordinate (on the map)
    fn is_valid_space(&self, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        match self.space_map.get(y as usize) {
            Some(row) => return row.get(x as usize).is_some(),
            None => false,
        }
    }

    /// Checks whether the space requested is free on the map (not blocked)
    fn is_free(&self, x: i64, y: i64) -> bool {
        if !self.is_valid_space(x, y) {
            true
        } else {
            !self.space_map[y as usize][x as usize].blockage
        }
    }

    /// Add an obstacle to the map at a given coordinate
    fn add_obstacle(&mut self, coordinate: &Coordinate) {
        let x = coordinate.x;
        let y = coordinate.y;
        self.space_map[y][x].blockage = true;
    }

    /// Remove an obstacle from the map at a given coordinate
    fn remove_obstacle(&mut self, coordinate: &Coordinate) {
        let x = coordinate.x;
        let y = coordinate.y;
        self.space_map[y][x].blockage = false;
    }
}

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

/// Parse the game from the file contents provided
fn parse_game(input: &str) -> (Guard, GameMap) {
    // Create a new guard and map
    let mut guard = Guard::new();
    let mut map = GameMap::new();

    // Iterate through the file contents line by line
    for (row_index, line) in input.lines().filter(|x| !x.is_empty()).enumerate() {
        // Start a new row for the given line of data
        let mut row = Vec::new();

        // Iterate through the line character by character
        for (col_index, character) in line.chars().enumerate() {
            // Initialize a coordinate for the map
            let mut coordinate = Coordinate {
                x: col_index,
                y: row_index,
                blockage: false,
            };

            // If the character is occupied-symbol, mark it as blocked
            if character == '#' {
                coordinate.blockage = true;
            }
            // Otherwise, if it's the guard, store the location in the appropriate places
            else if character == '^' {
                guard.location = Some(coordinate);
                map.start_location = Some(coordinate);
            }

            // Add the coordinate to the row
            row.push(coordinate);
        }

        // Add the row to the map space
        map.space_map.push(row);
    }

    // Return both the guard and map
    (guard, map)
}

/// Plays the game until the guard is removed
fn play_game(guard: &mut Guard, map: &GameMap) {
    while play_round(guard, map) {}
}

/// Plays the next round for the guard on the map
///
/// Returns whether the game is good to be played again
fn play_round(guard: &mut Guard, map: &GameMap) -> bool {
    match guard.next_action {
        Action::Check => guard.check(map),
        Action::Move => guard.move_space(),
        Action::Save => guard.save_location(),
        Action::Turn => guard.turn(),
        Action::Remove => return false,
    }
    true
}

/// Gets the next direction for a turn when facing a given direction
fn get_turn_direction(direction: &Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
    }
}

/// Get the next coordinates for a given location and direction
///
/// These are given in a pair of i64, which should then be checked for validity on the map
fn get_next_move_space(coordinate: &Coordinate, direction: &Direction) -> (i64, i64) {
    let x = coordinate.x as i64;
    let y = coordinate.y as i64;
    match direction {
        Direction::North => (x, y - 1),
        Direction::East => (x + 1, y),
        Direction::South => (x, y + 1),
        Direction::West => (x - 1, y),
    }
}

/// Analyze the route taken by the guard, simularing placing obstacles on their route
fn analyze_guard_route(guard: &Guard, map: &GameMap) -> usize {
    // Initialize a hash set of locations for store locations causing loops
    let mut looping_locations = HashSet::new();

    // Iterate through the guards movement history
    for movement in &guard.history {
        // Create a new guard at the original location for re-simulating the effect of the new obstacles
        let mut trial_guard = Guard {
            location: map.start_location,
            direction: Direction::North,
            next_action: Action::Save,
            history: HashSet::new(),
        };

        // Create a copy of the map that can be modified freely
        //
        // This could have been done using a mutable reference to the map, but
        // using a clone of the map signifies that the original map really should
        // not be having changes map to it.
        let mut trial_map = map.clone();

        // If the next move space isn't valid or free, skip checking
        let (obstacle_x, obstacle_y) =
            get_next_move_space(&movement.coordinate, &movement.direction);
        if !map.is_valid_space(obstacle_x, obstacle_y) || !map.is_free(obstacle_x, obstacle_y) {
            continue;
        }

        // Get the coordiantes of the hypothetical obstacle and add it to the map
        let obstacle_coordinate = Coordinate {
            x: obstacle_x as usize,
            y: obstacle_y as usize,
            blockage: true,
        };
        trial_map.add_obstacle(&obstacle_coordinate);

        // Play the games round by round
        while play_round(&mut trial_guard, &trial_map) {
            // Get the latest movement of the guard
            let trial_guard_movement = Movement {
                coordinate: trial_guard
                    .location
                    .expect("Could not get trial guard location"),
                direction: trial_guard.direction,
            };

            // If the movement is already in the guard's history ahead of saving it, they're now in a loop!
            if trial_guard.next_action == Action::Save
                && trial_guard.history.contains(&trial_guard_movement)
            {
                looping_locations.insert(obstacle_coordinate);
                break;
            }
        }

        // Remove the hypothetical obstacle from the map
        trial_map.remove_obstacle(&obstacle_coordinate);
    }

    // Return the number of hypothetical obstacle locations found
    looping_locations.len()
}

fn main_part_one(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Get the guard and the game map from the file contents
    let (mut guard, map) = parse_game(&contents);

    // Let the game play out
    play_game(&mut guard, &map);

    // Print the number of spaces visited
    let spaces_visited: HashSet<Coordinate> = guard.history.iter().map(|x| x.coordinate).collect();
    let num_spaces_visited = spaces_visited.len();
    println!("{num_spaces_visited}");
}

fn main_part_two(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // // Get the guard and the game map from the file contents
    let (mut guard, map) = parse_game(&contents);

    // Let the game play out
    play_game(&mut guard, &map);

    // Analyze the output of the game to find the number of obstacle loop locations
    let num_loopable_locations = analyze_guard_route(&guard, &map);
    println!("{num_loopable_locations}");
}
