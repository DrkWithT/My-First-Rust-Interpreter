use std::collections::VecDeque;

use crate::codegen::ir::*;
use crate::vm::value::Value;
use crate::vm::bytecode;

struct PatchEntry {
    pub instruction_pos: i32,
    pub patching_value: i32,
}

/// NOTE: compiles all procedures' CFGs into Chunks of bytecode.
#[derive(Default)]
pub struct BytecodeEmitter {
    /// NOTE: this is used as a stack of node locations to visit next during bytecode emission.
    pending_node_ids: VecDeque<i32>,

    /// NOTE: this is used as a stack of backpatching locations to fix jumps during bytecode emission.
    pending_patches: VecDeque<PatchEntry>,

    /// NOTE: stores temporary bytecode per CFG generated.
    temp_instructions: Vec<bytecode::Instruction>,
}

impl BytecodeEmitter {
    fn get_last_instruction_pos(&self) -> i32 {
        self.temp_instructions.len() as i32 - 1
    }

    fn start_backpatch(&mut self, patch: PatchEntry) {
        self.pending_patches.push_back(patch);
    }

    fn update_backpatch(&mut self) {
        let next_jump_location = self.get_last_instruction_pos();

        self.pending_patches.back_mut().unwrap().patching_value = next_jump_location;
    }

    fn apply_patches(&mut self) {
        while !self.pending_patches.is_empty() {
            let next_patch = self.pending_patches.pop_back().unwrap();

            let target_ref: &mut bytecode::Instruction = self.temp_instructions.get_mut(next_patch.instruction_pos as usize).unwrap();

            match target_ref {
                bytecode::Instruction::Jump(jump_target_loc) => {
                    *jump_target_loc = next_patch.patching_value;
                },
                bytecode::Instruction::JumpIf(_, jump_target_loc) => {
                    *jump_target_loc = next_patch.patching_value;
                },
                bytecode::Instruction::JumpElse(_, jump_target_loc) => {
                    *jump_target_loc = next_patch.patching_value;
                },
                _ => {},
            }
        }
    }

    fn emit_nonary_step_code(&mut self, ir_op: Opcode) -> bool {
        match ir_op {
            Opcode::Nop => {
                self.temp_instructions.push(bytecode::Instruction::Nop);
            },
            Opcode::Pop => {
                self.temp_instructions.push(bytecode::Instruction::Pop);
            },
            Opcode::Add => {
                self.temp_instructions.push(bytecode::Instruction::Add);
            },
            Opcode::Sub => {
                self.temp_instructions.push(bytecode::Instruction::Sub);
            },
            Opcode::Mul => {
                self.temp_instructions.push(bytecode::Instruction::Mul);
            },
            Opcode::Div => {
                self.temp_instructions.push(bytecode::Instruction::Div);
            },
            Opcode::CompareEq => {
                self.temp_instructions.push(bytecode::Instruction::CompareEq);
            },
            Opcode::CompareNe => {
                self.temp_instructions.push(bytecode::Instruction::CompareNe);
            },
            Opcode::CompareLt => {
                self.temp_instructions.push(bytecode::Instruction::CompareLt);
            },
            Opcode::CompareGt => {
                self.temp_instructions.push(bytecode::Instruction::CompareGt);
            },
            Opcode::GenPatch => {
                self.update_backpatch();
            },
            _ => {
                return false;
            },
        }

        true
    }

    fn emit_unary_step_code(&mut self, ir_op: Opcode, arg_0: i32) -> bool {
        match ir_op {
            Opcode::LoadConst => {
                self.temp_instructions.push(bytecode::Instruction::LoadConst(arg_0));
            },
            Opcode::Push => {
                self.temp_instructions.push(bytecode::Instruction::Push(arg_0));
            },
            Opcode::Neg => {
                self.temp_instructions.push(bytecode::Instruction::Neg(arg_0));
            },
            Opcode::Inc => {
                self.temp_instructions.push(bytecode::Instruction::Inc(arg_0));
            },
            Opcode::Dec => {
                self.temp_instructions.push(bytecode::Instruction::Dec(arg_0));
            },
            Opcode::Jump => {
                self.temp_instructions.push(bytecode::Instruction::Jump(arg_0));
                self.start_backpatch(PatchEntry {
                    instruction_pos: self.get_last_instruction_pos(),
                    patching_value: -1,
                });
            },
            Opcode::Return => {
                self.temp_instructions.push(bytecode::Instruction::Return(arg_0));
            },
            Opcode::Call => {
                self.temp_instructions.push(bytecode::Instruction::Call(arg_0));
            },
            _ => {
                return false;
            },
        }

        true
    }

    fn emit_binary_step_code(&mut self, ir_op: Opcode, arg_0: i32, arg_1: i32) -> bool {
        match ir_op {
            Opcode::Replace => {
                self.temp_instructions.push(bytecode::Instruction::Replace(arg_0, arg_1));
            },
            Opcode::JumpIf => {
                self.temp_instructions.push(bytecode::Instruction::JumpIf(arg_0, arg_1));
                self.start_backpatch(PatchEntry {
                    instruction_pos: self.get_last_instruction_pos(),
                    patching_value: -1,
                });
            },
            Opcode::JumpElse => {
                self.temp_instructions.push(bytecode::Instruction::JumpElse(arg_0, arg_1));
                self.start_backpatch(PatchEntry {
                    instruction_pos: self.get_last_instruction_pos(),
                    patching_value: -1,
                });
            },
            _ => {
                return false;
            },
        }

        true
    }

    fn emit_ir_block_code(&mut self, ir_block: &Node) -> (bool, i32, i32) {
        for temp_instr in ir_block.get_steps() {
            if !match temp_instr {
                Instruction::Nonary(op) => self.emit_nonary_step_code(*op),
                Instruction::Unary(op, arg_0) => self.emit_unary_step_code(*op, arg_0.1),
                Instruction::Binary(op, arg_0, arg_1) => self.emit_binary_step_code(*op, arg_0.1, arg_1.1),
            } {
                return (false, -1, -1);
            }
        }

        let truthy_block_id = ir_block.get_truthy_id().unwrap_or(-1);
        let falsy_block_id = ir_block.get_falsy_id().unwrap_or(-1);

        (true, truthy_block_id, falsy_block_id)
    }
    
    fn convert_ir_cfg_to_chunk(&mut self, temp_consts: &mut Vec<Value>, temp_cfg: &CFG) -> Option<bytecode::Chunk> {
        self.pending_node_ids.push_back(
            if temp_cfg.get_root().is_some() { 0 } else { -1 }
        );

        while !self.pending_node_ids.is_empty() {
            let next_node_id = self.pending_node_ids.pop_front().unwrap();

            if next_node_id == -1 {
                continue;
            }

            let (block_gen_ok, block_truthy_id, block_falsy_id) = self.emit_ir_block_code(
                temp_cfg.get_node_ref(next_node_id).unwrap()
            );

            if !block_gen_ok {
                return None;
            }

            if block_falsy_id != -1 {
                self.pending_node_ids.push_back(block_falsy_id);
            }

            if block_truthy_id != -1 {
                self.pending_node_ids.push_back(block_truthy_id);
            }

            self.apply_patches();
        }

        let mut temp_chunk_constants = Vec::<Value>::new();
        let mut temp_chunk_instructions = Vec::<bytecode::Instruction>::new();

        std::mem::swap(&mut temp_chunk_constants, temp_consts);
        std::mem::swap(&mut temp_chunk_instructions, &mut self.temp_instructions);

        Some(bytecode::Chunk::new(
            temp_chunk_constants,
            temp_chunk_instructions,
        ))
    }
    
    pub fn generate_bytecode(&mut self, cfg_list: &CFGStorage, temp_consts: &mut [Vec<Value>], main_fun_id: i32) -> Option<bytecode::Program> {
        let cfg_count = cfg_list.len() as i32;
        let mut temp_procedures = Vec::<bytecode::Procedure>::new();

        for cfg_id in 0..cfg_count {
            let temp_chunk = self.convert_ir_cfg_to_chunk(
                temp_consts.get_mut(cfg_id as usize).unwrap(),
                cfg_list.get(cfg_id as usize).unwrap(),
            );

            temp_chunk.as_ref()?;

            temp_procedures.push(bytecode::Procedure::new(
                temp_chunk.unwrap(),
                cfg_id,
            ));
        }

        Some(bytecode::Program::new(
            temp_procedures,
            main_fun_id,
        ))
    }
}
