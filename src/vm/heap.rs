use std::collections::VecDeque;

use crate::vm::value::Value;

const BASE_STRING_OVERHEAD: usize = 24;
const PRESET_STRING_CONTENT_OVERHEAD: usize = 26;
pub const TOTAL_STRING_OVERHEAD: usize = BASE_STRING_OVERHEAD + PRESET_STRING_CONTENT_OVERHEAD;
const MAX_HEAP_OVERHEAD: usize = i16::MAX as usize * TOTAL_STRING_OVERHEAD;
const DUD_OVERHEAD: usize = 1;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum ObjectTag {
    None,
    Varchar,
    // Array,
    Instance,
}

#[derive(Clone)]
pub enum HeapValue {
    Empty(),
    Varchar(String),
    // Array(Vec<Value>),
    Instance(Vec<Value>),
}

impl Default for HeapValue {
    fn default() -> Self {
        Self::Empty()
    }
}

impl HeapValue {
    pub fn get_object_tag(&self) -> ObjectTag {
        match self {
            Self::Empty() => ObjectTag::None,
            Self::Varchar(_) => ObjectTag::Varchar,
            Self::Instance(_) => ObjectTag::Instance,
        }
    }

    pub fn get_overhead(&self) -> usize {
        match self {
            Self::Varchar(s) => BASE_STRING_OVERHEAD + s.len(),
            _ => DUD_OVERHEAD,
        }
    }

    pub fn try_varchar_view(&self) -> Option<&str> {
        if let Self::Varchar(s) = self {
            return Some(s.as_str());
        }

        None
    }

    pub fn try_varchar_len(&self) -> i32 {
        if let Self::Varchar(s) = self {
            return s.len() as i32;
        }

        -1
    }

    pub fn try_varchar_get(&self, pos: i32) -> u8 {
        if let Self::Varchar(s) = self {
            if s.is_ascii() && pos >= 0 && pos < s.len() as i32 {
                return s.as_bytes()[pos as usize];
            }
        }

        0
    }

    pub fn try_varchar_set(&mut self, pos: i32, c: char) -> bool {
        if let Self::Varchar(s) = self {
            if s.is_ascii() {
                unsafe {
                   s.as_bytes_mut()[pos as usize] = c as u8;
                }
               return true;
            }
        }

        false
    }

    pub fn try_varchar_push(&mut self, c: char) -> bool {
        if let Self::Varchar(s) = self {
            if s.is_ascii() {
                s.push(c);
                return true
            }
        }

        false
    }

    pub fn try_varchar_pop(&mut self) -> u8 {
        if let Self::Varchar(s) = self {
            if s.is_ascii() {
                let ascii_c = s.pop().unwrap_or('\0');
                return ascii_c as u8;
            }
        }

        0
    }

    pub fn try_ref_instance_field(&self, field_pos: i32) -> Option<&Value> {
        if let Self::Instance(fields) = self {
            return fields.get(field_pos as usize);
        }

        None
    }

    pub fn try_ref_instance_field_mut(&mut self, field_pos: i32) -> Option<&mut Value> {
        if let Self::Instance(fields) = self {
            return fields.get_mut(field_pos as usize);
        }

        None
    }
}

#[derive(Clone)]
pub struct HeapCell {
    value: HeapValue,
    ref_count: i16,
}

impl HeapCell {
    pub fn new(value_arg: HeapValue) -> Self {
        Self {
            value: value_arg,
            ref_count: 0,
        }
    }

    pub fn is_live(&self) -> bool {
        self.ref_count > 0
    }

    pub fn inc_rc(&mut self) {
        self.ref_count += 1;
    }

    pub fn dec_rc(&mut self) {
        self.ref_count -= 1;
    }

    pub fn get_value(&self) -> &HeapValue {
        &self.value
    }

    pub fn get_value_mut(&mut self) -> &mut HeapValue {
        &mut self.value
    }
}

pub struct ObjectHeap {
    free_list: VecDeque<i16>,
    entries: Vec<HeapCell>,
    overhead_limit: usize,
    overhead: usize,
    slot_limit: i16,
    next_id: i16,
}

impl ObjectHeap {
    pub fn new(max_overhead: usize) -> Self {
        let checked_max_overhead: usize = if max_overhead <= MAX_HEAP_OVERHEAD { max_overhead } else { MAX_HEAP_OVERHEAD };
        let calculated_slot_n = 1 + checked_max_overhead / TOTAL_STRING_OVERHEAD;

        Self {
            free_list: VecDeque::<i16>::new(),
            entries: Vec::<HeapCell>::with_capacity(calculated_slot_n),
            overhead_limit: max_overhead,
            overhead: 0,
            slot_limit: calculated_slot_n as i16,
            next_id: 0,
        }
    }

    pub fn is_ripe_for_sweep(&self) -> bool {
        self.overhead > self.overhead_limit
    }

    pub fn preload_cell_at(&mut self, target_id: i16, value: HeapValue) -> bool {
        if let Some(target_ref) = self.entries.get_mut(target_id as usize) {
            *target_ref.get_value_mut() = value;
            return true;
        }

        false
    }

    pub fn get_cell(&self, id: i16) -> Option<&HeapCell> {
        self.entries.get(id as usize)
    }

    pub fn get_cell_mut(&mut self, id: i16) -> Option<&mut HeapCell> {
        self.entries.get_mut(id as usize)
    }

    pub fn try_create_cell(&mut self, tag: ObjectTag) -> i16 {
        let next_free_slot_opt = self.free_list.pop_front();
        let mut has_reclaimed_slot = false;

        let created_slot_id = if let Some(next_free_id) = next_free_slot_opt {
            has_reclaimed_slot = true;
            next_free_id
        } else if self.next_id <= self.slot_limit {
            has_reclaimed_slot = false;
            self.next_id += 1; self.next_id
        } else { -1 };

        if created_slot_id != -1 {
            #[allow(clippy::single_match)]
            match tag {
                ObjectTag::Varchar => {
                    let temp = HeapValue::Varchar(String::new());
                    let temp_size = temp.get_overhead();

                    if !has_reclaimed_slot {
                        self.entries.push(HeapCell::new(temp));
                    } else {
                        *self.entries.get_mut(created_slot_id as usize).unwrap() = HeapCell::new(temp);
                    }
                    self.overhead += temp_size;
                },
                _ => {},
            }
        }

        created_slot_id
    }

    pub fn try_collect_cell(&mut self, id: i16) {
        if self.free_list.binary_search(&id).is_ok() || id >= self.next_id {
            return;
        }

        if self.entries.get_mut(id as usize).unwrap().is_live() {
            return;
        }

        let dud_cell = HeapCell::new(HeapValue::Empty());
        let overhead_dec_n = self.entries.get_mut(id as usize).unwrap().get_value_mut().get_overhead();
        self.overhead -= overhead_dec_n;

        *self.entries.get_mut(id as usize).unwrap() = dud_cell;

        if id == self.next_id - 1 {
            self.next_id -= 1;
        }

        self.free_list.push_front(id);
    }

    pub fn force_collect_all(&mut self) {
        self.entries.clear();
        self.free_list.clear();
        self.overhead = 0;
        self.next_id = 0;
    }
}
