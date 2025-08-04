use std::collections::VecDeque;

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

fn print_ir_node(node: &Node) {
    for step in node.get_steps() {
        match step {
            Instruction::Nonary(op) => {
                println!("{}", op.get_name());
            },
            Instruction::Unary(op, arg_0) => {
                print!("{} {}", op.get_name(), format_locator(arg_0));
            },
            Instruction::Binary(op, arg_0, arg_1) => {
                print!("{} {} {}", op.get_name(), format_locator(arg_0), format_locator(arg_1));
            }
        }
    }
}

pub fn print_cfg(function_cfg: &CFG) {
    let mut next_nodes = VecDeque::<&Node>::new();
    next_nodes.push_back(function_cfg.get_root().unwrap());
    let mut next_id = 0;

    println!("IR:\n");

    while !next_nodes.is_empty() {
        let next_temp_opt = next_nodes.pop_front();

        if next_temp_opt.is_none() {
            continue;
        }

        println!("\nBlock {next_id}:\n");

        let next_temp_ref = next_temp_opt.unwrap();
        print_ir_node(next_temp_ref);
        next_id += 1;

        let next_left_opt = function_cfg.get_left_neighbor(next_temp_ref);

        if let Some(temp_left) = next_left_opt {
            next_nodes.push_back(temp_left);
        }

        let next_right_opt = function_cfg.get_right_neighbor(next_temp_ref);

        if let Some(temp_right) = next_right_opt {
            next_nodes.push_back(temp_right);
        }
    }
}
