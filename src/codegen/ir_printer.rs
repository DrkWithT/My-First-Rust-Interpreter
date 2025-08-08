use std::collections::{HashSet, VecDeque};

use crate::codegen::ir::*;

fn format_locator(loc: &Locator) -> String {
    let (loc_tag, loc_id) = loc;

    match *loc_tag {
        Region::Immediate => format!("const:{}", *loc_id),
        Region::TempStack => format!("temp_off:{}", *loc_id),
        Region::ArgStore => format!("args:{}", *loc_id),
        Region::ObjectHeap => format!("object:{}", *loc_id),
        Region::Functions => format!("function:{}", *loc_id),
        Region::BlockId => format!("block:{}", *loc_id),
    }
}

fn print_ir_node(node: &Node, id: i32) -> bool {
    let truthy_id = node.get_truthy_id().unwrap_or(-1);
    let falsy_id = node.get_falsy_id().unwrap_or(-1);

    println!("Block {id}:");
    println!("truthy-link: {truthy_id}, falsy-link: {falsy_id}\n");

    for step in node.get_steps() {
        match step {
            Instruction::Nonary(op) => {
                println!("{}", op.get_name());
            }
            Instruction::Unary(op, arg_0) => {
                println!("{} {}", op.get_name(), format_locator(arg_0));
            }
            Instruction::Binary(op, arg_0, arg_1) => {
                println!(
                    "{} {} {}",
                    op.get_name(),
                    format_locator(arg_0),
                    format_locator(arg_1)
                );
            }
        }
    }

    println!();

    truthy_id != -1 || falsy_id != -1
}

pub fn print_cfg(function_cfg: &CFG) {
    let mut done_ids = HashSet::<i32>::new();
    let mut next_nodes = VecDeque::<&Node>::new();
    next_nodes.push_back(function_cfg.get_root().unwrap());
    let mut next_id = 0;

    println!("\nIR:\n");

    while !next_nodes.is_empty() {
        let next_temp_opt = next_nodes.pop_front();

        if next_temp_opt.is_none() {
            continue;
        }

        let next_temp_ref = next_temp_opt.unwrap();

        if !print_ir_node(next_temp_ref, next_id) {
            break;
        }

        done_ids.insert(next_id);

        next_id += 1;

        let next_left_opt = function_cfg.get_left_neighbor(next_temp_ref);
        let temp_left_id_opt = next_temp_ref.get_truthy_id();

        if let Some(temp_left) = next_left_opt {
            let temp_left_id = if let Some(temp_id) = temp_left_id_opt {
                temp_id
            } else {
                -1
            };

            if temp_left_id != -1 && !done_ids.contains(&temp_left_id) {
                next_nodes.push_back(temp_left);
            }
        }

        let next_right_opt = function_cfg.get_right_neighbor(next_temp_ref);
        let temp_right_id_opt = next_temp_ref.get_falsy_id();

        if let Some(temp_right) = next_right_opt {
            let temp_right_id = if let Some(temp_id_2) = temp_right_id_opt {
                temp_id_2
            } else {
                -1
            };

            if temp_right_id != -1 && !done_ids.contains(&temp_right_id) {
                next_nodes.push_back(temp_right);
            }
        }
    }
}
