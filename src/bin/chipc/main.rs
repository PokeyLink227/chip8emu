use crate::assembler::assemble;
use clap::Parser;
use std::{
    fs::File,
    io::{Read, Write},
};

mod assembler;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_name = "input file")]
    input_file: String,

    #[arg(short = 'o', long = "out", value_name = "output file", id = "out")]
    destination_file: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut input_file = match File::open(args.input_file) {
        Ok(f) => f,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let mut source: String = String::new();
    match input_file.read_to_string(&mut source) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
    drop(input_file);

    let output_file_name: String = if args.destination_file.is_some() {
        args.destination_file.unwrap()
    } else {
        "a.out".to_string()
    };
    let mut output_file = match File::create(output_file_name) {
        Ok(f) => f,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let bin = match assemble(&source) {
        Ok(b) => b,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };
    match output_file.write_all(&bin) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
}
