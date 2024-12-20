use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::fs;

use clap::Parser;

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
            Direction::Up => Coordinate::from((self.x, self.y - 1)),
            Direction::Down => Coordinate::from((self.x, self.y + 1)),
            Direction::Right => Coordinate::from((self.x + 1, self.y)),
            Direction::Left => Coordinate::from((self.x - 1, self.y)),
        }
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

/// Representation of an entity on the map
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Entity {
    /// Unique identifier
    id: usize,
    /// The coordinate representing the left edge
    left: Coordinate,
    /// The coordinate representing the right edge
    right: Coordinate,
    /// Whether the entity is moveable
    moveable: bool,
}

impl Entity {
    /// Pushes the box in the given direction, updating it's coordinates
    fn slide(&mut self, direction: &Direction) {
        self.left = self.left.coordinate_for(direction);
        self.right = self.right.coordinate_for(direction);
    }
}

/// The vaarious directions in which entities can move
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

/// Representation of the game map
#[derive(Debug, Clone)]
struct GameMap {
    robot: Entity,
    entities: Vec<Entity>,
    instructions: Vec<Direction>,
    width: usize,
    height: usize,
    wide: bool,
}

impl GameMap {
    /// Parses the game map from the provided string
    fn parse(text: &str, wide: bool) -> Self {
        // Split the given text into the map and instructions portion
        let text_parts: Vec<&str> = text.split("\n\n").collect();
        let map_text = text_parts[0];
        let instruction_text = text_parts[1];

        // Parse the map from the map text
        let mut map = Self::parse_map(map_text, wide);

        // Parse the instructions from the instruction text in the map
        map.instructions = Self::parse_instructions(instruction_text);

        // Return the finalized map
        map
    }

    /// Parses the map text portion
    fn parse_map(map_text: &str, wide: bool) -> Self {
        // Initialize the robot
        let template_coord = Coordinate::from((0, 0));
        let mut robot = Entity {
            id: 0,
            left: template_coord,
            right: template_coord,
            moveable: true,
        };

        // Create a list for storing entities
        let mut entities = Vec::new();

        // Create an id for uniquely identifing entities
        let mut id = 0;

        // Iterate through the characters of the map text
        for (row_index, row) in map_text.trim().lines().enumerate() {
            for (col_index, character) in row.chars().enumerate() {
                // Increment the unique identifier
                id += 1;

                // Get the column index, depending on whether the map should be widened
                let wide_col_index = if wide { col_index * 2 } else { col_index };

                // Get the left and right coordinates
                let left_coord = Coordinate::from((wide_col_index as isize, row_index as isize));
                let right_coord = if wide {
                    Coordinate::from((wide_col_index as isize + 1, row_index as isize))
                } else {
                    left_coord
                };

                // Get the entity based on the character in the map
                let entity = match character {
                    '#' => Entity {
                        id,
                        left: left_coord,
                        right: left_coord,
                        moveable: false,
                    },
                    '@' => {
                        robot = Entity {
                            id,
                            left: left_coord,
                            right: left_coord,
                            moveable: true,
                        };
                        continue;
                    }
                    'O' => {
                        // println!("Found obstacle @ {left_coord:?} & {right_coord:?}!");
                        Entity {
                            id,
                            left: left_coord,
                            right: right_coord,
                            moveable: true,
                        }
                    }
                    '.' => continue,
                    _ => panic!("Could not parse character: {character}"),
                };

                // Add the entity to the tracked list
                entities.push(entity);

                // If the map should be widened, more actions are required
                if wide {
                    // Increment the unique identifier
                    id += 1;

                    // Create the second entity if necessary
                    let entity = match character {
                        '#' => Entity {
                            id,
                            left: right_coord,
                            right: right_coord,
                            moveable: false,
                        },
                        'O' | '@' | '.' => continue,
                        _ => panic!("Could not parse character: {character}"),
                    };

                    // Add the second entity to the tracked list
                    entities.push(entity);
                }
            }
        }

        // Calculate the map height and width
        let height = map_text.trim().lines().count();
        let mut width = map_text.trim().lines().last().unwrap().len();
        width = if wide { width * 2 } else { width };

        // Return the map object with a blank set of instructions
        Self {
            robot,
            entities,
            instructions: Vec::new(),
            width,
            height,
            wide,
        }
    }

    /// Parses the instuctions text to return a list of directions for the robot to move
    fn parse_instructions(instruction_text: &str) -> Vec<Direction> {
        // Create a list for storing parsed instructions
        let mut instructions = Vec::new();

        // Iterate through the list of instructions
        for row in instruction_text.trim().lines() {
            for character in row.trim().chars() {
                // Get the direction based on the character encountered
                let direction = match character {
                    '^' => Direction::Up,
                    '>' => Direction::Right,
                    'v' => Direction::Down,
                    '<' => Direction::Left,
                    _ => panic!("Could not parse direction: {character}"),
                };

                // Add the direction to the tracked list
                instructions.push(direction);
            }
        }

        // Return the finalized list of instructions
        instructions
    }

    /// Gets the collisions for a given entity in the given direction
    ///
    /// Returns a hash set of IDs of objects that this entity would collide with
    fn collisions_for(&self, entity: &Entity, direction: &Direction) -> HashSet<usize> {
        // Create a list for keeping track of collisions
        let mut collisions = HashSet::new();

        // Check based on the direction
        match direction {
            // For up and down, check both the left and right coordinates of the entity
            Direction::Up | Direction::Down => {
                let leftside = entity.left.coordinate_for(direction);
                if let Some(left_neightbor) = self.get(&leftside).unwrap() {
                    collisions.insert(left_neightbor.id);
                }

                let rightside = entity.right.coordinate_for(direction);
                if let Some(right_neighbor) = self.get(&rightside).unwrap() {
                    collisions.insert(right_neighbor.id);
                }
            }
            // For left, check the left side of the entity
            Direction::Left => {
                let nextspace = entity.left.coordinate_for(direction);
                if let Some(neighbor) = self.get(&nextspace).unwrap() {
                    collisions.insert(neighbor.id);
                }
            }
            // For right, check the right side of the entity
            Direction::Right => {
                let nextspace = entity.right.coordinate_for(direction);
                if let Some(neighbor) = self.get(&nextspace).unwrap() {
                    collisions.insert(neighbor.id);
                }
            }
        }

        collisions
    }

    /// Moves the robot in the given direction
    fn move_robot(&mut self, direction: &Direction) {
        // Create a list for tracking IDs of entities to move
        let mut moveable_ids = Vec::new();

        // Attempt to push the robot
        self.push_entity(&self.robot.clone(), direction, &mut moveable_ids);

        // For objects that should be moved (if successful), move them
        for moveable_id in moveable_ids {
            self.slide_entity(moveable_id, direction);
        }
    }

    /// Push the given entity in the given direction, checking for collisions and
    /// recursively pushing as needed
    fn push_entity(
        &mut self,
        entity: &Entity,
        direction: &Direction,
        moveable_ids: &mut Vec<usize>,
    ) -> bool {
        // Get IDs for objects that the entity collides with
        let next_ids = self.collisions_for(entity, direction);

        // Keep a copy of the current moveable IDs in case the move fails
        let moveable_ids_clone = moveable_ids.clone();

        // Iterate through the colliding entity IDs to check if the can be pushed
        for next_id in next_ids {
            // Get the entity with the ID
            let neighbor = self.get_by_id(next_id);

            // If the neighbor is not moveable, reset the moveable IDs to before any moves
            // and return the failure result
            if !neighbor.moveable {
                *moveable_ids = moveable_ids_clone;
                return false;
            }

            // Get the next moveable entity
            let next_entity = *self.get_by_id(next_id);

            // If the next entity cannot be moved, reset the moveable IDs to before any moves
            // and return the failure result
            if !self.push_entity(&next_entity, direction, moveable_ids) {
                *moveable_ids = moveable_ids_clone;
                return false;
            }
        }

        // Add the current entity ID to the list of moveable IDs and return the success result
        moveable_ids.push(entity.id);
        true
    }

    /// Slides an entity with the given ID in the given direction
    fn slide_entity(&mut self, id: usize, direction: &Direction) {
        let entity = self.get_by_id_mut(id);
        entity.slide(direction);
    }

    /// Gets the entity at a given coordinate
    ///
    /// Returns the a space if valid, or None if it's outside the bounds of the map.
    /// The answer is either the entity in the location, or None if it is empty.
    fn get(&self, coord: &Coordinate) -> Option<Option<&Entity>> {
        if coord.x < 0
            || coord.y < 0
            || coord.x >= self.width as isize
            || coord.y >= self.height as isize
        {
            return None;
        }

        match self.entities.iter().position(|e| {
            (e.left.x == coord.x && e.left.y == coord.y)
                || (e.right.x == coord.x && e.right.y == coord.y)
        }) {
            Some(pos) => Some(Some(&self.entities[pos])),
            None => {
                if self.robot.left.x == coord.x && self.robot.left.y == coord.y {
                    Some(Some(&self.robot))
                } else {
                    Some(None)
                }
            }
        }
    }

    /// Gets the entity at a given coordinate by ID
    fn get_by_id(&self, id: usize) -> &Entity {
        if id == self.robot.id {
            return &self.robot;
        }

        let pos = self
            .entities
            .iter()
            .position(|e| (e.id == id))
            .unwrap_or_else(|| panic!("Could not get entity with the given ID: {id}"));
        &self.entities[pos]
    }

    /// Gets the entity at a given coordinate by ID, mutably
    fn get_by_id_mut(&mut self, id: usize) -> &mut Entity {
        if id == self.robot.id {
            return &mut self.robot;
        }

        let pos = self
            .entities
            .iter()
            .position(|e| (e.id == id))
            .unwrap_or_else(|| panic!("Could not get entity with the given ID: {id}"));
        &mut self.entities[pos]
    }

    /// Gets the GPS coordinates for all moveable entities on the map
    fn gps_coordinates(&self) -> Vec<u128> {
        self.entities
            .iter()
            .filter(|x| x.moveable)
            .map(|e| (100 * e.left.y as u128) + e.left.x as u128)
            .collect()
    }
}

impl Display for GameMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut map_string = String::new();

        let mut skip_next = false;

        for row_index in 0..self.height as isize {
            for col_index in 0..self.width as isize {
                match self.get(&Coordinate::from((col_index, row_index))).unwrap() {
                    Some(entity) => {
                        if skip_next {
                            // println!("Remove skip flag");
                            skip_next = false;
                            continue;
                        } else if !entity.moveable {
                            map_string.push('#');
                        } else if entity.left == self.robot.left {
                            map_string.push('@');
                        } else if self.wide {
                            map_string.push_str("[]");
                            skip_next = true;
                        } else {
                            map_string.push('O');
                        }
                    }
                    None => map_string.push('.'),
                }
            }

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

    // Parse the input file contents into the game map
    let mut gamemap = GameMap::parse(&contents, false);

    // Play out the instructions
    for instruction in &gamemap.instructions.clone() {
        gamemap.move_robot(instruction);
    }

    // Print the sum of the GPS coordinates
    let gps_sum: u128 = gamemap.gps_coordinates().iter().sum();
    println!("{gps_sum}");
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the input file contents into the game map
    let mut gamemap = GameMap::parse(&contents, true);

    // Play out the instructions
    for instruction in gamemap.instructions.clone() {
        gamemap.move_robot(&instruction);
    }

    // Print the sum of the GPS coordinates
    let gps_sum: u128 = gamemap.gps_coordinates().iter().sum();
    println!("{gps_sum}");
}
