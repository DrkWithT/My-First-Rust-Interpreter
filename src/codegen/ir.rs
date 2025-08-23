use crate::semantics::types::OperatorTag;

#[repr(i32)]
#[derive(Clone, PartialEq)]
pub enum Region {
    Immediate,
    TempStack,
    ArgStore,
    ObjectHeap,
    Functions,
    Natives,
    BlockId,
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq)]
pub enum Opcode {
    Nop,
    LoadConst,
    Push,
    Pop,
    MakeHeapValue,
    Replace,
    Neg,
    Inc,
    Dec,
    Add,
    Sub,
    Mul,
    Div,
    GenBeginLoop,
    GenPatch,
    GenPatchBack,
    CompareEq,
    CompareNe,
    CompareLt,
    CompareGt,
    JumpIf,
    JumpElse,
    Jump,
    Return,
    Call,
    NativeCall,
}

impl Opcode {
    pub fn arity(&self) -> i32 {
        match self {
            Self::Nop => 0,
            Self::LoadConst => 1,
            Self::Push => 1,
            Self::Pop => 0,
            Self::MakeHeapValue => 1,
            Self::Replace => 2,
            Self::Neg => 1,
            Self::Inc => 1,
            Self::Dec => 1,
            Self::Add => 0,
            Self::Sub => 0,
            Self::Mul => 0,
            Self::Div => 0,
            Self::GenBeginLoop => 0,
            Self::GenPatch => 0,
            Self::GenPatchBack => 0,
            Self::CompareEq => 0,
            Self::CompareNe => 0,
            Self::CompareLt => 0,
            Self::CompareGt => 0,
            Self::JumpIf => 2,
            Self::JumpElse => 2,
            Self::Jump => 1,
            Self::Return => 1,
            Self::Call => 2,
            Self::NativeCall => 2,
        }
    }

    /// NOTE: `-1000` is a dud value which denotes that the relative stack base to offset from is reset for the `Opcode`.
    pub fn get_stack_delta(&self) -> i32 {
        match self {
            Self::Nop => 0,
            Self::LoadConst => 1,
            Self::Push => 1,
            Self::Pop => -1,
            Self::MakeHeapValue => 1,
            Self::Replace => 0,
            Self::Neg => 0,
            Self::Inc => 0,
            Self::Dec => 0,
            Self::Add => -1,
            Self::Sub => -1,
            Self::Mul => -1,
            Self::Div => -1,
            Self::GenBeginLoop => 0,
            Self::GenPatch => 0,
            Self::GenPatchBack => 0,
            Self::CompareEq => -1,
            Self::CompareNe => -1,
            Self::CompareLt => -1,
            Self::CompareGt => -1,
            Self::JumpIf => -1,
            Self::JumpElse => -1,
            Self::Jump => 0,
            Self::Return => -1000,
            Self::Call => 0,
            Self::NativeCall => 0,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Nop => "NOP",
            Self::LoadConst => "LOAD_CONST",
            Self::Push => "PUSH",
            Self::Pop => "POP",
            Self::MakeHeapValue => "MAKE_OBJECT",
            Self::Replace => "REPLACE",
            Self::Neg => "NEG",
            Self::Inc => "INC",
            Self::Dec => "DEC",
            Self::Add => "ADD",
            Self::Sub => "SUB",
            Self::Mul => "MUL",
            Self::Div => "DIV",
            Self::GenBeginLoop => "#GEN_BEGIN_LOOP",
            Self::GenPatch => "#GEN_PATCH",
            Self::GenPatchBack => "#GEN_PATCH_BACK",
            Self::CompareEq => "CMP_EQ",
            Self::CompareNe => "CMP_NE",
            Self::CompareLt => "CMP_LT",
            Self::CompareGt => "CMP_GT",
            Self::JumpIf => "JMP_IF",
            Self::JumpElse => "JMP_ELSE",
            Self::Jump => "JMP",
            Self::Return => "RET",
            Self::Call => "CALL",
            Self::NativeCall => "NATIVE_CALL",
        }
    }
}

/// TODO: implement Opcodes `Access` later for array support.
pub fn ast_op_to_ir_op(arg: OperatorTag) -> Opcode {
    match arg {
        OperatorTag::Noop => Opcode::Nop,
        // OperatorTag::Access => Opcode::Nop,
        OperatorTag::Negate => Opcode::Neg,
        OperatorTag::Times => Opcode::Mul,
        OperatorTag::Slash => Opcode::Div,
        OperatorTag::Plus => Opcode::Add,
        OperatorTag::Minus => Opcode::Sub,
        OperatorTag::Equality => Opcode::CompareEq,
        OperatorTag::Inequality => Opcode::CompareNe,
        OperatorTag::LessThan => Opcode::CompareLt,
        OperatorTag::GreaterThan => Opcode::CompareGt,
        OperatorTag::Assign => Opcode::Replace,
        _ => Opcode::Nop,
    }
}

pub type Locator = (Region, i32);

pub enum Instruction {
    Nonary(Opcode),
    Unary(Opcode, Locator),
    Binary(Opcode, Locator, Locator),
}

impl Instruction {
    pub fn get_opcode(&self) -> Opcode {
        match self {
            Self::Nonary(op) => *op,
            Self::Unary(op, _) => *op,
            Self::Binary(op, _, _) => *op,
        }
    }

    pub fn get_arity(&self) -> i32 {
        match self {
            Self::Nonary(_op) => 0,
            Self::Unary(_op, _) => 1,
            Self::Binary(_op, _, _) => 2,
        }
    }

    pub fn get_arg_0(&self) -> Option<&Locator> {
        match self {
            Self::Nonary(_) => None,
            Self::Unary(_, arg_0) => Some(arg_0),
            Self::Binary(_, arg_0, _) => Some(arg_0),
        }
    }

    pub fn get_arg_1(&self) -> Option<&Locator> {
        match self {
            Self::Nonary(_) => None,
            Self::Unary(_, _) => None,
            Self::Binary(_, _, arg_1) => Some(arg_1),
        }
    }
}

pub struct Node {
    steps: Vec<Instruction>,
    truthy_id: i32,
    falsy_id: i32,
}

impl Node {
    pub fn new(steps_arg: Vec<Instruction>, truthy_id_arg: i32, falsy_id_arg: i32) -> Self {
        Self {
            steps: steps_arg,
            truthy_id: truthy_id_arg,
            falsy_id: falsy_id_arg,
        }
    }

    pub fn get_steps(&self) -> &Vec<Instruction> {
        &self.steps
    }

    pub fn get_steps_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.steps
    }

    pub fn set_left_neighbor_id(&mut self, id: i32) {
        self.truthy_id = id;
    }

    pub fn set_right_neighbor_id(&mut self, id: i32) {
        self.falsy_id = id;
    }

    pub fn set_neighbor_ids(&mut self, left_id: i32, right_id: i32) {
        self.truthy_id = left_id;
        self.falsy_id = right_id;
    }

    pub fn get_truthy_id(&self) -> Option<i32> {
        if self.truthy_id != -1 {
            return Some(self.truthy_id);
        }

        None
    }

    pub fn get_falsy_id(&self) -> Option<i32> {
        if self.falsy_id != -1 {
            return Some(self.falsy_id);
        }

        None
    }

    pub fn append_instruction(&mut self, step: Instruction) {
        self.steps.push(step);
    }
}

pub struct CFG {
    nodes: Vec<Node>,
    count: i32,
}

impl CFG {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            count: 0,
        }
    }

    pub fn get_node_count(&self) -> i32 {
        self.count
    }

    pub fn get_root(&self) -> Option<&Node> {
        self.nodes.first()
    }

    pub fn get_newest_node_tip_pos(&self) -> i32 {
        self.nodes.last().unwrap().get_steps().len() as i32 - 1
    }

    pub fn get_newest_node_mut(&mut self) -> &mut Node {
        self.nodes.get_mut(self.count as usize).unwrap()
    }

    pub fn get_node_ref(&self, block_id: i32) -> Option<&Node> {
        self.nodes.get(block_id as usize)
    }

    pub fn get_node_mut(&mut self, block_id: i32) -> Option<&mut Node> {
        self.nodes.get_mut(block_id as usize)
    }

    pub fn add_instruction_recent(&mut self, arg: Instruction) {
        if self.nodes.is_empty() {
            return;
        }

        self.nodes.last_mut().unwrap().append_instruction(arg);
    }

    pub fn add_node(&mut self, node: Node) -> (Option<&Node>, i32) {
        let next_id = self.count;
        self.nodes.push(node);
        self.count += 1;

        (self.nodes.last(), next_id)
    }

    pub fn get_left_neighbor(&self, target: &Node) -> Option<&Node> {
        let target_truthy_id_opt = target.get_truthy_id();

        target_truthy_id_opt?;

        let target_truthy_id = target_truthy_id_opt.unwrap();

        Some(self.nodes.get(target_truthy_id as usize).unwrap())
    }

    pub fn get_right_neighbor(&self, target: &Node) -> Option<&Node> {
        let target_falsy_id_opt = target.get_falsy_id();

        target_falsy_id_opt?;

        let target_truthy_id = target_falsy_id_opt.unwrap();

        Some(self.nodes.get(target_truthy_id as usize).unwrap())
    }

    pub fn connect_nodes_by_id(&mut self, from_id: i32, to_id: i32) {
        if from_id == -1 || to_id == -1 {
            return;
        }

        let target_ref = self.nodes.get_mut(from_id as usize).unwrap();

        if target_ref.get_truthy_id().is_none() {
            target_ref.set_left_neighbor_id(to_id);
        } else {
            target_ref.set_right_neighbor_id(to_id);
        }
    }
}

pub type CFGStorage = Vec<CFG>;
