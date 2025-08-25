use std::collections::HashMap;

use crate::semantics::scope::SemanticNote;
use crate::semantics::types::ClassAccess;

pub struct ClassMember {
    pub note: SemanticNote,
    pub access_mod: ClassAccess,
}

/**
 * ### ABOUT
 * Represents the semantic scope per class type, containing field and method information.
 */
pub struct ClassBlueprint {
    entries: HashMap<String, ClassMember>,
    typing_id: i32,
}

impl ClassBlueprint {
    pub fn new(typing_id_arg: i32) -> Self {
        Self {
            entries: HashMap::<String, ClassMember>::default(),
            typing_id: typing_id_arg,
        }
    }

    pub fn get_type_id(&mut self) -> i32 {
        self.typing_id
    }

    pub fn try_get_entry_mut(&mut self, name_view: &str) -> Option<&mut ClassMember> {
        self.entries.get_mut(name_view)
    }

    pub fn try_set_entry(&mut self, name_view: &str, note: ClassMember) -> bool {
        self.entries.insert(String::from(name_view), note).is_none()
    }
}

/**
 * ### ABOUT
 * Maps class types by their exact type IDs to their semantic information.
 */
pub struct BlueprintTable {
    blueprints: HashMap<i32, ClassBlueprint>
}

impl BlueprintTable {
    pub fn default() -> Self {
        Self {
            blueprints: HashMap::<i32, ClassBlueprint>::default(),
        }
    }

    pub fn try_get_entry_mut(&mut self, class_id: i32) -> Option<&mut ClassBlueprint> {
        self.blueprints.get_mut(&class_id)
    }

    pub fn try_set_entry(&mut self, class_id: i32, blueprint: ClassBlueprint) -> bool {
        self.blueprints.insert(class_id, blueprint).is_none()
    }
}
