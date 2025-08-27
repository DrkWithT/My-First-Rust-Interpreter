use crate::vm::bytecode::{ArgMode, Chunk, Instruction, Program};

fn disassemble_op_arg(arg: &(ArgMode, i32)) {
    let (arg_pass_mode, arg_value) = arg;
    print!("{}:{} ", arg_pass_mode.get_name(), arg_value);
}

fn disassemble_proc_chunk(chunk: &Chunk) {
    for item in chunk.get_code() {
        match item {
            Instruction::Nop => {
                println!("NOP");
            },
            Instruction::LoadConst(arg_0) => {
                print!("LOAD_CONST ");
                disassemble_op_arg(arg_0);
                println!();
            },
            Instruction::Push(arg_0) => {
                print!("PUSH ");
                disassemble_op_arg(arg_0);
                println!();
            },
            Instruction::Pop => {
                println!("POP");
            },
            Instruction::MakeHeapValue(arg_0) => {
                print!("MAKE_HEAP_VAL");
                disassemble_op_arg(arg_0);
                println!();
            },
            Instruction::MakeHeapObject(arg_0) => {
                print!("MAKE_HEAP_OBJ");
                disassemble_op_arg(arg_0);
                println!();
            }
            Instruction::Replace(arg_0, arg_1) => {
                print!("REPLACE ");
                disassemble_op_arg(arg_0);
                disassemble_op_arg(arg_1);
                println!();
            },
            Instruction::Neg(arg_0) => {
                print!("NEG ");
                disassemble_op_arg(arg_0);
                println!();
            },
            Instruction::Inc(arg_0) => {
                print!("INC ");
                disassemble_op_arg(arg_0);
                println!();
            },
            Instruction::Dec(arg_0) => {
                print!("DEC ");
                disassemble_op_arg(arg_0);
                println!();
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
                print!("JMP_IF ");
                disassemble_op_arg(arg_0);
                disassemble_op_arg(arg_1);
                println!();
            },
            Instruction::JumpElse(arg_0, arg_1) => {
                print!("JMP_ELSE ");
                disassemble_op_arg(arg_0);
                disassemble_op_arg(arg_1);
                println!();
            },
            Instruction::Jump(arg_0) => {
                print!("JMP ");
                disassemble_op_arg(arg_0);
                println!();
            },
            Instruction::Return(arg_0) => {
                print!("RETURN ");
                disassemble_op_arg(arg_0);
                println!();
            },
            Instruction::Leave => {
                println!("LEAVE");
            },
            Instruction::Call(arg_0, arg_1) => {
                print!("CALL ");
                disassemble_op_arg(arg_0);
                disassemble_op_arg(arg_1);
                println!();
            },
            Instruction::InstanceCall(arg_0, arg_1, arg_2) => {
                print!("INST_CALL ");
                disassemble_op_arg(arg_0);
                disassemble_op_arg(arg_1);
                disassemble_op_arg(arg_2);
                println!();
            },
            Instruction::NativeCall(native_id) => {
                print!("NATIVE_CALL");
                disassemble_op_arg(native_id);
                println!();
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
