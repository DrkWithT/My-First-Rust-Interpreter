use std::collections::VecDeque;

use crate::utils::bundle::Bundle;
use crate::vm::bytecode::{self, ArgMode, Program};
use crate::vm::callable::ExecStatus;
use crate::vm::value::Value;
use crate::vm::heap::{HeapValue, ObjectHeap, ObjectTag};

struct CallFrame {
    /// NOTE: Tracks current callee's arguments.
    pub callee_args: Vec<Value>,

    /// NOTE: Tracks caller procedure ID.
    pub caller_id: i32,

    /// NOTE: Tracks caller procedure ret-address.
    pub caller_pos: i32,

    pub old_rbp: i32,

    pub opt_instance: i32,
}

/// TODO: integrate ObjectHeap... add GC sweeping methods.
pub struct Engine {
    heap: ObjectHeap,
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
    pub fn new(mut program_arg: Program, heap_size: usize, stack_size: i32) -> Self {
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
                opt_instance: -1,
            });
        }

        let mut initial_heap = ObjectHeap::new(heap_size);

        for temp_heap_val in program_arg.get_heap_preloadables_mut() {
            let temp_cell_id = initial_heap.try_create_cell(temp_heap_val.get_object_tag());
            *initial_heap.get_cell_mut(temp_cell_id).unwrap().get_value_mut() = std::mem::take(temp_heap_val);
        }

        let initial_stack_size = stack_size as usize;
        let mut initial_stack_mem = Vec::<Value>::with_capacity(initial_stack_size);
        initial_stack_mem.resize(initial_stack_size, Value::Empty());

        Self {
            heap: initial_heap,
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
            self.program
                .get_procedures()
                .get_unchecked(self.rpid as usize)
                .get_chunk()
                .get_code()
                .get_unchecked(self.rip as usize)
        }
    }

    fn try_sweep(&mut self) {
        if !self.heap.is_ripe_for_sweep() {
            return;
        }

        for val in &mut self.stack {
            if let Value::HeapRef(object_id) = val {
                self.heap.try_collect_cell(*object_id);
            }
        }
    }

    fn last_sweep(&mut self) {
        self.heap.force_collect_all();
    }

    fn fetch_constant(&self, const_id: i32) -> &Value {
        unsafe {
            self.program
                .get_procedures()
                .get_unchecked(self.rpid as usize)
                .get_chunk()
                .get_constant(const_id)
        }
    }

    fn fetch_stack_temp(&self, base_offset: i32) -> &Value {
        let absolute_offset = self.rbp + base_offset;

        unsafe {
            self.stack.get_unchecked(absolute_offset as usize)
        }
    }

    fn fetch_stored_arg(&self, arg_id: i32) -> &Value {
        let current_frame_ref = &self.frames.back().unwrap().callee_args;

        unsafe {
            current_frame_ref.get_unchecked(arg_id as usize)
        }
    }

    fn fetch_inst_field(&self, arg_id: i32) -> &Value {
        let instance_ref_slot_id = self.frames.back().unwrap().opt_instance;

        if instance_ref_slot_id == -1 {
            eprintln!("RunWarning: invalid slot ID -1 for instance fetched- crash is inevitable.");
        }

        // TODO: fix calculation for instance_slot_id!
        let instance_ref = self.fetch_stack_temp(instance_ref_slot_id);

        let instance_ref_heap_id = if let Value::HeapRef(obj_id) = instance_ref {
            // println!("fetch_inst_field: found heap-ref case...");
            *obj_id
        } else {
            // println!("fetch_inst_field: found non-heap-ref at stack-slot-{instance_ref_slot_id}: {instance_ref}");
            -1
        };

        if instance_ref_heap_id == -1 {
            eprintln!("RunWarning: invalid reference to instance fetched: heap-id-(-1)... crash inevitable.");
        }

        // unsafe {
            self.heap.get_cell(instance_ref_heap_id).unwrap().get_value().try_ref_instance_field(arg_id).unwrap()
        // }
    }

    fn fetch_value_by(&self, arg: bytecode::Argument) -> Option<&Value> {
        let arg_mode = arg.0;
        let arg_id = arg.1;

        match arg_mode {
            ArgMode::ConstantId => Some(self.fetch_constant(arg_id)),
            ArgMode::StackOffset => Some(self.fetch_stack_temp(arg_id)),
            ArgMode::ArgumentId => Some(self.fetch_stored_arg(arg_id)),
            ArgMode::InstanceFieldId => Some(self.fetch_inst_field(arg_id)),
            _ => None,
        }
    }

    pub fn fetch_heap_value_by(&mut self, arg: bytecode::Argument) -> Option<&mut HeapValue> {
        let (arg_mode, arg_obj_id) = arg;

        if arg_mode != ArgMode::StackOffset {
            self.status = ExecStatus::BadArgs;
            return None;
        }

        let heap_cell_opt = self.heap.get_cell_mut(arg_obj_id);

        if let Some(heap_cell_ref) = heap_cell_opt {
            return Some(heap_cell_ref.get_value_mut());
        }

        None
    }

    fn make_heap_value(&mut self, tag: ObjectTag) -> bool {
        match tag {
            ObjectTag::Varchar => {
                let temp_obj_id = self.heap.try_create_cell(tag);

                if temp_obj_id != -1 {
                    self.push_in(Value::HeapRef(temp_obj_id));
                }

                temp_obj_id != -1
            },
            _ => false,
        }
    }

    pub fn push_in(&mut self, temp: Value) {
        if self.rsp > self.stack_limit {
            eprintln!(
                "RunError: invalid stack top- rbp = {}, rsp = {}",
                self.rbp, self.rsp
            );
            self.status = ExecStatus::AccessError;
            return;
        }

        self.rsp += 1;

        if self.rsp > self.stack_limit {
            eprintln!(
                "RunError: rsp too large: {}",
                self.rsp
            );
            self.status = ExecStatus::AccessError;
            return;
        }

        if let Value::HeapRef(object_id) = &temp {
            let object_ref_opt = self.heap.get_cell_mut(*object_id);

            if let Some(object_ref) = object_ref_opt {
                object_ref.inc_rc();
            } else {
                self.status = ExecStatus::RefError;
            }
        }

        *self.stack.get_mut(self.rsp as usize).unwrap() = temp;
    }

    pub fn pop_off(&mut self) -> Option<Value> {
        if self.rsp < 0 {
            self.status = ExecStatus::AccessError;
            return None;
        }

        let temp_value = self.stack.get_mut(self.rsp as usize).unwrap();
        self.rsp -= 1;

        if let Value::HeapRef(object_id) = temp_value {
            let object_ref_opt = self.heap.get_cell_mut(*object_id);

            if let Some(object_ref) = object_ref_opt {
                object_ref.dec_rc();
            } else {
                self.status = ExecStatus::RefError;
            }
        }

        Some(*temp_value)
    }

    fn do_load_const(&mut self, const_id: bytecode::Argument) {
        let constant_id = const_id.1;

        self.push_in(*self.fetch_constant(constant_id));

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

    fn do_make_heap_value(&mut self, arg: bytecode::Argument) {
        let arg_tag = match arg.1 {
            0 => ObjectTag::Varchar,
            _ => ObjectTag::None,
        };
        
        if !self.make_heap_value(arg_tag) {
            self.status = ExecStatus::RefError;
            return;
        }

        self.rip += 1;
    }

    fn do_make_heap_object(&mut self, arg: bytecode::Argument) {
        // TODO: implement this after adding Instances to HeapValue.
        let instance_field_n = arg.1;
        let mut temp_fields = Vec::<Value>::with_capacity(instance_field_n as usize);
        temp_fields.resize(instance_field_n as usize, Value::Empty());

        let obj_id = self.heap.try_create_cell(ObjectTag::Instance);

        if !self.heap.preload_cell_at(obj_id, HeapValue::Instance(temp_fields)) {
            self.status = ExecStatus::RefError;
            eprintln!("RunError: invalid reference created for a class instance: heap-id-{obj_id}");
            return;
        }

        self.push_in(Value::HeapRef(obj_id));
        let obj_slot_id = self.rsp;
        self.frames.back_mut().unwrap().opt_instance = obj_slot_id;

        self.rip += 1;
    }

    fn do_replace(&mut self, target: bytecode::Argument, source: bytecode::Argument) {
        let target_slot = target.1;
        let incoming_value_opt = self.fetch_value_by(source);

        if incoming_value_opt.is_none() {
            self.status = ExecStatus::AccessError;
            return;
        }

        let instance_slot_id = self.frames.back().unwrap().opt_instance;
        let has_object_field = target.0 == ArgMode::InstanceFieldId && instance_slot_id != -1;

        if !has_object_field {
            unsafe {
                let incoming_value = incoming_value_opt.unwrap_unchecked();
                *self.stack.get_unchecked_mut(target_slot as usize) = *incoming_value;
            }
        } else {
            let instance_ref = self.fetch_stack_temp(instance_slot_id);

            let instance_ref_heap_id = if let Value::HeapRef(obj_id) = instance_ref {
                *obj_id
            } else { -1 };

            if instance_ref_heap_id == -1 {
                eprintln!("RunWarning: invalid instance reference fetched, crash inevitable.");
            }

            unsafe {
                let incoming_value_for_field = incoming_value_opt.unwrap_unchecked();

                *self.heap.get_cell_mut(instance_ref_heap_id).unwrap().get_value_mut().try_ref_instance_field_mut(target_slot).unwrap() = *incoming_value_for_field;
            }
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
            self.stack.get_unchecked_mut(target_slot as usize).negate();
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
            self.stack
                .get_unchecked_mut(target_slot as usize)
                .increment();
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
            self.stack
                .get_unchecked_mut(target_slot as usize)
                .decrement();
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
            let lhs_value = lhs_temp
                .unwrap_unchecked()
                .add(rhs_temp.as_ref().unwrap_unchecked());
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
            let lhs_value = lhs_temp
                .unwrap_unchecked()
                .sub(rhs_temp.as_ref().unwrap_unchecked());
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
            let lhs_value = lhs_temp
                .unwrap_unchecked()
                .mul(rhs_temp.as_ref().unwrap_unchecked());
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
            let lhs_value = lhs_temp
                .unwrap_unchecked()
                .div(rhs_temp.as_ref().unwrap_unchecked());

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
            let lhs_value = lhs_temp
                .unwrap_unchecked()
                .is_equal(rhs_temp.as_ref().unwrap_unchecked());

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
            let cmp_value = lhs_temp
                .unwrap_unchecked()
                .is_unequal(rhs_temp.as_ref().unwrap_unchecked());

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
            let cmp_value = lhs_temp
                .unwrap_unchecked()
                .is_lesser(rhs_temp.as_ref().unwrap_unchecked());

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
            let cmp_value = lhs_temp
                .unwrap_unchecked()
                .is_greater(rhs_temp.as_ref().unwrap_unchecked());

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

        let (jump_arg_mode, jump_target) = jump_to;

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

        let (jump_arg_mode, jump_target) = jump_to;

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

    fn do_leave(&mut self) {
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

        let (_, pending_arg_count) = arg_count;

        let mut temp_callee_args = Vec::<Value>::with_capacity(pending_arg_count as usize);
        temp_callee_args.resize(pending_arg_count as usize, Value::Empty());

        for arg_it in 0..pending_arg_count {
            if let Some(temp_arg) = self.pop_off() {
                unsafe {
                    let arg_insert_it = pending_arg_count - (1 + arg_it);
                    *temp_callee_args.get_unchecked_mut(arg_insert_it as usize) = temp_arg;
                }
            } else {
                eprintln!(
                    "RunError: Could not pop off args into callee ArgStore:\n\trbp = {}, rsp = {}",
                    self.rbp, self.rsp
                );
                self.status = ExecStatus::AccessError;
                return;
            }
        }

        let ret_instruction_pos = self.rip + 1;

        self.frames.push_back(CallFrame {
            callee_args: temp_callee_args,
            caller_id: self.rpid,
            caller_pos: ret_instruction_pos,
            old_rbp: self.rbp,
            opt_instance: -1,
        });

        self.rpid = proc_id;
        self.rip = 0;
        self.rbp = self.rsp + 1;
    }

    fn do_instance_call(&mut self, instance_arg: bytecode::Argument, fun_id_arg: bytecode::Argument, args_n: bytecode::Argument) {
        let instance_ref_stk_slot = self.rbp + instance_arg.1;
        let instance_ref_copy = *self.stack.get_mut(instance_ref_stk_slot as usize).unwrap();

        let (proc_arg_mode, proc_id) = fun_id_arg;

        if proc_arg_mode != ArgMode::ProcedureId {
            self.status = ExecStatus::BadArgs;
            return;
        }

        let (_, pending_arg_count) = args_n;

        let mut temp_callee_args = Vec::<Value>::with_capacity(pending_arg_count as usize);
        temp_callee_args.resize(pending_arg_count as usize, Value::Empty());

        for arg_it in 0..pending_arg_count {
            if let Some(temp_arg) = self.pop_off() {
                unsafe {
                    let arg_insert_it = pending_arg_count - (1 + arg_it);
                    *temp_callee_args.get_unchecked_mut(arg_insert_it as usize) = temp_arg;
                }
            } else {
                eprintln!(
                    "RunError: Could not transfer all args into method's ArgStore:\n\trbp = {}, rsp = {}",
                    self.rbp, self.rsp
                );
                self.status = ExecStatus::AccessError;
                return;
            }
        }

        let ret_caller_id = self.rpid;
        let ret_caller_rbp = self.rbp;
        let ret_instruction_pos = self.rip + 1;

        self.rpid = proc_id;
        self.rip = 0;
        self.rbp = self.rsp + 1;
        self.push_in(instance_ref_copy);

        self.frames.push_back(CallFrame {
            callee_args: temp_callee_args,
            caller_id: ret_caller_id,
            caller_pos: ret_instruction_pos,
            old_rbp: ret_caller_rbp,
            // NOTE: the method call will have the copied instance ref at relative offset 0 because RBP is first set there.
            opt_instance: 0,
        });
    }

    fn do_native_call(&mut self, natives: &Bundle, native_arg: bytecode::Argument) {
        unsafe {
            self.status = natives.get_native(native_arg.1)(self);
        }

        self.rip += 1;
    }

    fn is_done(&mut self) -> bool {
        self.frames.is_empty() || self.status != ExecStatus::Ok
    }

    pub fn run(&mut self, natives: &Bundle) -> ExecStatus {
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
                bytecode::Instruction::MakeHeapValue(tag_arg) => {
                    self.do_make_heap_value(*tag_arg);
                },
                bytecode::Instruction::MakeHeapObject(heap_cell_n_arg) => {
                    self.do_make_heap_object(*heap_cell_n_arg);
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
                bytecode::Instruction::Leave => {
                    self.do_leave();
                }
                bytecode::Instruction::Call(proc_id, arity) => {
                    self.do_call(*proc_id, *arity);
                },
                bytecode::Instruction::InstanceCall(instance_slot, fun_id, arity) => {
                    self.do_instance_call(*instance_slot, *fun_id, *arity);
                },
                bytecode::Instruction::NativeCall(native_id) => {
                    self.do_native_call(natives, *native_id);
                },
            }

            self.try_sweep();
        }

        self.last_sweep();

        let main_result_code = self.stack.first().unwrap();
        let zero_ok = Value::Int(0);

        if self.status == ExecStatus::Ok && !main_result_code.is_equal(&zero_ok) {
            self.status = ExecStatus::NotOk;
        }

        self.status
    }
}
