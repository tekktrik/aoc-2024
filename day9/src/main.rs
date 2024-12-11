use std::fs;

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Representation of a contiguous block of memory
#[derive(Clone, Copy, Debug)]
struct MemoryBlock {
    /// The ID of the memory block
    ///
    /// This is Some(id) if it is data, and None if it is empty space
    id: Option<usize>,

    /// The size/length of the memory block
    size: usize,
}

impl MemoryBlock {
    /// Reduces the size of the memory block by a specified amount
    ///
    /// Returns whether the memory was reduced.
    fn reduce(&mut self, amount: usize) -> bool {
        if amount > self.size {
            return false;
        }
        self.size -= amount;
        true
    }

    /// Creates an empty memory block of a certain size
    fn as_empty(size: usize) -> Self {
        Self { id: None, size }
    }

    /// Checks whether the memory block is considered free
    fn is_free(&self) -> bool {
        self.id.is_none()
    }

    /// Gets the memory block as a vector of bytes
    fn as_byte_list(&self) -> Vec<Option<usize>> {
        vec![self.id; self.size]
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
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the input
    let mut data = create_byte_list(&contents);

    // Defragment the data
    defragment_data_bytewise(&mut data);

    // Caclulate and print the checksum
    let checksum = calculate_checksum(&data);
    println!("{checksum}");
}

/// Runs part one
fn main_part_two(filepath: String) {
    // Read the contents of the file
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Parse the input
    let mut blocks = create_block_list(&contents);

    // Defragment the data
    defragment_data_blockwise(&mut blocks);

    // Create the newly defragmented data in bytes format
    let mut data = Vec::new();
    blocks.iter().for_each(|x| data.extend(x.as_byte_list()));

    // Caclulate and print the checksum
    let checksum = calculate_checksum(&data);
    println!("{checksum}");
}

/// Creates a list of numbers based on the input string
///
/// IDs are placed in their respective locations, with None being
/// used to signify empty spaces.
fn create_byte_list(input: &str) -> Vec<Option<usize>> {
    // Create a new list for storing byte data
    let mut data = Vec::new();

    // Keep track of the mode (data or empty)
    let mut data_mode = true;
    let mut id = 0;

    // Iterate through the characters of the input
    for character in input.trim().chars() {
        // Get the number of spaces by reading and parsing the digit
        let num_spaces = character.to_digit(10).expect("Could not parse the digit");

        // Perform the action for the number of spaces
        for _x in 0..num_spaces {
            // If data mode, push IDs to the list; otherwise push empty space
            if data_mode {
                data.push(Some(id));
            } else {
                data.push(None);
            }
        }

        // If just performed the action in data mode, increase the ID by 1
        if data_mode {
            id += 1;
        }

        // Switch between data and empty mode
        data_mode = !data_mode
    }

    // Return the data list
    data
}

/// Creates a list of memory blocks based on the input string
fn create_block_list(input: &str) -> Vec<MemoryBlock> {
    // Create a new list for storing byte data
    let mut data = Vec::new();

    // Keep track of the mode (data or empty)
    let mut data_mode = true;
    let mut id = 0;

    // Iterate through the characters of the input
    for character in input.trim().chars() {
        // Get the number of spaces by reading and parsing the digit
        let num_spaces = character.to_digit(10).expect("Could not parse the digit");

        // Prepare the ID of the memory block depending on whether it is data or empty space
        let assignable_id = if data_mode { Some(id) } else { None };

        // Create and add the memory block to the list
        let memory_block = MemoryBlock {
            id: assignable_id,
            size: num_spaces as usize,
        };
        data.push(memory_block);

        // If just performed the action in data mode, increase the ID by 1
        if data_mode {
            id += 1;
        }

        // Switch between data and empty mode
        data_mode = !data_mode
    }

    // Return the data list
    data
}

/// Defragment the data at the "byte" level
fn defragment_data_bytewise(data: &mut Vec<Option<usize>>) {
    // While empty space is still detected in the data...
    while data.contains(&None) {
        // Remove trailing empty space
        while data.last().expect("Data vector is empty").is_none() {
            data.pop();
        }

        // If an additional empty space is detected, move the last byte into it's location
        if let Some(pos) = data.iter().position(|x| x.is_none()) {
            let last = data.pop().expect("Vector is empty!").unwrap();
            data[pos] = Some(last)
        }
    }
}

/// Defragment the data at the "memory block" level
fn defragment_data_blockwise(data: &mut Vec<MemoryBlock>) {
    // Get the number of IDs to be iterated over
    let num_id = data.iter().filter_map(|x| x.id).count();

    // Iterate through the IDs in descending order
    for id in (0..num_id).rev() {
        // Get the files original position and remove it from the list
        let file_position = data.iter().position(|x| x.id == Some(id)).unwrap();
        let file_memory = data.remove(file_position);

        // Look for a free memory block of at least the same size ahead of the original position
        if let Some(free_position) = data
            .iter()
            .enumerate()
            .position(|(i, x)| x.is_free() && x.size >= file_memory.size && i < file_position)
        {
            // Get the free memory and reduce it's size by the file size
            let free_memory = data.get_mut(free_position).unwrap();
            free_memory.reduce(file_memory.size);

            // Insert the file ahead of the free memory
            data.insert(free_position, file_memory);

            // Insert an empty memory block the same size as the file memory at the file's original position
            data.insert(file_position, MemoryBlock::as_empty(file_memory.size));
        } else {
            // No free space was found, re-insert the file in it's original postion
            data.insert(file_position, file_memory);
        }
    }

    data.retain_mut(|x| x.size > 0)
}

/// Calculate the checksum for an array of data bytes
fn calculate_checksum(data: &[Option<usize>]) -> usize {
    let mut checksum = 0;
    for (index, id) in data.iter().enumerate() {
        if let Some(value) = id {
            checksum += index * value
        }
    }
    checksum
}
