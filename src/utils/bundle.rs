/*
 * bundle.rs
 * This implements the collection of native procedures for use by the interpreter.
 * DrkWithT
 * 8/9/2025
 */

use std::collections::HashMap;

use crate::vm::{callable::Callable, engine::Engine};

/// Stores information per native function in a Bundle: ID & expected arity.
#[derive(Clone, Copy)]
pub struct NativeBrief {
    pub id: i32,
    pub arity: i32,
}

/*
 * Defines the collection of native procedures used during runtime.
 * Stored state includes a registry table for native procedure names -> their ID's, the correspondingly ordered Vec of native routines, and an internal counter for registering ID's.
 */
#[derive(Default)]
pub struct Bundle {
    registry: HashMap<&'static str, NativeBrief>,
    routines: Vec<Callable<Engine>>,
    next_id: i32,
}

impl Bundle {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            routines: Vec::<Callable<Engine>>::new(),
            next_id: 0,
        }
    }

    /// NOTE: all arguments to `arity_arg` must match the number of arguments popped off the Engine stack.
    pub fn register_native(&mut self, name: &'static str, callable_arg: Callable<Engine>, arity_arg: i32) -> bool {
        if self.registry.contains_key(name) {
            return false;
        }

        let next_callable_id = self.next_id;

        self.registry.insert(name, NativeBrief { id: next_callable_id, arity: arity_arg });
        self.routines.push(callable_arg);
        self.next_id += 1;

        true
    }

    /// # SAFETY
    /// This Bundle method is unsafe for performance reasons, as index-checked dispatches to native functions would create unneeded slowdowns. Thus, all ID's passed must be valid!
    pub unsafe fn get_native(&self, native_id: i32) -> &Callable<Engine> {
        unsafe {
            self.routines.get_unchecked(native_id as usize)
        }
    }

    pub fn peek_registry(&self) -> &HashMap<&'static str, NativeBrief> {
        &self.registry
    }
}
