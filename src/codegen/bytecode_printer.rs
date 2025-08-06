use crate::vm::callable::Callable;
use crate::vm::bytecode::{Chunk, Instruction, Program};

fn disassemble_proc_chunk(chunk: &Chunk) {
    for item in chunk.get_code() {
        match item {
            Instruction::Nop => {
                println!("NOP");
            },
            Instruction::LoadConst(arg_0) => {
                println!("LOAD_CONST {}", *arg_0);
            },
            Instruction::Push(arg_0) => {
                println!("PUSH {}", *arg_0);
            },
            Instruction::Pop => {
                println!("POP");
            },
            Instruction::Replace(arg_0, arg_1) => {
                println!("REPLACE {} {}", *arg_0, *arg_1);
            },
            Instruction::Neg(arg_0) => {
                println!("NEG {}", *arg_0);
            },
            Instruction::Inc(arg_0) => {
                println!("INC {}", *arg_0);
            },
            Instruction::Dec(arg_0) => {
                println!("DEC {}", *arg_0);
            },
            Instruction::Add => {
                println!("ADD");
            },
            Instruction::Sub => {
                println!("SUB");
            },
            Instruction::Mul => {
                println!("MUL");
            },
            Instruction::Div => {
                println!("DIV");
            },
            Instruction::CompareEq => {
                println!("CMP_EQ");
            },
            Instruction::CompareNe => {
                println!("CMP_NE");
            },
            Instruction::CompareLt => {
                println!("CMP_LT");
            },
            Instruction::CompareGt => {
                println!("CMP_GT");
            },
            Instruction::JumpIf(arg_0, arg_1) => {
                println!("JMP_IF {} {}", *arg_0, *arg_1);
            },
            Instruction::JumpElse(arg_0, arg_1) => {
                println!("JMP_ELSE {} {}", *arg_0, *arg_1);
            },
            Instruction::Jump(arg_0) => {
                println!("JMP {}", *arg_0);
            },
            Instruction::Return(arg_0) => {
                println!("RETURN {}", *arg_0);
            },
            Instruction::Call(arg_0) => {
                println!("CALL {}", *arg_0);
            },
        }
    }
}

pub fn disassemble_program(program: &Program) {
    let main_proc_id = program.get_entry_procedure_id().unwrap_or(-1);

    for proc_entry in program.get_procedures() {
        let proc_id = proc_entry.get_id();
        
        if proc_id == main_proc_id {
            println!("proc (main):\n");
        } else {
            println!("proc #{proc_id}:\n");
        }

        disassemble_proc_chunk(proc_entry.get_chunk());
    }
}