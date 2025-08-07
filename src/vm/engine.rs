use std::collections::VecDeque;

use crate::vm::value::Value;
use crate::vm::callable::ExecStatus;
use crate::vm::bytecode::{self, ArgMode, Program};

struct CallFrame {
    /// NOTE: Tracks current callee's arguments.
    pub callee_args: Vec<Value>,

    /// NOTE: Tracks caller procedure ID.
    pub caller_id: i32,

    /// NOTE: Tracks caller procedure ret-address.
    pub caller_pos: i32,

    pub old_rbp: i32,
}

pub struct Engine {
    program: Program,
    frames: VecDeque<CallFrame>,
    stack: Vec<Value>,

    /// INFO: Holds the current Procedure index.
    rpid: i32,

    /// INFO: Holds the current instruction pointer.
    rip: i32,

    /// INFO: Holds the "base-pointer" into the Value stack for the current procedure call.
    rbp: i32,

    /// INFO: Denotes the top of the stack.
    rsp: i32,

    stack_limit: i32,

    /// INFO: Indicates execution status, including when to abort the program early.
    status: ExecStatus,
}

impl Engine {
    pub fn new(program_arg: Program, stack_size: i32) -> Self {
        let program_main_id_opt = program_arg.get_entry_procedure_id();

        let mut initial_frames = VecDeque::<CallFrame>::new();
        let mut saved_main_id = -1;

        if let Some(main_id) = program_main_id_opt {
            saved_main_id = main_id;

            initial_frames.push_back(CallFrame {
                callee_args: Vec::new(),
                caller_id: 0,
                caller_pos: 0,
                old_rbp: 0,
            });
        }

        let initial_stack_size = stack_size as usize;
        let mut initial_stack_mem = Vec::<Value>::with_capacity(initial_stack_size);
        initial_stack_mem.resize(initial_stack_size, Value::Empty());

        Self {
            program: program_arg,
            frames: initial_frames,
            stack: initial_stack_mem,
            rpid: saved_main_id,
            rip: 0,
            rbp: 0,
            rsp: -1,
            stack_limit: stack_size,
            status: ExecStatus::Ok,
        }
    }

    fn fetch_instruction(&self) -> &bytecode::Instruction {
        unsafe {
            self.program.get_procedures().get_unchecked(
                self.rpid as usize
            ).get_chunk().get_code().get_unchecked(
                self.rip as usize
            )
        }
    }

    fn fetch_constant(&self, const_id: i32) -> &Value {
        unsafe {
            self.program.get_procedures().get_unchecked(
                self.rpid as usize
            ).get_chunk().get_constant(
                const_id
            )
        }
    }

    fn fetch_stack_temp(&self, base_offset: i32) -> &Value {
        let absolute_offset = self.rbp + base_offset;

        unsafe {
            self.stack.get_unchecked(
                absolute_offset as usize
            )
        }
    }

    fn fetch_stored_arg(&self, arg_id: i32) -> &Value {
        let current_frame_ref = &self.frames.back().unwrap().callee_args;

        unsafe {
            current_frame_ref.get_unchecked(
                arg_id as usize
            )
        }
    }

    fn fetch_value_by(&self, arg: bytecode::Argument) -> Option<&Value> {
        let arg_mode = arg.0;
        let arg_id = arg.1;

        match arg_mode {
            ArgMode::ConstantId => Some(self.fetch_constant(arg_id)),
            ArgMode::StackOffset => Some(self.fetch_stack_temp(arg_id)),
            ArgMode::ArgumentId => Some(self.fetch_stored_arg(arg_id)),
            _ => None
        }
    }

    fn push_in(&mut self, temp: Value){
        if self.rsp > self.stack_limit {
            self.status = ExecStatus::AccessError;
        }

        self.rsp += 1;
        *self.stack.get_mut(self.rsp as usize).unwrap() = temp;
    }

    fn pop_off(&mut self) -> Option<Value> {
        if self.rsp < 0 {
            self.status = ExecStatus::AccessError;
            return None;
        }

        let temp_value = self.stack.get_mut(self.rsp as usize).unwrap();
        self.rsp -= 1;

        Some(*temp_value)
    }

    fn do_load_const(&mut self, const_id: bytecode::Argument) {
        let constant_id = const_id.1;

        self.push_in(
            *self.fetch_constant(constant_id)
        );

        self.rip += 1;
    }

    fn do_push(&mut self, source: bytecode::Argument) {
        let pushing_item = self.fetch_value_by(source);

        if self.rsp > self.stack_limit {
            self.status = ExecStatus::AccessError;
            return;
        }

        if let Some(val) = pushing_item {
            *self.stack.get_mut((self.rsp + 1) as usize).unwrap() = *val;
            self.rsp += 1;
            self.rip += 1;
        } else {
            self.status = ExecStatus::AccessError;
        }
    }

    fn do_pop(&mut self) {
        if self.rsp <= 0 {
            self.status = ExecStatus::ValueError;
            return;
        }

        self.rsp -= 1;
        self.rip += 1;
    }

    fn do_replace(&mut self, target: bytecode::Argument, source: bytecode::Argument) {
        let target_slot = target.1;
        let incoming_value_opt = self.fetch_value_by(source);

        if incoming_value_opt.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }
        
        unsafe {
            let incoming_value = incoming_value_opt.unwrap_unchecked();

            *self.stack.get_unchecked_mut(
                target_slot as usize
            ) = *incoming_value;
        }

        self.rip += 1;
    }

    fn do_neg(&mut self, target: bytecode::Argument) {
        if target.0 != ArgMode::StackOffset {
            self.status = ExecStatus::AccessError;
            return;
        }

        let target_slot = self.rbp + target.1;

        unsafe {
            self.stack.get_unchecked_mut(
                target_slot as usize
            ).negate();
        }

        self.rip += 1;
    }

    fn do_inc(&mut self, target: bytecode::Argument) {
        if target.0 != ArgMode::StackOffset {
            self.status = ExecStatus::AccessError;
            return;
        }

        let target_slot = self.rbp + target.1;

        unsafe {
            self.stack.get_unchecked_mut(
                target_slot as usize
            ).increment();
        }

        self.rip += 1;
    }

    fn do_dec(&mut self, target: bytecode::Argument) {
        if target.0 != ArgMode::StackOffset {
            self.status = ExecStatus::AccessError;
            return;
        }

        let target_slot = self.rbp + target.1;

        unsafe {
            self.stack.get_unchecked_mut(
                target_slot as usize
            ).decrement();
        }

        self.rip += 1;
    }

    fn do_add(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let lhs_value = lhs_temp.unwrap_unchecked().add(
                rhs_temp.as_ref().unwrap_unchecked()
            );
            self.push_in(lhs_value);
        }

        self.rip += 1;
    }

    fn do_sub(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let lhs_value = lhs_temp.unwrap_unchecked().sub(
                rhs_temp.as_ref().unwrap_unchecked()
            );
            self.push_in(lhs_value);
        }

        self.rip += 1;
    }

    fn do_mul(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let lhs_value = lhs_temp.unwrap_unchecked().mul(
                rhs_temp.as_ref().unwrap_unchecked()
            );
            self.push_in(lhs_value);
        }

        self.rip += 1;
    }

    fn do_div(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let lhs_value = lhs_temp.unwrap_unchecked().div(
                rhs_temp.as_ref().unwrap_unchecked()
            );

            if let Value::Empty() = lhs_value {
                self.status = ExecStatus::BadMath;
                return;
            }

            self.push_in(lhs_value);
        }

        self.rip += 1;
    }

    fn do_cmp_eq(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let lhs_value = lhs_temp.unwrap_unchecked().is_equal(
                rhs_temp.as_ref().unwrap_unchecked()
            );

            self.push_in(Value::Bool(lhs_value));
        }

        self.rip += 1;
    }

    fn do_cmp_ne(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let cmp_value = lhs_temp.unwrap_unchecked().is_unequal(
                rhs_temp.as_ref().unwrap_unchecked()
            );

            self.push_in(Value::Bool(cmp_value));
        }

        self.rip += 1;
    }

    fn do_cmp_lt(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let cmp_value = lhs_temp.unwrap_unchecked().is_lesser(
                rhs_temp.as_ref().unwrap_unchecked()
            );

            self.push_in(Value::Bool(cmp_value));
        }

        self.rip += 1;
    }

    fn do_cmp_gt(&mut self) {
        let rhs_temp = self.pop_off();
        let lhs_temp = self.pop_off();

        if lhs_temp.is_none() || rhs_temp.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        unsafe {
            let cmp_value = lhs_temp.unwrap_unchecked().is_greater(
                rhs_temp.as_ref().unwrap_unchecked()
            );

            self.push_in(Value::Bool(cmp_value));
        }

        self.rip += 1;
    }

    fn do_jump_if(&mut self, test: bytecode::Argument, jump_to: bytecode::Argument) {
        let test_value = self.fetch_value_by(test);

        if test_value.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        let (jump_arg_mode, jump_target)= jump_to;

        if jump_arg_mode != ArgMode::CodeOffset {
            self.status = ExecStatus::BadArgs;
            return;
        }

        unsafe {
            if test_value.unwrap_unchecked().test() {
                self.rip = jump_target;
            } else {
                self.rip += 1;
            }
        }

        self.rsp -= 1;
    }
    
    fn do_jump_else(&mut self, test: bytecode::Argument, jump_to: bytecode::Argument) {
        let test_value = self.fetch_value_by(test);

        if test_value.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        let (jump_arg_mode, jump_target)= jump_to;

        if jump_arg_mode != ArgMode::CodeOffset {
            self.status = ExecStatus::BadArgs;
            return;
        }

        unsafe {
            if !test_value.unwrap_unchecked().test() {
                self.rip = jump_target;
            } else {
                self.rip += 1;
            }
        }

        self.rsp -= 1; // Pop temporary boolean value that was checked...
    }
    
    fn do_jump(&mut self, jump_to: bytecode::Argument) {
        if jump_to.0 != ArgMode::CodeOffset {
            self.status = ExecStatus::BadArgs;
            return;
        }
        
        self.rip = jump_to.1;
    }
    
    fn do_return(&mut self, source: bytecode::Argument) {
        let result_value_opt = self.fetch_value_by(source);
        
        if result_value_opt.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        let result_temp = result_value_opt.unwrap();
        *self.stack.get_mut(self.rbp as usize).unwrap() = *result_temp;

        let returning_frame = self.frames.back().unwrap();
        self.rpid = returning_frame.caller_id;
        self.rip = returning_frame.caller_pos;
        self.rsp = self.rbp;

        self.rbp = returning_frame.old_rbp;
        self.frames.pop_back();
    }

    fn do_call(&mut self, procedure_id: bytecode::Argument, arg_count: bytecode::Argument) {
        let (proc_arg_mode, proc_id) = procedure_id;

        if proc_arg_mode != ArgMode::ProcedureId {
            self.status = ExecStatus::BadArgs;
            return;
        }

        let (_, mut pending_arg_count) = arg_count;

        let mut temp_callee_args = Vec::<Value>::new();

        while pending_arg_count > 0 {
            if let Some(temp_arg) = self.pop_off() {   
                temp_callee_args.push(temp_arg);
            } else {
                eprintln!("RunError: Could not pop off args into callee ArgStore:\n\trbp = {}, rsp = {}", self.rbp, self.rsp);
                self.status = ExecStatus::AccessError;
                return;
            }

            pending_arg_count -= 1;
        }

        temp_callee_args.reverse();

        let ret_instruction_pos = self.rip + 1;

        self.frames.push_back(
            CallFrame {
                callee_args: temp_callee_args,
                caller_id: self.rpid,
                caller_pos: ret_instruction_pos,
                old_rbp: self.rbp,
            }
        );

        self.rpid = proc_id;
        self.rip = 0;
        self.rbp = self.rsp + 1;
    }
    
    // fn do_native_call(&mut self) {}

    fn is_done(&mut self) -> bool {
        self.frames.is_empty() || self.status != ExecStatus::Ok
    }

    pub fn run(&mut self) -> ExecStatus {
        while !self.is_done() {
            let next_instr = self.fetch_instruction();

            match next_instr {
                bytecode::Instruction::Nop => {
                    self.rip += 1;
                },
                bytecode::Instruction::LoadConst(source) => {
                    self.do_load_const(*source);
                },
                bytecode::Instruction::Push(source) => {
                    self.do_push(*source);
                },
                bytecode::Instruction::Pop => {
                    self.do_pop();
                },
                bytecode::Instruction::Replace(target, source) => {
                    self.do_replace(*target, *source);
                },
                bytecode::Instruction::Neg(target) => {
                    self.do_neg(*target);
                },
                bytecode::Instruction::Inc(target) => {
                    self.do_inc(*target);
                },
                bytecode::Instruction::Dec(target) => {
                    self.do_dec(*target);
                },
                bytecode::Instruction::Add => {
                    self.do_add();
                },
                bytecode::Instruction::Sub => {
                    self.do_sub();
                },
                bytecode::Instruction::Mul => {
                    self.do_mul();
                },
                bytecode::Instruction::Div => {
                    self.do_div();
                },
                bytecode::Instruction::CompareEq => {
                    self.do_cmp_eq();
                },
                bytecode::Instruction::CompareNe => {
                    self.do_cmp_ne();
                },
                bytecode::Instruction::CompareLt => {
                    self.do_cmp_lt();
                },
                bytecode::Instruction::CompareGt => {
                    self.do_cmp_gt();
                },
                bytecode::Instruction::JumpIf(test, jump_target) => {
                    self.do_jump_if(*test, *jump_target);
                },
                bytecode::Instruction::JumpElse(test, jump_target) => {
                    self.do_jump_else(*test, *jump_target);
                },
                bytecode::Instruction::Jump(jump_target) => {
                    self.do_jump(*jump_target);
                },
                bytecode::Instruction::Return(source) => {
                    self.do_return(*source);
                },
                bytecode::Instruction::Call(proc_id, arg_count) => {
                    self.do_call(*proc_id, *arg_count);
                },
            }
        }

        self.status
    }
}