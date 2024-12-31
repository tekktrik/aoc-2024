use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs,
};

use clap::Parser;

/// Type representation of a single towel
type Towel = String;

/// Memory structure for storing a previous calculated number of ways
/// to create a given towel pattern
type PatternCache = HashMap<String, usize>;

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Towel pattern to be created
struct TowelPattern {
    pattern: String,
}

impl TowelPattern {
    /// Checks whether the given pattern is possible using the given array of towels
    fn is_pattern_possible_using(pattern: &str, towels: &[Towel]) -> bool {
        // If the pattern is empty, all previous parts have been created
        if pattern.is_empty() {
            return true;
        }

        // For each towel, check whether it can be used as the next towel, and recursively
        // checking the resulting pattern to see whether it can be created using the given
        // array of towels
        for towel in towels {
            if let Some(remaining_pattern) = pattern.strip_prefix(towel) {
                if Self::is_pattern_possible_using(remaining_pattern, towels) {
                    return true;
                }
            }
        }

        // The given towel pattern cannot be made using the given array of towels
        false
    }

    /// Checks whether this towel pattern is possible using the given array of towels
    fn is_possible_using(&self, towels: &[Towel]) -> bool {
        Self::is_pattern_possible_using(&self.pattern, towels)
    }

    /// Checks the number of ways the given towel pattern can be created using the given
    /// array of towels, and returns it in the given `count` variable, utilizing a given
    /// cache of previously created towel pattern results
    fn pattern_variations_using(
        pattern: &str,
        towels: &[Towel],
        count: &mut usize,
        pattern_cache: &mut PatternCache,
    ) {
        // If the pattern is empty, it represents a completed to create a towel pattern
        if pattern.is_empty() {
            *count += 1;
            return;
        }

        // If the towel pattern has been created previously, use the cached results
        if let Some(cached) = pattern_cache.get(pattern) {
            *count += *cached;
            return;
        }

        // Store the number of ways to create the overall pattern before creating this sub-pattern
        let initial_count = *count;

        // For each towel, check whether it can be used as the next towel, and recursively
        // checking the resulting pattern to see how many ways the remaining pattern can be
        // created using the given array of towels
        for towel in towels {
            if let Some(remaining_pattern) = pattern.strip_prefix(towel) {
                Self::pattern_variations_using(remaining_pattern, towels, count, pattern_cache);
            }
        }

        // Get the number of ways to create the overall pattern aftter creating this sub-pattern
        let new_count = *count;

        // Calculate the nubmer of ways to create the specific, given sub-pattern and insert it
        // into the cache memory
        let diff_count = new_count - initial_count;
        pattern_cache.insert(String::from(pattern), diff_count);
    }

    /// Calculates the number of ways to create the towel pattern using the given array of towels
    fn variations_using(&self, towels: &[Towel]) -> usize {
        // Create a variable for tracking the number of ways to create the towel pattern
        let mut count = 0;

        // Create a blank cache memory for sub-pattern results
        let mut pattern_cache = PatternCache::new();

        // Calculate the number of ways to create this towel pattern and return it
        Self::pattern_variations_using(&self.pattern, towels, &mut count, &mut pattern_cache);
        count
    }
}

impl Display for TowelPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern)
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

    // Get the set of towels and towel patterns
    let (towels, patterns) = parse(&contents);

    // Calculate the number of possible towel patterns
    let num_possible = patterns
        .iter()
        .filter(|p| p.is_possible_using(&towels))
        .count();
    println!("{num_possible}");
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the contents of the given filepath
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Get the set of towels and towel patterns
    let (towels, patterns) = parse(&contents);

    // Calculate the number of ways to create all possible towel patterns
    let mut total_count = 0;
    patterns
        .iter()
        .for_each(|p| total_count += p.variations_using(&towels));
    println!("{total_count}");
}

/// Parses the input text into the array of towels and towel patterns
fn parse(text: &str) -> (Vec<Towel>, Vec<TowelPattern>) {
    // Split the input text into the towel and towel patterns portions
    let texts: Vec<&str> = text.split("\n\n").collect();
    let towels_text = texts[0];
    let patterns_text = texts[1];

    // Get the array of towels from the towel portion
    let towels: Vec<Towel> = towels_text
        .split(",")
        .map(|t| String::from(t.trim()))
        .collect();

    // Create the array of towel patterns from the towel portion
    let mut patterns = Vec::new();
    for pattern_text in patterns_text.trim().lines() {
        let pattern = TowelPattern {
            pattern: String::from(pattern_text),
        };
        patterns.push(pattern);
    }

    // Return the towels and towel patterns
    (towels, patterns)
}
