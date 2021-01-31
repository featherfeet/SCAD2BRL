extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs;
use std::env;
use std::vec::Vec;
use std::string::String;
use std::collections::HashMap;

use pest::Parser;
use pest::iterators::{ Pair, Pairs };

#[derive(Parser)]
#[grammar = "csg.pest"]
struct CSGParser;

#[derive(Debug)]
pub enum ASTNode {
    Cylinder {
        h: f64,
        r1: f64,
        r2: f64,
        center: bool
    },
    Difference {
        block: Box<ASTNode>
    },
    MultMatrix {
        matrix: Vec<Vec<f64>>,
        block: Box<ASTNode>
    },
    Block {
        statements: Vec<ASTNode>
    },
    Unknown
}

fn generate_block(pairs: Pairs<'_, Rule>) -> Option<ASTNode> {
    let mut statements = vec![];
    for pair in pairs {
        match generate_ast(pair) {
            Some(ast_node) => statements.push(ast_node),
            None => {}
        }
    }

    Some(ASTNode::Block { statements: statements })
}

fn generate_ast(pair: Pair<'_, Rule>) -> Option<ASTNode> {
    match pair.as_rule() {
        Rule::function_call => {
            // Get the Pair structures inside of the function_call node.
            let mut inner_pairs = pair.into_inner();
            // Get the name of the function being called.
            let identifier = inner_pairs.next().unwrap().as_span().as_str();
            // Get the next pair from the parse tree and use it to create an ASTNode.
            let next_pair = inner_pairs.next().unwrap();
            match next_pair.as_rule() {
                // If the next pair in the parse tree is an argument list, then the function is
                // either a cylinder or a multmatrix() call.
                Rule::argument_list => {
                    match identifier {
                        "cylinder" => {
                            let mut h: f64 = 1.0;
                            let mut r1: f64 = 1.0;
                            let mut r2: f64 = 1.0;
                            let mut center: bool = false;
                            for argument_pair in next_pair.into_inner() {
                                let mut argument = argument_pair.into_inner();
                                let argument_name = argument.next().unwrap().as_span().as_str();
                                let argument_value = argument.next().unwrap().as_span().as_str();
                                if argument_name == "h" {
                                    h = argument_value.parse().unwrap();
                                }
                                else if argument_name == "r1" {
                                    r1 = argument_value.parse().unwrap();
                                }
                                else if argument_name == "r2" {
                                    r2 = argument_value.parse().unwrap();
                                }
                                else if argument_name == "center" {
                                    center = argument_value.parse().unwrap();
                                }
                            }
                            Some(ASTNode::Cylinder { h: h, r1: r1, r2: r2, center: center })
                        }
                        "multmatrix" => {
                            let mut matrix = vec![];
                            let mut arguments_pairs = next_pair.into_inner();
                            let argument_pair = arguments_pairs.next().unwrap();
                            let matrix_pair = argument_pair.into_inner().next().unwrap();
                            let matrix_rows_pairs = matrix_pair.into_inner().next().unwrap().into_inner();
                            for matrix_row_pair in matrix_rows_pairs {
                                let mut matrix_row = vec![];
                                let matrix_numbers_pairs = matrix_row_pair.into_inner().next().unwrap().into_inner();
                                for matrix_number_pair in matrix_numbers_pairs {
                                    let matrix_number: f64 = matrix_number_pair.as_span().as_str().parse().unwrap();
                                    matrix_row.push(matrix_number);
                                }
                                matrix.push(matrix_row);
                            }
                            let block = generate_block(inner_pairs.next().unwrap().into_inner()).unwrap();
                            Some(ASTNode::MultMatrix { matrix: matrix, block: Box::new(block) })
                        }
                        _ => {
                            Some(ASTNode::Unknown { }) // TODO
                        }
                    }
                }
                // If the next pair in the parse tree is a block, then the function is a
                // difference() call.
                Rule::block => {
                    let block = generate_block(next_pair.into_inner()).unwrap();
                    match identifier {
                        "difference" => {
                            Some(ASTNode::Difference { block: Box::new(block) })
                        }
                        _ => {
                            Some(ASTNode::Unknown { }) // TODO
                        }
                    }
                }
                _ => {
                    Some(ASTNode::Unknown { }) // TODO
                }
            }
        }
        Rule::EOI => {
            None
        }
        _ => {
            Some(ASTNode::Unknown { }) // TODO
        }
    }
}

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

    // Parse the CSG file.
    let parsed_csg = CSGParser::parse(Rule::program, &raw_csg);
//    pest_ascii_tree::print_ascii_tree(parsed_csg.clone());
    let program = parsed_csg.expect("Failed to parse CSG!").next().expect("No program found.").into_inner();

    // Generate an AST from the CSG file.
    let mut ast = vec![];
    for pair in program {
        match generate_ast(pair) {
            Some(ast_node) => ast.push(ast_node),
            None => {}
        }
    }

    println!("{:#?}", ast);
}
