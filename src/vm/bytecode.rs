// use crate::vm::callable::*;
// use crate::vm::engine::Engine;
use crate::vm::value::Value;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq)]
pub enum ArgMode {
    Dud,
    ConstantId,
    StackOffset,
    ArgumentId,
    CodeOffset,
    // HeapId,
    ProcedureId,
}

impl ArgMode {
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Dud => "unknown",
            Self::ConstantId => "const-id",
            Self::StackOffset => "temp-off",
            Self::ArgumentId => "arg-id",
            Self::CodeOffset => "code-pos",
            // Self::HeapId => "heap-id",
            Self::ProcedureId => "proc-id",
        }
    }
}

pub type Argument = (ArgMode, i32);

pub enum Instruction {
    Nop,
    LoadConst(Argument),
    Push(Argument),
    Pop,
    Replace(Argument, Argument),
    Neg(Argument),
    Inc(Argument),
    Dec(Argument),
    Add,
    Sub,
    Mul,
    Div,
    CompareEq,
    CompareNe,
    CompareLt,
    CompareGt,
    JumpIf(Argument, Argument),
    JumpElse(Argument, Argument),
    Jump(Argument),
    Return(Argument),
    Call(Argument, Argument),
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

    pub fn get_constant_mut(&mut self, arg: i32) -> &mut Value {
        self.constants.get_mut(arg as usize).unwrap()
    }

    pub fn get_code(&self) -> &Vec<Instruction> {
        &self.code
    }
}

pub struct Procedure {
    chunk: Chunk,
    id: i32,
}

impl Procedure {
    pub fn new(chunk_arg: Chunk, id_arg: i32) -> Self {
        Self {
            chunk: chunk_arg,
            id: id_arg,
        }
    }

    pub fn get_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunk
    }

    pub fn get_chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn get_id(&self) -> i32 {
        self.id
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

    pub fn get_procedures_mut(&mut self) -> &mut Vec<Procedure> {
        &mut self.procedures
    }

    pub fn get_entry_procedure_id(&self) -> Option<i32> {
        if self.entry_id != -1 {
            Some(self.entry_id)
        } else {
            None
        }
    }
}
