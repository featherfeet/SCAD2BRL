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
    Group {
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
                // difference() or group() call.
                Rule::block => {
                    let block = generate_block(next_pair.into_inner()).unwrap();
                    match identifier {
                        "difference" => {
                            Some(ASTNode::Difference { block: Box::new(block) })
                        }
                        "group" => {
                            Some(ASTNode::Group { block: Box::new(block) })
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

enum CSGOperation {
    Union,
    Difference,
    Intersection
}

fn generate_mged_code(solid_number: &mut i64, combination_number: &mut i64, csg_operation: CSGOperation, ast: Vec<ASTNode>) -> String {
    let mut mged_code = String::new();
    let mut child_transformation_code = String::new();
    let mut children: Vec<String> = vec![];

    let my_combination_number = *combination_number;
    
    for ast_node in ast {
        match ast_node {
            ASTNode::Cylinder { h, r1, r2, center } => {
                let z: f64 = if center {
                    -h / 2.0
                }
                else {
                    0.0
                };
                let name = format!("cylinder{}.s", solid_number);
                *solid_number += 1;
                mged_code += &format!("in {} trc 0 0 {} 0 0 {} {} {}\n", name, z, h, r1, r2);
                children.push(name);
            }
            ASTNode::Difference { block } => {
                match *block {
                    ASTNode::Block { statements } => {
                        *combination_number += 1;
                        let name = format!("comb{}.c", combination_number);
                        mged_code += &generate_mged_code(solid_number, combination_number, CSGOperation::Difference, statements);
                        children.push(name);
                    }
                    _ => {}
                }
            }
            ASTNode::MultMatrix { matrix, block } => {
                match *block {
                    ASTNode::Block { statements } => {
                        *combination_number += 1;
                        let name = format!("comb{}.c", combination_number);
                        mged_code += &generate_mged_code(solid_number, combination_number, CSGOperation::Union, statements);
                        children.push(name.clone());
                        let mut matrix_string = String::new();
                        for matrix_row in matrix {
                            for matrix_number in matrix_row {
                                matrix_string += &format!(" {}", matrix_number);
                            }
                        }
                        child_transformation_code += &format!("arced comb{}.c/{} matrix rmul{}\n", my_combination_number, name, matrix_string);
                    }
                    _ => {}
                }
            }
            ASTNode::Group { block } => {
                match *block {
                    ASTNode::Block { statements } => {
                        *combination_number += 1;
                        let name = format!("comb{}.c", combination_number);
                        mged_code += &generate_mged_code(solid_number, combination_number, CSGOperation::Union, statements);
                        children.push(name);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    mged_code += &format!("comb comb{}.c", my_combination_number);
    match csg_operation {
        CSGOperation::Union => {
            for child in children {
                mged_code += &format!(" u {}", child);
            }
        }
        CSGOperation::Difference => {
            for (i, child) in children.iter().enumerate() {
                if i == 0 {
                    mged_code += &format!(" u {}", child);
                }
                else {
                    mged_code += &format!(" - {}", child);
                }
            }
        }
        CSGOperation::Intersection => {
            println!("Intersections not supported yet.");
        }
    }
    mged_code += &"\n".to_string();

    mged_code += &child_transformation_code;

    mged_code
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
    //pest_ascii_tree::print_ascii_tree(parsed_csg.clone());
    let program = parsed_csg.expect("Failed to parse CSG!").next().expect("No program found.").into_inner();

    // Generate an AST from the CSG file.
    let mut ast = vec![];
    for pair in program {
        match generate_ast(pair) {
            Some(ast_node) => ast.push(ast_node),
            None => {}
        }
    }

    //println!("{:#?}", ast);
    
    // Generate MGED (BRL-CAD geometry editor) code from the AST.
    let mut solid_number: i64 = 0;
    let mut combination_number: i64 = 0;
    let mged_code = generate_mged_code(&mut solid_number, &mut combination_number, CSGOperation::Union, ast);
    print!("{}", mged_code);
}
