use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs,
};

use clap::Parser;

type Distance = usize;

type VisitMap = HashMap<Coordinate, VisitInfo>;

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Coordinates that can be travelled to on the map
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
    fn cardinals(&self) -> Vec<Coordinate> {
        let mut coords = Vec::new();
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            coords.push(self.coordinate_for(&direction));
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

/// The various directions in which the player can move
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

/// Information about specific coordinates visited during Dijkstra's algorithm
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct VisitInfo {
    /// Distance from the start node
    distance: Distance,
    /// The node that led to this one
    previous: Coordinate,
}

/// The map of the program
#[derive(Debug, Clone)]
struct ProgramMap {
    /// The height of the map
    height: usize,
    /// The width of the map
    width: usize,
    /// The start coordinate
    start: Coordinate,
    /// The end coordinate
    end: Coordinate,
    /// Active obstacles on the map
    obstacles: Vec<Coordinate>,
    /// Planned obstacles to be added to the map
    planned_obstacles: Vec<Coordinate>,
    /// The set of visited coordinates
    visited: VisitMap,
    /// The set of discovered but unvisited coordinates
    unvisited: VisitMap,
}

impl ProgramMap {
    /// Parses the program map from the given input text with the given height and width
    fn from_string(text: &str, height: usize, width: usize) -> Self {
        // Create a list for storing the obstacles planned to fall
        let mut planned_obstacles = Vec::new();

        // Iterate through the lines of the input text
        for line in text.trim().lines() {
            // Split the line by the comma
            let numbers: Vec<&str> = line.split(",").collect();

            // Parse the coordinate for the obstacle
            let x = numbers[0]
                .parse::<isize>()
                .expect("Could not parse X coordinate value");
            let y = numbers[1]
                .parse::<isize>()
                .expect("Could not parse X coordinate value");
            let coord = Coordinate::from((x, y));

            // Add the obstacle to the list of planned obstacle
            planned_obstacles.push(coord);
        }

        // Reverse the list of planned obstacles so they can be popped off later
        planned_obstacles.reverse();

        // Get the start and end nodes of the map
        let start = Coordinate::from((0, 0));
        let end = Coordinate::from((width as isize - 1, height as isize - 1));

        // Create the list of unvisited coordinates, seeding the start location
        let mut unvisited = HashMap::new();
        let start_info = VisitInfo {
            distance: 0,
            previous: start,
        };
        unvisited.insert(start, start_info);

        // Create and return the program map
        Self {
            height,
            width,
            start,
            end,
            obstacles: Vec::new(),
            planned_obstacles,
            visited: HashMap::new(),
            unvisited,
        }
    }

    /// Resets the list of visited nodes
    fn reset_visited(&mut self) {
        // Recreate the original set of unvisited coordinates
        let start = self.start;
        let mut unvisited = HashMap::new();
        let start_info = VisitInfo {
            distance: 0,
            previous: start,
        };
        unvisited.insert(start, start_info);

        // Reset the sets of unvisited and visited coordinates
        self.unvisited = unvisited;
        self.visited = HashMap::new();
    }

    /// Checks whether a given coordinate is free of an obstacle
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

        Ok(!self.obstacles.contains(coord))
    }

    /// Gets the valid moves in cardinal directions
    fn valid_cardinal_moves(&self, coord: &Coordinate) -> Vec<Coordinate> {
        let cardinal_moves = coord.cardinals();
        cardinal_moves
            .iter()
            .filter(|m| self.check_free(m).is_ok())
            .filter(|m| self.check_free(m).expect("Invalid space"))
            .copied()
            .collect()
    }

    /// Gets the next available moves for a given coordinate
    ///
    /// This is needed for Dijkstra's algorithm, for calculating the new scores for connections
    /// from the current location being used (the given coordinate).
    ///
    /// Returns a list of new coordinates the given location connects to and the distance from
    /// the start associated with that move.
    fn next_moves(&self, coord: &Coordinate) -> Vec<(Coordinate, Distance)> {
        // Get all the valid moves that can be performed from the given location
        let valid_moves = self.valid_cardinal_moves(coord);

        // Get the list of valid moves that would go to coordinates not yet visited
        let new_moves: Vec<Coordinate> = valid_moves
            .iter()
            .filter(|m| !self.visited.contains_key(m))
            .copied()
            .collect();

        // Get the distance of the current location
        let visit_info = self
            .unvisited
            .get(coord)
            .expect("Could not get visited node");

        // Create a list for new moves with associated scores
        let mut new_measured_moves = Vec::new();

        // Iterate through the new moves
        for new_coordinate in new_moves {
            // Add one to the distance for the associated move forward
            let new_distance = visit_info.distance + 1;

            // Add the new move set to the list of new moves
            new_measured_moves.push((new_coordinate, new_distance));
        }

        // Return the list of new moves
        new_measured_moves
    }

    /// Corrupts the next space, moving the next planned obstacle to the list of active obstacles
    fn corrupt_next_space(&mut self) {
        let next_corruption = self
            .planned_obstacles
            .pop()
            .expect("Could not get next obstacle");
        self.obstacles.push(next_corruption);
    }

    /// Uncorrupts the last space, moving the last planned obstacle to the list of planned obstacles
    fn uncorrupt_next_space(&mut self) {
        let next_corruption = self.obstacles.pop().expect("Could not get next obstacle");
        self.planned_obstacles.push(next_corruption);
    }

    /// Gets the closet unvisited location
    fn get_closest_unvisited(&self) -> Coordinate {
        // If there is only one unvisited location, return that one
        if self.unvisited.len() == 1 {
            return *self
                .unvisited
                .iter()
                .last()
                .expect("Could not get last element")
                .0;
        }

        // Return the unvisited coordinate with the lowest distance
        *self
            .unvisited
            .iter()
            .min_by(|x, y| x.1.distance.cmp(&y.1.distance))
            .expect("No items to sort out minimum")
            .0
    }

    /// Performs a single iteration of Dijkstra's algorithm
    fn perform_dijkstra_iteration(&mut self) {
        // Get the closest coordinate from the start
        let closest_coordinate = self.get_closest_unvisited();

        // Get the neighbor connections/moves from the closest coordinate
        let next_moves = self.next_moves(&closest_coordinate);

        // Iterate through the connections/moves
        for (next_coordinate, next_distance) in next_moves {
            // Update the set of unvisited nodes
            match self.unvisited.get_mut(&next_coordinate) {
                // This coordinate has been visited before
                Some(info) => {
                    // If new distance would be at least as large as the stored one, ignore it
                    if next_distance >= info.distance {
                        continue;
                    }

                    // Otherwise, update the distnace for this node
                    info.distance = next_distance;
                }
                // This coordinate is being visited for the first time
                None => {
                    // Create the new entry
                    let next_visit = VisitInfo {
                        distance: next_distance,
                        previous: closest_coordinate,
                    };

                    // Add an entry for this next coordinate
                    self.unvisited.insert(next_coordinate, next_visit);
                }
            }
        }

        // Remove the current coordinate from the unvisited set, and add it to the visited set
        let closest_entry = self
            .unvisited
            .remove(&closest_coordinate)
            .expect("Could not complete marking as visited");
        self.visited.insert(closest_coordinate, closest_entry);
    }

    /// Visit all possible coordinates in the maze, using Dijkstra's algorithm
    fn visit_nodes(&mut self) {
        loop {
            if self.unvisited.is_empty() {
                break;
            }
            self.perform_dijkstra_iteration();
        }
    }

    /// Presimulate the maze corruption with the first n obstacles
    fn presimulate_corruption(&mut self, n: usize) {
        for _i in 0..n {
            self.corrupt_next_space();
        }
    }
}

impl Display for ProgramMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut map_string = String::new();

        let mut route_coords = Vec::from_iter([self.end]);
        if self.visited.contains_key(&self.end) {
            let mut current_coord = self.end;
            while current_coord != self.start {
                current_coord = self.visited.get(&current_coord).unwrap().previous;
                route_coords.push(current_coord);
            }
        }

        for row_index in 0..self.height as isize {
            for col_index in 0..self.width as isize {
                let coord = Coordinate::from((col_index, row_index));
                if route_coords.contains(&coord) {
                    map_string.push('@');
                } else if self.obstacles.contains(&coord) {
                    map_string.push('#');
                } else {
                    map_string.push('.');
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
    // Get the contents of the given filepath
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the program map from the input text
    let mut program_map = ProgramMap::from_string(&contents, 71, 71);

    // Pre-simulate the first 1024 bytes of corruption
    program_map.presimulate_corruption(1024);

    // Visit all possible locations to find the associated minimum distances
    program_map.visit_nodes();

    // Get the number of steps from the start to the end
    let end = program_map.end;
    let end_info = program_map
        .visited
        .get(&end)
        .expect("Could not get distance to end");
    let end_distance = end_info.distance;
    println!("{end_distance}");
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the contents of the given filepath
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the program map from the input text
    let mut program_map: ProgramMap = ProgramMap::from_string(&contents, 71, 71);

    // Pre-simulate all of the corruption
    program_map.presimulate_corruption(program_map.planned_obstacles.len());

    // Visit all possible locations
    program_map.visit_nodes();

    // While the end cannot be found, uncorrupt spaces, reset the sets of visited and unvisited
    // coordinates, and re-attempt to visit all possible locations
    while !program_map.visited.contains_key(&program_map.end) {
        program_map.uncorrupt_next_space();
        program_map.reset_visited();
        program_map.visit_nodes();
    }

    // Once the end can be located again, find the next planned obstacle, which is the one that
    // would block the end
    let last_obstacle = program_map
        .planned_obstacles
        .last()
        .expect("Could not get last obstacle");
    let x = last_obstacle.x;
    let y = last_obstacle.y;
    let last_obstacle_str = format!("{x},{y}");
    println!("{last_obstacle_str}");
}
