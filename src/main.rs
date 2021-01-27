extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs;
use std::env;
use std::vec::Vec;
use std::string::String;
use std::collections::HashMap;

use pest::Parser;

#[derive(Parser)]
#[grammar = "csg.pest"]
struct CSGParser;

fn main() {
    // Read in the CSG file.
    let args: Vec<String> = env::args().collect();
    let csg_file_path = args.get(1).expect("No CSG file path provided!");
    let raw_csg = fs::read_to_string(csg_file_path).expect("Failed to read input CSG file.");

    // Get the name of the output MGED script file.
    let output_file_name: String = match args.get(2) {
        Some(arg) => arg.to_string(),
        None => "out.mged".to_string()
    };

    // Parse the assembly file.
    let parsed_csg = CSGParser::parse(Rule::program, &raw_csg);
    pest_ascii_tree::print_ascii_tree(parsed_csg);
//    let program = parsed_csg.expect("Failed to parse CSG!").next().expect("No program found.").into_inner();
}
