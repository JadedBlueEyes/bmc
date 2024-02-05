use std::io::BufRead;

use crate::machine_code::MachineMemory;

pub fn read_memory_file(reader: Box<dyn BufRead>) -> MachineMemory {
    let mut memory: MachineMemory = [0; 256];
    for (number, line) in reader.lines().enumerate() {
        let value = line.unwrap_or_else(|_| panic!("Failed to read line {}", number + 1));
        let mut components = value.split_whitespace();
        if let (Some(loc), Some(value)) = (components.next(), components.next()) {
            let loc = u8::from_str_radix(&loc[0..=1], 16)
                .unwrap_or_else(|_| panic!("Invalid memory location on line {}", number + 1));

            for i in 0..value.len() / 2 {
                memory[loc as usize + i] = u8::from_str_radix(&value[(i * 2)..=(i * 2) + 1], 16)
                    .unwrap_or_else(|_| panic!("Invalid memory value on line {}", number + 1));
            }
        }
    }
    memory
}
