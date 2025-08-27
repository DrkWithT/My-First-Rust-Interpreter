use std::collections::HashMap;

#[derive(Default)]
pub struct ClassLayout {
    members: HashMap<String, i32>,
    /// Contains mappings of method names to these entries: `(<class-method-id>, <top-function-id>)`
    method_table: HashMap<String, (i32, i32)>,
}

impl ClassLayout {
    pub fn add_member(&mut self, name: String) -> bool {
        let next_member_id = self.members.len() as i32;
        self.members.insert(name, next_member_id).is_none()
    }

    pub fn get_member_id(&self, name: String) -> Option<i32> {
        if let Some(member_id) = self.members.get(&name) {
            return Some(*member_id);
        }

        None
    }

    pub fn add_method_id(&mut self, name: String, real_fun_id: i32) -> bool {
        let next_method_id = self.method_table.len() as i32;

        self.method_table.insert(name, (next_method_id, real_fun_id)).is_none()
    }

    pub fn get_real_method_id(&self, name: String) -> Option<(i32, i32)> {
        if let Some(method_loc) = self.method_table.get(&name) {
            return Some(*method_loc);
        }

        None
    }
}

pub type LayoutTable = HashMap<String, ClassLayout>;
