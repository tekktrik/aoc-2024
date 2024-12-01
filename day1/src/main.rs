use std::fs;

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

fn main() {
    // Parse CLI arguments
    let cli = CliArgs::parse();
    let file_contents = fs::read_to_string(cli.filepath).expect("Could not read file");

    // Run the code for the desired challenge part
    match cli.part {
        1 => main_part_one(file_contents),
        2 => main_part_two(file_contents),
        _ => panic!("Invalid selection part selection!"),
    }
}

// Function to create sorted lists of numbers based on the input text file
fn create_lists(contents: String) -> (Vec<u64>, Vec<u64>) {
    // Create empty, mutable lists
    let mut first_list: Vec<u64> = Vec::new();
    let mut second_list: Vec<u64> = Vec::new();

    // For each line in the supplied text, split the string and parse the number, and add to the list
    for line in contents.lines() {
        let mut numbers: Vec<u64> = line
            .split(" ")
            .filter(|x| !x.is_empty())
            .map(|y| str::parse::<u64>(y).unwrap())
            .collect();
        first_list.push(numbers.remove(0));
        second_list.push(numbers.remove(0));
    }

    // Sort the populated lists
    first_list.sort();
    second_list.sort();

    // Return both lists
    (first_list, second_list)
}

fn main_part_one(contents: String) {
    // Parse the file contents for the lists
    let (first_list, second_list) = create_lists(contents);

    // Initialize the different as 0
    let mut diff: u64 = 0;

    // For each pair of entries, get the absolute difference and add to the difference
    for (item_one, item_two) in first_list.iter().zip(second_list.iter()) {
        diff += item_one.abs_diff(*item_two);
    }

    // Print the difference
    println!("{diff}");
}

fn main_part_two(contents: String) {
    // Parse the file contents for the lists
    let (first_list, second_list) = create_lists(contents);

    // Initialize the different as 0
    let mut similarity: u64 = 0;

    // For each entry in the first list, get the number of times it's in list two,
    // and add the similarity score to the running total
    for entry in first_list {
        let entry_count = second_list.iter().filter(|x| **x == entry).count() as u64;
        similarity += entry_count * entry;
    }

    // Print the similarity score
    println!("{similarity}");
}
