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

    // Run the code for the desired challenge part
    match cli.part {
        1 => main_part_one(cli.filepath),
        2 => main_part_two(cli.filepath),
        _ => panic!("Invalid selection part selection!"),
    }
}

// Function to create sorted lists of numbers based on the input text file
fn create_list(filepath: String) -> Vec<Vec<u64>> {
    let contents = fs::read_to_string(filepath).expect("Could not read file");

    // Create empty, mutable lists
    let mut data: Vec<Vec<u64>> = Vec::new();

    // For each line in the supplied text, split the string and parse the number, and add to the list
    for line in contents.lines() {
        let report: Vec<u64> = line
            .split(" ")
            .filter(|x| !x.is_empty())
            .map(|y| str::parse::<u64>(y).unwrap())
            .collect();
        data.push(report.clone());
    }

    // Return the list
    data
}

fn is_safe_report(report: &Vec<u64>) -> bool {
    // Iterate through entries and see if change is within the safe margin
    let mut previous_entry = *report.first().unwrap();
    let mut first_entry = true;
    for entry in report {
        let entry_diff = entry.abs_diff(previous_entry);
        if entry_diff > 3 || (entry_diff == 0 && !first_entry) {
            return false;
        }
        previous_entry = *entry;
        first_entry = false;
    }
    true
}

fn is_list_sorted(report: &Vec<u64>) -> bool {
    // Create a clone of an ascending sorted report
    let mut asc_sorted_report = report.clone();
    asc_sorted_report.sort();

    // Create a clone of a descending sorted report
    let mut desc_sorted_report = asc_sorted_report.clone();
    desc_sorted_report.reverse();

    // Immediately disqualify report for safety if array not sorted
    if *report == asc_sorted_report || *report == desc_sorted_report {
        return true;
    }

    false
}

fn main_part_one(filepath: String) {
    // Parse the file contents for the lists
    let data = create_list(filepath);

    // Initialize the number of safe reports as 0
    let mut safe_report_count: u64 = 0;

    // Iterate through all reports in the data
    for report in data {
        // Check whether the list is sorted
        if !is_list_sorted(&report) {
            continue;
        }

        // If the current report is not safe, check the next one
        if !is_safe_report(&report) {
            continue;
        }

        // Add to the number of safe reports found
        safe_report_count += 1;
    }

    // Print the number of safe reports
    println!("{safe_report_count}");
}

fn main_part_two(filepath: String) {
    // Parse the file contents for the lists
    let data = create_list(filepath);

    // Initialize the number of safe reports as 0
    let mut safe_report_count: u64 = 0;

    // Iterate through all reports in the data
    'report_check: for report in data {
        // Iterate through report, removing each entry until a safe report is detected
        'removal_check: for index in 0..report.len() {
            // Create a report with an single point removed
            // Already safe reports will still pass regardless!
            let mut modified_report = report.clone();
            modified_report.remove(index);

            // Check whether the modified list is sorted, try to remove
            // a different entry if it's not.
            if !is_list_sorted(&modified_report) {
                continue 'removal_check;
            }

            // Check whether the modified report is safe
            if is_safe_report(&modified_report) {
                safe_report_count += 1;
                continue 'report_check;
            }
        }
    }

    // Print the number of safe reports
    println!("{safe_report_count}");
}
