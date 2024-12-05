use std::{collections::HashSet, fs};

use clap::Parser;
use regex::Regex;

#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
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

fn transpose(input: &str) -> String {
    // Initialize an empty matrix to use for eventually building the string
    let mut output_matrix: Vec<Vec<char>> = Vec::new();

    // Get the number of rows and columns
    let input_row_count = input.lines().count();
    let input_col_count = input.lines().next().unwrap().len();

    // Initialize the matrix
    for _line in 0..input_col_count {
        output_matrix.push(vec!['\0'; input_row_count]);
    }

    // Create the transposed matrix element-wise
    for (row_index, line) in input.lines().filter(|x| !x.is_empty()).enumerate() {
        for (col_index, character) in line.as_bytes().iter().filter(|x| **x != b'\n').enumerate() {
            let row = output_matrix.get_mut(col_index).unwrap(); //.get_mut(col_index).unwrap();
            row[row_index] = *character as char;
        }
    }

    // Convert the transposed matrix into a string with newlines
    let mut output_string = String::new();
    for row in output_matrix {
        for character in row {
            output_string.push(character);
        }
        output_string.push('\n')
    }

    // Return the built string
    output_string
}

fn get_diagonal_representation(input: &str) -> String {
    // Initialize an empty matrix to use for eventually building the string
    let mut output_matrix: Vec<Vec<char>> = Vec::new();

    // Get the number of rows and columns, as well as the number of rows in the diagonal represenatation
    let input_row_count = input.lines().count();
    let input_col_count = input.lines().next().unwrap().len();
    let output_row_count = input_row_count + input_col_count - 1;

    // Add the inner lists for the matrix
    for _line in 0..output_row_count {
        output_matrix.push(Vec::new());
    }

    // Create the diagonal matrix by pushing elements, using the caclulated offset to create it
    for (offset, line) in input.lines().enumerate() {
        for (index, character) in line.chars().enumerate() {
            let insert_index = index + offset;
            output_matrix[insert_index].push(character);
        }
    }

    // Convert the constructed matrix into a string with newlines
    let mut output_string = String::new();
    for row in output_matrix {
        for character in row {
            output_string.push(character);
        }
        output_string.push('\n')
    }

    // Return the built string
    output_string
}

fn mirror(input: &str) -> String {
    // Create an empty string to manipulate
    let mut output = String::new();

    // Get the input string as a list of strings
    let mut input_matrix: Vec<&str> = input.lines().collect();

    // Reverse the list (flip over the horizontal center)
    input_matrix.reverse();

    // Build the string by pushing into the string, using newlines to seperate rows
    for line in input_matrix {
        output.push_str(line);
        output.push('\n');
    }

    // Return the constructed string
    output
}

fn check_for_xmas(x: &str) -> u64 {
    // Look-around not supported, so check for string in both orientations
    // Initialize the variable storing the number of matches
    let mut count = 0;

    // Count the number of XMAS matches
    let regex = Regex::new(r"XMAS").unwrap();
    count += regex.captures_iter(x).count();

    // Count the number of SAMX matches
    let regex = Regex::new(r"SAMX").unwrap();
    count += regex.captures_iter(x).count();

    // Return the number of matches
    count as u64
}

fn get_diagonal_characters(
    input_matrix: &[&str],
    row_index: usize,
    col_index: usize,
    forward_dir: bool,
) -> HashSet<char> {
    // Create a hash set for storing the found characters
    let mut slash_characters: HashSet<char> = HashSet::new();

    // Get the column modifier with checking for forward slashes or backslashes
    let col_modifier = if forward_dir { 1 } else { -1 };

    // Get the column indices for upper and lower character access
    let col_upper_index = (col_index as i32 + col_modifier) as usize;
    let col_lower_index = (col_index as i32 - col_modifier) as usize;

    // Get the upper and lower characters
    let upper_char = input_matrix[row_index + 1]
        .chars()
        .nth(col_upper_index)
        .unwrap();
    let lower_char = input_matrix[row_index - 1]
        .chars()
        .nth(col_lower_index)
        .unwrap();

    // Add the characters to the empty hashset
    slash_characters.insert(upper_char);
    slash_characters.insert(lower_char);

    // Return the hash set
    slash_characters
}

fn check_for_cross_mas(input_matrix: &[&str]) -> u64 {
    // Initialize the count of found X-MAS
    let mut count = 0;

    // Create a hash set representing the target hashset to compare against when 'A' is found
    let mut target_hashset = HashSet::new();
    target_hashset.insert('M');
    target_hashset.insert('S');

    // Iterate along the matrix row-by-row to look for matches
    for (row_index, line) in input_matrix.iter().enumerate() {
        // Skip the first and last rows, where matches will never be found (and avoid access issues)
        if row_index == 0 || row_index == (input_matrix.len() - 1) {
            continue;
        }

        // Iterate along the row character-by-character to look for matches
        for (col_index, character) in line.chars().enumerate() {
            // Skip the first and last column, where matches will never be found (and avoid access issues)
            if col_index == 0 || col_index == (line.len() - 1) {
                continue;
            }

            // If 'A' is found, check for 'M' and 'S'
            if character == 'A' {
                // Get the forward slash characters as a hashset
                let fslash_characters =
                    get_diagonal_characters(input_matrix, row_index, col_index, true);

                // Get the backslash characters as a hashset
                let bslash_characters =
                    get_diagonal_characters(input_matrix, row_index, col_index, false);

                // If the forward slash and backslash characters are both 'M' and 'S', add to the running tally
                if fslash_characters == target_hashset && bslash_characters == target_hashset {
                    count += 1;
                }
            }
        }
    }

    // Return the number of found matches
    count
}

fn main_part_one(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Initialize the count of matches
    let mut count = 0;

    // Check for horizontal matches
    count += check_for_xmas(&contents);

    // Check for vertical matches
    let transposed = transpose(&contents);
    count += check_for_xmas(&transposed);

    // Check for forward slash matches
    let fslash = get_diagonal_representation(&contents);
    count += check_for_xmas(&fslash);

    // Check for back slash matches
    let mirrored = mirror(&contents);
    let bslash = get_diagonal_representation(&mirrored);
    count += check_for_xmas(&bslash);

    // Print the result
    println!("{count}");
}

fn main_part_two(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Get the contents of the file as a list of strings
    let matrix: Vec<&str> = contents.lines().collect();

    // Get the number of X-MAS in the matrix
    let count = check_for_cross_mas(&matrix);

    // Print the result
    println!("{count}");
}
