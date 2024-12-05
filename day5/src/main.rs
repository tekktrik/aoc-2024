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

/// Generate the rules for the page ordering, specifically rules that
/// indicate an ordering is NOT in the correct order.  That means for
/// any given page, a set of other pages is available that CANNOT come
/// before it.
fn generate_rules(rules_text: &str) -> HashMap<u16, HashSet<u16>> {
    // Create an empty hash map to populate with rules
    let mut all_rule_breaks: HashMap<u16, HashSet<u16>> = HashMap::new();

    // Iterate through the rules text line by line
    for line in rules_text.lines().map(|x| x.trim()) {
        // Get the pages in the rule
        let page_split: Vec<&str> = line.split("|").collect();
        let leading_page = page_split[0].parse::<u16>().unwrap();
        let following_page = page_split[1].parse::<u16>().unwrap();

        // Either create a new set of pages for each entry (page), or add to
        // the existing one
        match all_rule_breaks.get_mut(&following_page) {
            Some(entry) => {
                entry.insert(leading_page);
            }
            None => {
                let mut leading_page_set = HashSet::new();
                leading_page_set.insert(leading_page);
                all_rule_breaks.insert(following_page, leading_page_set);
            }
        }
    }

    // Return the rules
    all_rule_breaks
}

/// Generate the list of updates from the provided text
fn generate_updates(updates_text: &str) -> Vec<Vec<u16>> {
    // Create a new vector to populate
    let mut all_updates = Vec::new();

    // For each line in the text:
    // 1. Split by commas
    // 2. Parse each number into a u16
    // 3. Collect the list into a vector
    // 4. Push it to the previously created vector
    for line in updates_text.lines().map(|x| x.trim()) {
        all_updates.push(line.split(",").map(|x| x.parse::<u16>().unwrap()).collect());
    }

    // Return the vector
    all_updates
}

// Convenience function for creating both the rules and updates from the provided text
fn generate_rules_and_updates(input: &str) -> (HashMap<u16, HashSet<u16>>, Vec<Vec<u16>>) {
    // Split the text by the double newline
    let input_split: Vec<&str> = input.split("\n\n").collect();
    let rules_str = input_split[0];
    let updates_str = input_split[1];

    // Get the rules and updates from their respective parts
    let rules = generate_rules(rules_str);
    let updates = generate_updates(updates_str);

    // Return both the rules and updates
    (rules, updates)
}

/// Checks an update if any rules (rule breaks) apply
fn check_for_rule_break(update: &[u16], rules: &HashMap<u16, HashSet<u16>>) -> bool {
    // Iterate through the update page by page
    for (index, page) in update.iter().enumerate() {
        // Create a hash set from the remaining pages after it in the update
        let following_pages: HashSet<u16> = HashSet::from_iter(update[index..].iter().cloned());

        // Get the applicable rule breaks for the given pages
        match rules.get(page) {
            // There are rules that must be checked for this page
            Some(forbidden_following_pages) => {
                // Get the intersection of rules that indicate bad ordering and remaining pages
                // in the ordering
                let all_found_forbidden_pages: HashSet<&u16> = following_pages
                    .intersection(forbidden_following_pages)
                    .collect();

                // If the intersection is not empty, rules have be broken, so early return true
                // to the caller
                if !all_found_forbidden_pages.is_empty() {
                    return true;
                }
            }
            // No rules can be broken, so check the next page in the update
            None => continue,
        }
    }

    // No rules were ever broken, return false
    false
}

/// Calculates the order score for a given page
///
/// This is intended to be used for comparing pages for ordering.
fn calculate_order_score(
    page: &u16,
    update: &Vec<u16>,
    rules: &HashMap<u16, HashSet<u16>>,
    previously_calculated: &mut HashMap<u16, u64>,
) -> u64 {
    // Checking for scores recursively for large lists can take a long time,
    // so if a score is previously calculated and stored, we can skip recursively
    // calculating it again
    if previously_calculated.contains_key(page) {
        return *previously_calculated.get(page).unwrap();
    }

    // Initialize the default score at 1
    let mut score = 1;

    // Get the remaining pages to check (other than the current one)
    let mut remaining_pages = HashSet::from_iter(update.iter().copied());
    remaining_pages.remove(page);

    // Caclculate the order score for the page
    if let Some(possible_rules) = rules.get(page) {
        // Get the intersection of pages that have ordering rules and remaining pages,
        // and recursively add their own scores to this one
        let applicable_rules: HashSet<u16> = possible_rules
            .intersection(&remaining_pages)
            .copied()
            .collect();
        for applicable_rule in applicable_rules {
            score += calculate_order_score(&applicable_rule, update, rules, previously_calculated)
        }
    }

    // Store the caclulated score for the page for faster calculations for the rest of the pages
    previously_calculated.insert(*page, score);

    // Return the order score
    score
}

/// Gets the list of middle pages for INCORRECTLY ordered pages
///
/// This functions finds incorrect orderings, sorts them, and returns middle pages for them
/// as a vector.
fn get_incorrectly_ordered_middles(
    updates: &Vec<Vec<u16>>,
    rules: &HashMap<u16, HashSet<u16>>,
) -> Vec<u16> {
    // Create a new list of middle pages to populate
    let mut middle_pages = Vec::new();

    // Iterate through the page updates
    for update in updates {
        // Create a new hash map for storing previously discovered score of given pages,
        // which greatly improves the speed at which the sorting later on takes
        let mut saved_scores: HashMap<u16, u64> = HashMap::new();

        // If the update follows all the rules, skip it
        if !check_for_rule_break(update, rules) {
            continue;
        }

        // Create a clone of the update
        let mut ordered_update = update.clone();

        // Sort the cloned copy of the update
        ordered_update.sort_by(|x, y| {
            calculate_order_score(x, update, rules, &mut saved_scores).cmp(&calculate_order_score(
                y,
                update,
                rules,
                &mut saved_scores,
            ))
        });

        // Get the middle page of the newly sorted vector
        let update_length = ordered_update.len();
        let middle_page_index = (update_length - 1) / 2;
        let middle_page = ordered_update[middle_page_index];

        // Add the middle page to the vector
        middle_pages.push(middle_page);
    }

    // Return all of the middle pages
    middle_pages
}

/// Gets the list of middle pages for CORRECTLY ordered pages
///
/// This functions finds correct orderings and returns middle pages for them
/// as a vector.
fn get_correctly_ordered_middles(
    updates: &Vec<Vec<u16>>,
    rules: &HashMap<u16, HashSet<u16>>,
) -> Vec<u16> {
    // Create a new list of middle pages to populate
    let mut middle_pages = Vec::new();

    // Iterate through the page updates
    for update_pages in updates {
        // If the update doesn;t follow all the rules, skip it
        if check_for_rule_break(update_pages, rules) {
            continue;
        }

        // Update is valid, so get the middle page
        let update_length = update_pages.len();
        let middle_page_index = (update_length - 1) / 2;
        let middle_page = update_pages[middle_page_index];

        // Add the middle page to the vector
        middle_pages.push(middle_page);
    }

    // Return all of the middle pages
    middle_pages
}

fn main_part_one(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Get the rules and updates
    let (rules, updates) = generate_rules_and_updates(&contents);

    // Get the middle pages of correctly ordered updates
    let valid_middle_pages: Vec<u16> = get_correctly_ordered_middles(&updates, &rules);

    // Get the sum of the middle pages
    let sum_valid_middle_pages: u64 = valid_middle_pages.iter().map(|x| *x as u64).sum();

    // Announce the sum
    println!("{sum_valid_middle_pages}");
}

fn main_part_two(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Get the rules and updates
    let (rules, updates) = generate_rules_and_updates(&contents);

    // Get the intended middle pages of incorrectly ordered updates
    let reordered_middle_pages = get_incorrectly_ordered_middles(&updates, &rules);

    // Get the sum of the middle pages
    let sum_reordered_middle_pages: u64 = reordered_middle_pages.iter().map(|x| *x as u64).sum();

    // Announce the sum
    println!("{sum_reordered_middle_pages}");
}
