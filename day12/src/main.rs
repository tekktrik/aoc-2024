use std::{
    collections::{HashMap, HashSet},
    fs,
};

use clap::Parser;

/// Type representing information about plots, which is a hash map
/// using plot labels for keys and a hash set of locations within
/// the plot as values
type PlotBreakdown = HashMap<String, HashSet<Location>>;

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Representation of the plot data
struct PlotData {
    area: u64,
    perimeter: u64,
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
    label: char,
}

/// Representation of the game map
struct GameMap {
    spaces: Vec<Vec<Location>>,
}

impl GameMap {
    // Creates a new map with the given spaces
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

                // Add the location (with label) to the row
                let plotspace = Location {
                    coord,
                    label: character,
                };
                row.push(plotspace);
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

    /// Gets the "cornerings" of a given coordindate, each of which is the list of the four
    /// coordinates surrounding a point on the grid, starting with the top-left and moving
    /// clockwise.  All four cornerings are returned for the given coordinate, starting
    /// with the top-left corner.
    fn cornerings(&self, coord: &Coordinate) -> Vec<Vec<Coordinate>> {
        // Create a list for storing cornerings
        let mut cornerings = Vec::new();

        // Shorthands for x and y
        let x = coord.x;
        let y = coord.y;

        // Get the coordinates at the cardinal directions
        let north = Coordinate::from((x, y + 1));
        let east = Coordinate::from((x + 1, y));
        let south = Coordinate::from((x, y - 1));
        let west = Coordinate::from((x - 1, y));

        // Get the coordinates at the diagonal directions
        let northwest = Coordinate::from((x - 1, y + 1));
        let northeast = Coordinate::from((x + 1, y + 1));
        let southeast = Coordinate::from((x + 1, y - 1));
        let southwest = Coordinate::from((x - 1, y - 1));

        // Create individual cornerings from the coordinates
        let topleft: Vec<Coordinate> = vec![northwest, north, *coord, west];
        let topright: Vec<Coordinate> = vec![north, northeast, east, *coord];
        let bottomright: Vec<Coordinate> = vec![*coord, east, southeast, south];
        let bottomleft: Vec<Coordinate> = vec![west, *coord, south, southwest];

        // Add the individual cornerings to the list
        cornerings.push(topleft);
        cornerings.push(topright);
        cornerings.push(bottomright);
        cornerings.push(bottomleft);

        // Return the list of cornerings
        cornerings
    }

    /// Converts cornering coordinates into cornering locations, where all non-valid,
    /// non-grouping locations are None
    fn convert_cornering(
        &self,
        label: &str,
        cornering: &Vec<Coordinate>,
        grouping: &HashSet<Location>,
    ) -> Vec<Option<&Location>> {
        // Create a list of cornering locations
        let mut plot_corners = Vec::new();

        // Iterate through the cornering coordiantes
        for space in cornering {
            // If the location is valid and within the grouping, add it to the list
            if let Some(location) = self.get(space) {
                if label.contains(location.label) && grouping.contains(location) {
                    plot_corners.push(Some(location));
                    continue;
                }
            }
            // Otherwise, push None to the list
            plot_corners.push(None);
        }

        // Return the list of cornering locations
        plot_corners
    }

    /// Finds groupings of plots recursively, and ultimately adding the grouping locations
    /// to the hash set provided.  The full set of locations that are a part of the grouping
    /// is returned.
    fn find_grouping(
        &self,
        label: &str,
        space: &Location,
        grouping: &mut HashSet<Location>,
    ) -> HashSet<Location> {
        // Create a list for storing the discovered locations
        let mut discovered_locations = HashSet::new();

        // If the provided space has a label that matches the providede one,
        // and the space is not already grouping, it should be added
        if label == space.label.to_string() && !grouping.contains(space) {
            // Add the space to the grouping and list of discovered locations
            grouping.insert(*space);
            discovered_locations.insert(*space);

            // Get the neighbors of the current space
            let neighbors = self.neighbors(&space.coord);

            // Recursively checking if the new space is part of the grouping
            for new_neighbor in neighbors {
                let other_locations = self.find_grouping(label, new_neighbor, grouping);
                discovered_locations.extend(other_locations);
            }
        }

        // Return the list of discovered locations in the grouping
        discovered_locations
    }

    // Get all of the trails, grouped by start location
    fn get_plots(&self) -> PlotBreakdown {
        // Create a hash map for grouped plots
        let mut plots = HashMap::new();

        // Create a hash set for keeping track of checked locations
        let mut checked_spaces = HashSet::new();

        // Create an enumeration variable for differentiating between plots with the same label
        let mut enumerator = 0;

        // Iterate through the map space by space
        for row in &self.spaces {
            for space in row {
                // If the space has already been checked, skip it
                if checked_spaces.contains(space) {
                    continue;
                }

                // Create a hash set for storing groupings
                let mut grouping = HashSet::new();

                // Get the grouping for the given space
                let explored = self.find_grouping(&space.label.to_string(), space, &mut grouping);

                // Add the spaces from the grouping to the list of checked spaces
                checked_spaces.extend(explored);

                // Create a unique identifier for the label
                let label = space.label;
                let key = format!("{label}{enumerator}");

                // Increment the enumerator to keep the next label unique
                enumerator += 1;

                // Add the grouped plots to the hash map
                plots.insert(key, grouping);
            }
        }

        // Return the completed hash map of grouped plots
        plots
    }

    /// Calculate the plot data from the given spaces
    fn calculate_plot_data(&self, spaces: &HashSet<Location>) -> PlotData {
        // Create variables for keeping track of the area and perimeter
        let mut area = 0;
        let mut perimeter = 0;

        // Create a copy of the grouped plot for later use
        let plot_spaces = spaces.clone();

        // Iterate through all of the grouped plot locations
        for space in spaces {
            // Get the neighboring locations as a hash set
            let neighbors: HashSet<Location> =
                HashSet::from_iter(self.neighbors(&space.coord).iter().copied().copied());

            // Get overlap between neighboring locations and locations in the grouping
            let overlap: HashSet<&Location> = neighbors.intersection(&plot_spaces).collect();

            // Increment the area by 1 (a location in the grouping is being operated on currently)
            area += 1;

            // Increment the perimeter by four minus the number of overlapping neighbor squares
            // (these squares mean there is no boundary on that side)
            perimeter += 4 - overlap.len() as u64;
        }

        // Return the area and perimenter plot data
        PlotData { area, perimeter }
    }

    /// Counts the number of corners within a grouping, which (nearly) corresponds
    /// to the number of sides for the plot
    fn count_corners(&self, grouping: &HashSet<Location>) -> u64 {
        // Create a variable for keeping track of the number of corners
        let mut num_corners = 0;

        // Create a list for keeping track of the analyzed cornerings
        let mut analyzed_corners: Vec<HashSet<Coordinate>> = Vec::new();

        // Iterate through the locations in the grouping
        for location in grouping {
            // Get the coordinates for the current location
            let current_coord = location.coord;

            // Get the label of the current location
            let location_label = &location.label.to_string();

            // Get the cornerings of the current coordinate
            let cornerings = self.cornerings(&current_coord);

            // Iterate through each of the individual cornerings
            for cornering in &cornerings {
                // Convert the list of cornerings into a hash map
                let coordinate_set = HashSet::from_iter(cornering.clone());

                // If the cornerings have already been analyzed, skip them
                if analyzed_corners.contains(&coordinate_set) {
                    continue;
                }

                // Add the unanalyzed cornering coordinates to the list of analyzed ones
                analyzed_corners.push(coordinate_set);

                // Covert the cornering coordinate information into cornering location information
                let plot_cornering = self.convert_cornering(location_label, cornering, grouping);

                // Analyze the number of corners for the given cornering locations, and add it to the running count
                num_corners += analyze_corners(plot_cornering);
            }
        }

        // Return the number of corners for the grouped plot
        num_corners
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

    // Get the plot breakdown
    let plots = map.get_plots();

    // Iterate and calculate fence prices
    let mut total_price = 0;
    for spaces in plots.values() {
        let plot_data = map.calculate_plot_data(spaces);
        let price = plot_data.area * plot_data.perimeter;
        total_price += price
    }

    // Print the total price
    println!("{total_price}")
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Create the game map from the file contents
    let map = GameMap::parse(&contents);

    // Get the plot breakdown
    let plots = map.get_plots();

    // Iterate and calculate fence prices
    let mut total_price = 0;
    for spaces in plots.values() {
        // Get the plot area
        let plot_data = map.calculate_plot_data(spaces);

        // Get the number of sides for the grouped plot
        let num_sides = map.count_corners(spaces);

        // Calculate the price of the fence and add it to the running total
        let price = plot_data.area * num_sides;
        total_price += price
    }

    // Print the total price
    println!("{total_price}")
}

/// Analyzes the given set of cornering location information to determine the number of corners
/// counted, which is identical to the number of sides (provided some points are counted twice)
fn analyze_corners(plot_cornering: Vec<Option<&Location>>) -> u64 {
    // If all the locations are not from the plot, it is a bad input
    if plot_cornering.iter().all(|x| x.is_none()) {
        panic!("No locations from which to analyze corners!")
    }

    // If all the spaces in the cornering coordinates are valid locations, there is no corner
    if plot_cornering.iter().all(|x| x.is_some()) {
        return 0;
    }

    // Breakout the locations from the cornering squares
    let topleft = plot_cornering[0];
    let topright = plot_cornering[1];
    let bottomright = plot_cornering[2];
    let bottomleft = plot_cornering[3];

    // Get the conditionals for diagonal plot squares
    let diagonal_a =
        topleft.is_some() && bottomright.is_some() && topright.is_none() && bottomleft.is_none();
    let diagonal_b =
        topleft.is_none() && bottomright.is_none() && topright.is_some() && bottomleft.is_some();

    // Get the conditional for single-line plot squares
    let top =
        topleft.is_some() && topright.is_some() && bottomleft.is_none() && bottomright.is_none();
    let bottom =
        topleft.is_none() && topright.is_none() && bottomleft.is_some() && bottomright.is_some();
    let left =
        topleft.is_some() && bottomleft.is_some() && topright.is_none() && bottomright.is_none();
    let right =
        topleft.is_none() && bottomleft.is_none() && topright.is_some() && bottomright.is_some();

    // If diagonal squares are identified, count the corner twice (once for each use of it)
    if diagonal_a || diagonal_b {
        return 2;
    }

    // If single-line squares are identified, there is no corner
    if top || bottom || left || right {
        return 0;
    }

    // The remaining cases are where there are one or three plot locaions, which both
    // indicate in a single corner
    1
}
