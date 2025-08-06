use crate::vm::callable::*;
use crate::vm::engine::Engine;
use crate::vm::value::Value;

pub enum Instruction {
    Nop,
    LoadConst(i32),
    Push(i32),
    Pop,
    Replace(i32, i32),
    Neg(i32),
    Inc(i32),
    Dec(i32),
    Add,
    Sub,
    Mul,
    Div,
    CompareEq,
    CompareNe,
    CompareLt,
    CompareGt,
    JumpIf(i32, i32),
    JumpElse(i32, i32),
    Jump(i32),
    Return(i32),
    Call(i32),
    // NativeCall(i32),
}

pub struct Chunk {
    constants: Vec<Value>,
    code: Vec<Instruction>,
}

impl Chunk {
    pub fn new(constants_arg: Vec<Value>, code_arg: Vec<Instruction>) -> Self {
        Self {
            constants: constants_arg,
            code: code_arg,
        }
    }

    pub fn get_constant(&self, arg: i32) -> &Value {
        self.constants.get(arg as usize).unwrap()
    }

    pub fn get_code(&self) -> &Vec<Instruction> {
        &self.code
    }
}

pub struct Procedure {
    chunk: Chunk,
    id: i32,
}

impl Callable<Engine> for Procedure {
    fn invoke(&self, vm: &mut Engine) -> ExecStatus {
        vm.dispatch_virtual(self)
    }

    fn get_id(&self) -> i32 {
        self.id
    }
}

impl Procedure {
    pub fn new(chunk_arg: Chunk, id_arg: i32) -> Self {
        Self {
            chunk: chunk_arg,
            id: id_arg,
        }
    }

    pub fn get_chunk(&self) -> &Chunk {
        &self.chunk
    }
}

#[derive(Default)]
pub struct Program {
    procedures: Vec<Procedure>,
    entry_id: i32,
}

impl Program {
    pub fn new(procedures_arg: Vec<Procedure>, entry_id_arg: i32) -> Self {
        Self {
            procedures: procedures_arg,
            entry_id: entry_id_arg,
        }
    }

    pub fn get_procedures(&self) -> &Vec<Procedure> {
        &self.procedures
    }

    pub fn get_entry_procedure_id(&self) -> Option<i32> {
        if self.entry_id != -1 {
            Some(self.entry_id)
        } else {
            None
        }
    }
}
