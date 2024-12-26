use std::fs;

use clap::Parser;

use regex::Regex;

/// CLI arguments
#[derive(Parser)]
struct CliArgs {
    part: u64,
    filepath: String,
}

/// Type representing a literal operand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

struct LiteralOperand(u8);

impl LiteralOperand {
    /// Gets the value of the literal operand
    fn value(&self) -> u64 {
        self.0 as u64
    }
}

/// Type representing a combo operand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

struct ComboOperand(u8);

/// Type representing the operand when it is not needed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

struct UnusedOperand(u8);

/// Instructions that the computer can perfrom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Instruction {
    Adv(ComboOperand),
    Bxl(LiteralOperand),
    Bst(ComboOperand),
    Jnz(LiteralOperand),
    Bxc(UnusedOperand),
    Out(ComboOperand),
    Bdv(ComboOperand),
    Cdv(ComboOperand),
}

impl Instruction {
    /// Parses an opcode and operand into the associated instruction
    fn parse(opcode: u8, operand: u8) -> Instruction {
        let literal = LiteralOperand(operand);
        let combo = ComboOperand(operand);
        let unused = UnusedOperand(operand);
        match opcode {
            0 => Self::Adv(combo),
            1 => Self::Bxl(literal),
            2 => Self::Bst(combo),
            3 => Self::Jnz(literal),
            4 => Self::Bxc(unused),
            5 => Self::Out(combo),
            6 => Self::Bdv(combo),
            7 => Self::Cdv(combo),
            _o => panic!("Cound not parse opcode: {_o}"),
        }
    }

    /// Gets the instruction as the pair of integers it represents
    fn as_numbers(&self) -> (u8, u8) {
        match *self {
            Self::Adv(combo) => (0, combo.0),
            Self::Bxl(literal) => (1, literal.0),
            Self::Bst(combo) => (2, combo.0),
            Self::Jnz(literal) => (3, literal.0),
            Self::Bxc(unused) => (4, unused.0),
            Self::Out(combo) => (5, combo.0),
            Self::Bdv(combo) => (6, combo.0),
            Self::Cdv(combo) => (7, combo.0),
        }
    }
}

/// The computer that will execute the program
#[derive(Debug, Clone)]
struct Computer {
    /// Register A
    register_a: u64,
    /// Register B
    register_b: u64,
    /// Register C
    register_c: u64,
    /// List of instructions to execute
    instructions: Vec<Instruction>,
    /// Pointer that points to the index of the instruction to execute next
    pointer: usize,
    /// Running list of output numbers from the program
    output: Vec<u64>,
}

impl Computer {
    /// Creates a computer from the given string input
    fn from_string(text: &str) -> Self {
        // Create the regex patterns for the register portions of the text
        let register_a_re = Regex::new(r"Register A: (\d+)").unwrap();
        let register_b_re = Regex::new(r"Register B: (\d+)").unwrap();
        let register_c_re = Regex::new(r"Register C: (\d+)").unwrap();

        // Get the value of register A
        let register_a_captures = register_a_re
            .captures(text)
            .expect("Could not get match for Register A");
        let register_a = register_a_captures
            .get(1)
            .expect("Invalid capture group")
            .as_str()
            .parse::<u64>()
            .expect("Could not parse Register A data to u64");

        // Get the value of register B
        let register_b_captures = register_b_re
            .captures(text)
            .expect("Could not get match for Register B");
        let register_b = register_b_captures
            .get(1)
            .expect("Invalid capture group")
            .as_str()
            .parse::<u64>()
            .expect("Could not parse Register B data to u64");

        // Get the value of register C
        let register_c_captures = register_c_re
            .captures(text)
            .expect("Could not get match for Register C");
        let register_c = register_c_captures
            .get(1)
            .expect("Invalid capture group")
            .as_str()
            .parse::<u64>()
            .expect("Could not parse Register C data to u64");

        // Create the regex pattern for parsing instructions
        let instructions_re = Regex::new(r"(?:Program: )*(?: *)(\d+),(\d+)").unwrap();

        // Create a list for storing instructions
        let mut instructions = Vec::new();

        // Iterate through the captures for the instructions
        for (_, [opcode_str, operand_str]) in
            instructions_re.captures_iter(text).map(|c| c.extract())
        {
            // Convert the captures into the opcode and operand
            let opcode = opcode_str.parse::<u8>().expect("Could not parse opcode");
            let operand = operand_str.parse::<u8>().expect("Could not parse operand");

            // Parse the instruction
            let instruction = Instruction::parse(opcode, operand);

            // Add the instruction to the list
            instructions.push(instruction);
        }

        // Create and return the computer
        Self {
            register_a,
            register_b,
            register_c,
            instructions,
            pointer: 0,
            output: Vec::new(),
        }
    }

    /// Gets the value of the given combo operand
    fn get_combo_operand_value(&self, combo: &ComboOperand) -> u64 {
        match combo.0 {
            0..=3 => combo.0 as u64,
            4 => self.register_a,
            5 => self.register_b,
            6 => self.register_c,
            _ => panic!("Encounter unrecognized combo operand"),
        }
    }

    /// Runs the programs and returns the output string of numbers
    fn run_program(&mut self) -> String {
        while let Some(instruction) = self.fetch_instruction() {
            self.execute_instruction(&instruction);
        }
        self.create_output()
    }

    /// Runs a single cycle of the instructions and returns the output number for that cycle
    fn run_program_once(&mut self) -> u8 {
        // Reset the instruction pointer to the first instruction
        self.pointer = 0;

        // Reset the output list of numbers
        self.output = Vec::new();

        // Get the number of instructions
        let num_instructions = self.instructions.len();

        // Iterate through the instructions
        while let Some(instruction) = self.fetch_instruction() {
            // Execute the next instruction
            self.execute_instruction(&instruction);

            // If the pointer has jumped to the start or exceeded available instructions,
            // return the last number output
            if self.pointer == 0 || self.pointer == num_instructions {
                return *self.output.last().unwrap() as u8;
            }
        }

        // Something went wrong
        panic!("Could not get output number for this cycle");
    }

    /// Fetches the next instruction
    fn fetch_instruction(&self) -> Option<Instruction> {
        self.instructions.get(self.pointer).copied()
    }

    // Executes the given instruction
    fn execute_instruction(&mut self, instruction: &Instruction) {
        // Execute the instruction and determine whether the pointer should be incremented
        let advance_pointer = match instruction {
            Instruction::Adv(op) => self.perform_adv(op),
            Instruction::Bxl(op) => self.perform_bxl(op),
            Instruction::Bst(op) => self.perform_bst(op),
            Instruction::Jnz(op) => self.perform_jnz(op),
            Instruction::Bxc(_unused) => self.perform_bxc(),
            Instruction::Out(op) => self.perform_out(op),
            Instruction::Bdv(op) => self.perform_bdv(op),
            Instruction::Cdv(op) => self.perform_cdv(op),
        };

        // Increment the pointer if needed
        if advance_pointer {
            self.pointer += 1;
        }
    }

    /// Performs the ADV instruction
    fn perform_adv(&mut self, combo: &ComboOperand) -> bool {
        let numerator = self.register_a;
        let exp = self.get_combo_operand_value(combo) as u32;
        let result = numerator / 2_u64.pow(exp);
        self.register_a = result;
        true
    }

    /// Performs the BXL instruction
    fn perform_bxl(&mut self, literal: &LiteralOperand) -> bool {
        let x = self.register_b;
        let y = literal.value();
        let result = x ^ y;
        self.register_b = result;
        true
    }

    /// Performs the BST instruction
    fn perform_bst(&mut self, combo: &ComboOperand) -> bool {
        let x = self.get_combo_operand_value(combo);
        let result = x % 8;
        self.register_b = result;
        true
    }

    /// Performs the JNZ instruction
    fn perform_jnz(&mut self, literal: &LiteralOperand) -> bool {
        if self.register_a == 0 {
            return true;
        }

        let jump_location = literal.value();
        self.pointer = jump_location as usize;
        false
    }

    /// Performs the BXC instruction
    fn perform_bxc(&mut self) -> bool {
        let x = self.register_b;
        let y = self.register_c;
        let result = x ^ y;
        self.register_b = result;
        true
    }

    /// Performs the OUT instruction
    fn perform_out(&mut self, combo: &ComboOperand) -> bool {
        let value = self.get_combo_operand_value(combo);
        let result = value % 8;
        self.output.push(result);
        true
    }

    /// Performs the BDV instruction
    fn perform_bdv(&mut self, combo: &ComboOperand) -> bool {
        let numerator = self.register_a;
        let exp = self.get_combo_operand_value(combo) as u32;
        let result = numerator / 2_u64.pow(exp);
        self.register_b = result;
        true
    }

    /// Performs the CDV instruction
    fn perform_cdv(&mut self, combo: &ComboOperand) -> bool {
        let numerator = self.register_a;
        let exp = self.get_combo_operand_value(combo) as u32;
        let result = numerator / 2_u64.pow(exp);
        self.register_c = result;
        true
    }

    /// Creates a string of the output numbers separated with commas
    fn create_output(&self) -> String {
        let strings: Vec<String> = self.output.iter().map(|o| o.to_string()).collect();
        strings.join(",")
    }

    /// Finds the lowest value of Register A that creates an output of its own instructions
    fn find_self_outputing_register_a(&mut self) -> u64 {
        // Create the list of output numbers from the instructions
        let mut output = Vec::new();
        for instruction in &self.instructions {
            let (x, y) = instruction.as_numbers();
            output.push(x);
            output.push(y);
        }

        // Start with the register at the needed value of zero
        let register_a = 0;

        // Reverse engineer the value for Register A
        let result = self.reverse_engineer_register_a(register_a, &mut output);

        // Return the value of Register A
        result.1
    }

    /// Reverse engineers the value of Register A recursively as needed
    ///
    /// The input program uses only register A to calculate the values of Registers B and C,
    /// which in turn is what specifies the output number.  The value of Register A is then
    /// divided by 8, and the cycle repeats until Register A equals 0.  This method determines
    /// what the output number is for possible values of Register A that would create the
    /// current output, and recursively searches to make sure it can output all other values
    /// of the output, searching until a match is found.
    fn reverse_engineer_register_a(
        &mut self,
        register_a: u64,
        output: &mut Vec<u8>,
    ) -> (bool, u64) {
        // If there is no additional output to reverse engindeer, return the current value of Register A
        if output.is_empty() {
            return (true, register_a);
        }

        // Get the next value to reverse engineer
        let printout = output.pop().unwrap();

        // Get the bounds of the new Register A values to test
        let new_register_a_base = register_a * 8;
        let bound_register_a = new_register_a_base + 8;

        // Check the possible values for Register A
        for a in new_register_a_base..bound_register_a {
            // Set Register A to the test value
            self.register_a = a;

            // Get the out number for a single cycle of the program
            let printed = self.run_program_once();

            // If the output number matches the necessary number, recursively search for the
            // next number using the current value of Register A
            if printed == printout {
                let (finished, answer) = self.reverse_engineer_register_a(a, output);
                if finished {
                    return (true, answer);
                }
            }
        }

        // This search did not yield a possible result, return the printout number to the output list
        // and allow the caller to keep searching the previous printout number
        output.push(printout);
        (false, register_a)
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

    // Get the computer, initialized
    let mut computer = Computer::from_string(&contents);

    // Run the program
    let output = computer.run_program();

    // Output the readout from the program
    println!("{output}");
}

/// Runs part two
fn main_part_two(filepath: String) {
    // Get the trail ratings
    let contents = fs::read_to_string(filepath).expect("Invalid filepath");

    // Get the computer, initialized
    let mut computer = Computer::from_string(&contents);

    // Get the value of Register A for the self-outputting program
    let register_a = computer.find_self_outputing_register_a();
    println!("{register_a}");
}
