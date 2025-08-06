use std::collections::VecDeque;

use crate::vm::value::Value;
use crate::vm::callable::ExecStatus;
use crate::vm::bytecode::{Procedure, Program};

#[allow(clippy::empty_line_after_doc_comments)]
/**
 * VM Opcode Notes:
 * Self::Nop => 0
 * Self::LoadConst => 1
 * Self::Push => 2
 * Self::Pop => 3
 * Self::Replace => 4
 * Self::Neg => 5
 * Self::Inc => 6
 * Self::Dec => 7
 * Self::Add => 8
 * Self::Sub => 9
 * Self::Mul => 10
 * Self::Div => 11
 * Self::CompareEq => 12
 * Self::CompareNe => 13
 * Self::CompareLt => 14
 * Self::CompareGt => 15
 * Self::JumpIf => 16
 * Self::JumpElse => 17
 * Self::Jump => 18
 * Self::Return => 19
 * Self::Call => 20
 * Self::NativeCall => 21 (TODO)
 */

#[allow(dead_code)]
struct CallFrame {
    pub callee_args: Vec<Value>,
    pub callee_id: i32,
    pub callee_pos: i32,
}

/// TODO: implement the Engine!!
#[allow(dead_code)]
pub struct Engine {
    program: Program,
    frames: VecDeque<CallFrame>,
    stack: VecDeque<Value>,

    /// INFO: Holds the current Procedure index.
    rpid: i32,

    /// INFO: Holds the current instruction pointer.
    rip: i32,

    /// INFO: Holds the "base-pointer" into the Value stack for the current procedure call.
    rbp: i32,

    /// INFO: Indicates execution status, including when to abort the program early.
    status: ExecStatus,
}

impl Engine {
    pub fn new(program_arg: Program) -> Self {
        let program_main_id_opt = program_arg.get_entry_procedure_id();

        let mut initial_frames = VecDeque::<CallFrame>::new();
        let mut saved_main_id = -1;

        if let Some(main_id) = program_main_id_opt {
            saved_main_id = main_id;

            initial_frames.push_back(CallFrame {
                callee_args: Vec::new(),
                callee_id: main_id,
                callee_pos: 0,
            });
        }

        Self {
            program: program_arg,
            frames: initial_frames,
            stack: VecDeque::new(),
            rpid: saved_main_id,
            rip: 0,
            rbp: 0,
            status: ExecStatus::Ok,
        }
    }

    // fn do_load_const(&mut self) {}
    // fn do_push(&mut self) {}
    // fn do_pop(&mut self) {}
    // fn do_replace(&mut self) {}
    // fn do_neg(&mut self) {}
    // fn do_inc(&mut self) {}
    // fn do_dec(&mut self) {}
    // fn do_add(&mut self) {}
    // fn do_sub(&mut self) {}
    // fn do_mul(&mut self) {}
    // fn do_div(&mut self) {}
    // fn do_cmp_eq(&mut self) {}
    // fn do_cmp_ne(&mut self) {}
    // fn do_cmp_lt(&mut self) {}
    // fn do_cmp_gt(&mut self) {}
    // fn do_jump_if(&mut self) {}
    // fn do_jump_else(&mut self) {}
    // fn do_jump(&mut self) {}
    // fn do_return(&mut self) {}
    // fn do_call(&mut self) {}
    // fn do_native_call(&mut self) {}

    /// TODO: implement after previous op methods- the `do_call` method must use this! 
    #[allow(unused_variables)]
    pub fn dispatch_virtual(&mut self, fun: &Procedure) -> ExecStatus {
        ExecStatus::GeneralError
    }

    // TODO: implement...
    pub fn run(&mut self) -> ExecStatus {
        ExecStatus::GeneralError
    }
}