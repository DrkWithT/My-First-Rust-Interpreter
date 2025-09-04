use std::collections::{HashSet, VecDeque};

use crate::codegen::ir::*;
use crate::vm::bytecode::{self, ArgMode};
use crate::vm::heap::HeapValue;
use crate::vm::value::Value;

struct PatchEntry {
    pub instruction_pos: i32,
    pub patching_value: i32,
    pub is_backward: bool,
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

fn convert_ir_arg_tag(arg: Region) -> ArgMode {
    match arg {
        Region::Immediate => ArgMode::ConstantId,
        Region::TempStack => ArgMode::StackOffset,
        Region::ObjectHeap => ArgMode::HeapId,
        Region::Field => ArgMode::InstanceFieldId,
        Region::Functions => ArgMode::ProcedureId,
        Region::Natives => ArgMode::NativeId,
        Region::BlockId => ArgMode::CodeOffset,
        _ => ArgMode::Dud,
    }
}

impl BytecodeEmitter {
    pub fn reset_state(&mut self) {
        self.pending_node_ids.clear();
        self.pending_patches.clear();
        self.temp_instructions.clear();
    }

    fn get_last_instruction_pos(&self) -> i32 {
        self.temp_instructions.len() as i32 - 1
    }

    fn start_backpatch(&mut self, patch: PatchEntry) {
        self.pending_patches.push_back(patch);
    }

    fn update_patch(&mut self) {
        let next_jump_location = self.get_last_instruction_pos();

        self.pending_patches.front_mut().unwrap().patching_value = next_jump_location;
    }

    fn update_backpatch(&mut self) {
        let back_jump_location = self.get_last_instruction_pos();

        self.pending_patches.front_mut().unwrap().instruction_pos = back_jump_location;
    }

    fn apply_patch(&mut self) {
        if !self.pending_patches.is_empty() {
            let is_backwards_patch = self.pending_patches.front().unwrap().is_backward;

            if is_backwards_patch {
                self.update_backpatch();
            } else {
                self.update_patch();
            }

            let next_patch = self.pending_patches.pop_front().unwrap();

            if next_patch.instruction_pos == -1 || next_patch.patching_value == -1 {
                self.pending_patches.push_front(next_patch);
                return;
            }

            let target_ref: &mut bytecode::Instruction = self
                .temp_instructions
                .get_mut(next_patch.instruction_pos as usize)
                .unwrap();

            if target_ref.is_valid_jump() {
                // println!(
                //     "Omitted duplicate patch... is_backward={}, patching_value={}, instruction_pos{}",
                //     next_patch.is_backward, next_patch.patching_value, next_patch.instruction_pos
                // );
                return;
            }

            match target_ref {
                bytecode::Instruction::Jump(jump_target_loc) => {
                    jump_target_loc.1 = next_patch.patching_value;
                },
                bytecode::Instruction::JumpIf(_, jump_target_loc) => {
                    jump_target_loc.1 = next_patch.patching_value;
                },
                bytecode::Instruction::JumpElse(_, jump_target_loc) => {
                    jump_target_loc.1 = next_patch.patching_value;
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
                self.temp_instructions
                    .push(bytecode::Instruction::CompareEq);
            },
            Opcode::CompareNe => {
                self.temp_instructions
                    .push(bytecode::Instruction::CompareNe);
            },
            Opcode::CompareLt => {
                self.temp_instructions
                    .push(bytecode::Instruction::CompareLt);
            },
            Opcode::CompareGt => {
                self.temp_instructions
                    .push(bytecode::Instruction::CompareGt);
            },
            Opcode::GenBeginLoop => {
                self.start_backpatch(PatchEntry {
                    instruction_pos: -1,
                    patching_value: self.get_last_instruction_pos(),
                    is_backward: true,
                });
            },
            Opcode::GenPatch => {
                self.apply_patch();
            },
            Opcode::GenPatchBack => {
                self.apply_patch();
            },
            Opcode::Leave => {
                self.temp_instructions
                    .push(bytecode::Instruction::Leave);
            },
            _ => {
                eprintln!("GenError: invalid nonary variant.");
                return false;
            },
        }

        true
    }

    fn emit_unary_step_code(&mut self, ir_op: Opcode, arg_0: Locator) -> bool {
        let converted_arg_0 = (convert_ir_arg_tag(arg_0.0), arg_0.1);

        match ir_op {
            Opcode::LoadConst => {
                self.temp_instructions
                    .push(bytecode::Instruction::LoadConst(converted_arg_0));
            },
            Opcode::Push => {
                self.temp_instructions
                    .push(bytecode::Instruction::Push(converted_arg_0));
            },
            Opcode::MakeHeapValue => {
                self.temp_instructions
                    .push(bytecode::Instruction::MakeHeapValue(converted_arg_0));
            },
            Opcode::MakeHeapObject => {
                self.temp_instructions
                    .push(bytecode::Instruction::MakeHeapObject(converted_arg_0));
            },
            Opcode::Neg => {
                self.temp_instructions
                    .push(bytecode::Instruction::Neg(converted_arg_0));
            },
            Opcode::Inc => {
                self.temp_instructions
                    .push(bytecode::Instruction::Inc(converted_arg_0));
            },
            Opcode::Dec => {
                self.temp_instructions
                    .push(bytecode::Instruction::Dec(converted_arg_0));
            },
            Opcode::Jump => {
                self.temp_instructions
                    .push(bytecode::Instruction::Jump(converted_arg_0));
                self.start_backpatch(PatchEntry {
                    instruction_pos: self.get_last_instruction_pos(),
                    patching_value: -1,
                    is_backward: false,
                });
            },
            Opcode::Return => {
                self.temp_instructions
                    .push(bytecode::Instruction::Return(converted_arg_0));
            },
            Opcode::NativeCall => {
                self.temp_instructions
                    .push(bytecode::Instruction::NativeCall(converted_arg_0));
            },
            _ => {
                eprintln!("Invalid unary variant.");
                return false;
            },
        }

        true
    }

    fn emit_binary_step_code(&mut self, ir_op: Opcode, arg_0: Locator, arg_1: Locator) -> bool {
        let converted_arg_0 = (convert_ir_arg_tag(arg_0.0), arg_0.1);
        let converted_arg_1 = (convert_ir_arg_tag(arg_1.0), arg_1.1);

        match ir_op {
            Opcode::Replace => {
                self.temp_instructions.push(bytecode::Instruction::Replace(
                    converted_arg_0,
                    converted_arg_1,
                ));
            },
            Opcode::JumpIf => {
                self.temp_instructions.push(bytecode::Instruction::JumpIf(
                    converted_arg_0,
                    converted_arg_1,
                ));
                self.start_backpatch(PatchEntry {
                    instruction_pos: self.get_last_instruction_pos(),
                    patching_value: -1,
                    is_backward: false,
                });
            },
            Opcode::JumpElse => {
                self.temp_instructions.push(bytecode::Instruction::JumpElse(
                    converted_arg_0,
                    converted_arg_1,
                ));
                self.start_backpatch(PatchEntry {
                    instruction_pos: self.get_last_instruction_pos(),
                    patching_value: -1,
                    is_backward: false,
                });
            },
            Opcode::Call => {
                self.temp_instructions.push(bytecode::Instruction::Call(
                    converted_arg_0,
                    converted_arg_1,
                ));
            },
            _ => {
                eprintln!("GenError: invalid binary variant.");
                return false;
            },
        }

        true
    }

    fn emit_ternary_step_code(&mut self, ir_op: Opcode, arg_0: Locator, arg_1: Locator, arg_2: Locator) -> bool {
        let converted_arg_0 = (convert_ir_arg_tag(arg_0.0), arg_0.1);
        let converted_arg_1 = (convert_ir_arg_tag(arg_1.0), arg_1.1);
        let converted_arg_2 = (convert_ir_arg_tag(arg_2.0), arg_2.1);

        match ir_op {
            Opcode::InstanceCall => {
                self.temp_instructions.push(bytecode::Instruction::InstanceCall(converted_arg_0, converted_arg_1, converted_arg_2));
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
                Instruction::Unary(op, arg_0) => self.emit_unary_step_code(*op, arg_0.clone()),
                Instruction::Binary(op, arg_0, arg_1) => {
                    self.emit_binary_step_code(*op, arg_0.clone(), arg_1.clone())
                },
                Instruction::Ternary(op, arg_0, arg_1, arg_2) => {
                    self.emit_ternary_step_code(*op, arg_0.clone(), arg_1.clone(), arg_2.clone())
                }
            } {
                eprintln!("GenError: Unknown instruction variant.");
                return (false, -1, -1);
            }
        }

        let truthy_block_id = ir_block.get_truthy_id().unwrap_or(-1);
        let falsy_block_id = ir_block.get_falsy_id().unwrap_or(-1);

        (true, truthy_block_id, falsy_block_id)
    }

    fn convert_ir_cfg_to_chunk(
        &mut self,
        temp_consts: &mut Vec<Value>,
        temp_cfg: &CFG,
    ) -> Option<bytecode::Chunk> {
        let mut done_ids = HashSet::<i32>::new();

        self.pending_node_ids
            .push_back(if temp_cfg.get_root().is_some() { 0 } else { -1 });

        while !self.pending_node_ids.is_empty() {
            let next_node_id = self.pending_node_ids.pop_back().unwrap();

            if next_node_id == -1 || done_ids.contains(&next_node_id) {
                continue;
            }

            let (block_gen_ok, block_truthy_id, block_falsy_id) =
                self.emit_ir_block_code(temp_cfg.get_node_ref(next_node_id).unwrap());
            done_ids.insert(next_node_id);

            if !block_gen_ok {
                eprintln!("GenError: failed to emit block.");
                return None;
            }

            if block_truthy_id != -1 && block_falsy_id == -1 {
                // A single-truthy-child block always implies a post-if block... Prioritize the child's generation to occur after the if/else blocks are done.
                self.pending_node_ids.push_front(block_truthy_id);
            } else {
                // Normal case: prioritize if/else branches to generate after the pre-branch block. The previous code handles the follow-up case.
                self.pending_node_ids.push_back(block_falsy_id);
                self.pending_node_ids.push_back(block_truthy_id);
            }
        }

        self.apply_patch();

        let mut temp_chunk_constants = Vec::<Value>::new();
        let mut temp_chunk_instructions = Vec::<bytecode::Instruction>::new();

        std::mem::swap(&mut temp_chunk_constants, temp_consts);
        std::mem::swap(&mut temp_chunk_instructions, &mut self.temp_instructions);

        Some(bytecode::Chunk::new(
            temp_chunk_constants,
            temp_chunk_instructions,
        ))
    }

    pub fn generate_bytecode(
        &mut self,
        cfg_list: &CFGStorage,
        temp_consts: &mut [Vec<Value>],
        main_fun_id: i32,
        temp_heap_preloadables: &mut Vec<HeapValue>
    ) -> Option<bytecode::Program> {
        let cfg_count = cfg_list.len() as i32;
        let mut temp_procedures = Vec::<bytecode::Procedure>::new();

        for cfg_id in 0..cfg_count {
            let temp_chunk = self.convert_ir_cfg_to_chunk(
                temp_consts.get_mut(cfg_id as usize).unwrap(),
                cfg_list.get(cfg_id as usize).unwrap(),
            );

            temp_chunk.as_ref()?;

            println!("loaded bytecode of proc-CFG #{cfg_id}");
            temp_procedures.push(bytecode::Procedure::new(temp_chunk.unwrap(), cfg_id));
        }

        let moved_preloadables = std::mem::take(temp_heap_preloadables);

        Some(bytecode::Program::new(temp_procedures, moved_preloadables, main_fun_id))
    }
}
