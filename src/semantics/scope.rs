use std::collections::{HashMap};

use crate::semantics::types::ValueCategoryTag;

/// See `SemanticNote::DataValue`
pub type RawDataValue = (i32, ValueCategoryTag);

/// See `SemanticNote::Callable`
pub type RawCallable<'a> = (&'a Vec<i32>, i32, i32);

/// Stores the following semantic info per expr / decl: type-index, value category
#[derive(Clone, PartialEq)]
pub enum SemanticNote {
    /// contains no info
    Dud,

    /// contains type-index & value category info
    DataValue(i32, ValueCategoryTag),
    
    /// contains function-type-index, function-return-type-index, and arity info
    Callable(Vec<i32>, i32, i32),
}

impl SemanticNote {
    pub fn is_dud(&self) -> bool {
        if let SemanticNote::Dud = self {
            return true;
        }

        false
    }

    pub fn try_unbox_data_value(&self) -> Option<RawDataValue> {
        if let SemanticNote::DataValue(type_id, value_category) = self {
            return Some((*type_id, *value_category));
        }

        None
    }

    pub fn try_unbox_callable_info(&self) -> Option<RawCallable> {
        if let SemanticNote::Callable(full_type_id, result_type_id, arity_n) = self {
            return Some((full_type_id, *result_type_id, *arity_n));
        }

        None
    }
}

pub struct Scope {
    entries: HashMap<String, SemanticNote>,
    name: String,
}

impl Scope {
    pub fn new(scope_title: &str) -> Self {
        Self {
            entries: HashMap::new(),
            name: String::from(scope_title),
        }
    }

    pub fn try_get_entry(&self, name: &str) -> Option<&SemanticNote> {
        self.entries.get(name)
    }

    pub fn try_set_entry(&mut self, name: &str, arg: SemanticNote) -> bool {
        if self.entries.contains_key(name) {
            return false;
        }

        self.entries.insert(String::from(name), arg);
        true
    }

    pub fn get_name_str(&self) -> &str {
        self.name.as_str()
    }
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl Default for ScopeStack {
    fn default() -> Self {
        let temp_scopes = vec![Scope::new("#GLOBAL")];

        Self {
            scopes: temp_scopes,
        }
    }
}

impl ScopeStack {
    pub fn enter_scope(&mut self, name: &str) {
        self.scopes.push(Scope::new(name));
    }

    pub fn global_scope(&self) -> Option<&Scope> {
        self.scopes.first()
    }

    pub fn global_scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.first_mut()
    }

    pub fn current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    pub fn leave_scope(&mut self) {
        if self.scopes.len() <= 1 {
            return;
        }

        self.scopes.pop();
    }

    pub fn lookup_name_info(&self, name: &str) -> SemanticNote {
        for scope in &self.scopes {
            let temp_entry_opt = scope.try_get_entry(name);

            if temp_entry_opt.is_none() {
                continue;
            }

            return temp_entry_opt.unwrap().clone();
        }

        SemanticNote::Dud
    }

    pub fn record_name_info(&mut self, name: &str, arg: SemanticNote) -> bool {
        self.scopes.last_mut().unwrap().try_set_entry(name, arg)
    }
}
