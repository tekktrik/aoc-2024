use std::fs;

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

fn find_multiplications(contents: String) -> u64 {
    // Initialize multiplication total
    let mut total: u64 = 0;

    // Create regex to parse the input string
    let re = Regex::new(r"mul\((\d{1,3}),(\d{1,3})\)").unwrap();

    // Iterate though the regex matches and multiple
    for (_, [factor_one_str, factor_two_str]) in re.captures_iter(&contents).map(|x| x.extract()) {
        let factor_one = factor_one_str.parse::<u64>().unwrap();
        let factor_two = factor_two_str.parse::<u64>().unwrap();
        total += factor_one * factor_two;
    }

    // Return the total
    total
}

fn create_instructioned_string(contents: String) -> String {
    // Initialize flagss for modifying the input string
    let mut delete_mode = false;
    let mut start_index: usize = 0;
    let mut deletions = Vec::new();

    // Create the regex for parsing the input string
    let re = Regex::new(r"(do(n't)?\(\))").unwrap();

    // Iterate through the regex matches
    for keyword_match in re.captures_iter(&contents) {
        // Matches the keyword found
        match keyword_match.get(1).unwrap().as_str() {
            "do()" => {
                // If in delete mode, turn off delete mode, get the final bounds of
                // deletion, and save it to the list
                if delete_mode {
                    delete_mode = false;
                    let end_index = keyword_match.get(1).unwrap().start();
                    deletions.push((start_index, end_index));
                }
            }
            "don't()" => {
                // If not in delete mode, turn off delete mode and save the start index
                // of the deletion for later use
                if !delete_mode {
                    delete_mode = true;
                    start_index = keyword_match.get(1).unwrap().start();
                }
            }
            _e => panic!("Found {_e} - something unexpected!"),
        }
    }

    // The list needs to be operated in reverse to preserve index values
    deletions.reverse();

    // Create a mutable string based on the input, and replace the saved index slices with nothing ("")
    let mut modified_contents = contents;
    for (start_index, end_index) in deletions {
        modified_contents.replace_range(start_index..end_index, "");
    }

    // Return the modified multiplication string
    modified_contents
}

fn main_part_one(filepath: String) {
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");
    let total = find_multiplications(contents);
    println!("The multiplication total is {total}");
}

fn main_part_two(filepath: String) {
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");
    let modified_contents = create_instructioned_string(contents);
    let total = find_multiplications(modified_contents);
    println!("The conditional multiplication total is {total}");
}
