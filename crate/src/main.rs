use bmc::execute;
use bmc::highlight::highlight;
use bmc::machine_code::Ctx;
use bmc::memory::read_memory_file;
use clap::{Parser, Subcommand};
// use std::alloc::System;
// use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Executes the given machine code
    Execute {
        /// Indicates that the input is a file path
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Highlights the given assembly
    Highlight {
        /// Indicates that the input is a file path
        #[arg(short, long)]
        file: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Execute { file } => {
            let reader: Box<dyn BufRead> = match file {
                Some(file_path) => {
                    let path = std::path::PathBuf::from(file_path);
                    let f = File::open(path).expect("File not found");
                    Box::new(BufReader::new(f))
                }
                None => Box::new(BufReader::new(io::stdin())),
            };

            let memory = read_memory_file(reader);
            println!("{:?}", memory);
            let mut ctx: Ctx = Ctx {
                executing: true,
                memory,
                pc: 0,
                registers: [0; 16],
            };
            let mut burned = 0;
            const STEP: usize = 256;
            while ctx.executing {
                burned += STEP - execute(&mut ctx, STEP);
            }

            println!("Used {} instruction cycles", burned);
            println!("MEMORY: \n{:?}", ctx.memory);
            println!("REGISTERS: \n{:?}", ctx.registers);
        },
        Commands::Highlight { file } => {
            let mut reader: Box<dyn BufRead> = match file {
                Some(file_path) => {
                    let path = std::path::PathBuf::from(file_path);
                    let f = File::open(path).expect("File not found");
                    Box::new(BufReader::new(f))
                }
                None => Box::new(BufReader::new(io::stdin())),
            };
            let mut source = String::new();
            
            let mut w: Vec<u8> = Vec::new();
            let _ = reader.read_to_string(&mut source);
            
            highlight(&source, &mut w);
            let _ = io::stdout().write(&w);
        }
    };
}

