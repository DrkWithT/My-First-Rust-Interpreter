#[repr(i32)]
#[derive(Clone, Copy)]
pub enum ObjectTag {
    Str,
    Array,
}

// #[derive(Clone, Copy)]
// pub struct HeapRef {
//     pub id: i32,
//     pub count: i32,
//     pub tag: ObjectTag,
// }

#[derive(Clone, Copy)]
pub enum Value {
    Empty(),
    Bool(bool),
    Int(i32),
    Float(f32),
    // Str(HeapRef),
    // Array(HeapRef),
}

impl From<Value> for bool {
    fn from(val: Value) -> Self {
        match val {
            Value::Bool(flag) => flag,
            _ => false,
        }
    }
}

impl From<Value> for i32 {
    fn from(val: Value) -> Self {
        match val {
            Value::Int(value) => value,
            _ => 0
        }
    }
}

impl From<Value> for f32 {
    fn from(val: Value) -> Self {
        match val {
            Value::Float(value) => value,
            _ => 0.0f32,
        }
    }
}

impl Value {
    pub fn get_type_code(&self) -> i32 {
        match self {
            Self::Empty() => 0,
            Self::Bool(_) => 1,
            Self::Int(_) => 2,
            Self::Float(_) => 3,
        }
    }

    pub fn check_type_code(&self, rhs: &Self) -> bool {
        self.get_type_code() == rhs.get_type_code()
    }

    pub fn test(&self) -> bool {
        match self {
            Self::Empty() => false,
            Self::Bool(value) => *value,
            Self::Int(value) => *value != 0,
            Self::Float(value) => *value != 0.0f32,
        }
    }

    pub fn negate(&mut self) {
        match self {
            Self::Int(value) => {
                *value = -*value;
            },
            Self::Float(value) => {
                *value = -*value;
            },
            _ => {},
        }
    }

    pub fn increment(&mut self) {
        match self {
            Self::Int(value) => {
                *value += 1;
            },
            Self::Float(value) => {
                *value += 1.0f32;
            },
            _ => {},
        }
    }

    pub fn decrement(&mut self) {
        match self {
            Self::Int(value) => {
                *value -= 1;
            },
            Self::Float(value) => {
                *value -= 1.0f32;
            },
            _ => {},
        }
    }

    pub fn is_equal(&self, rhs: &Self) -> bool {
        if !self.check_type_code(rhs) {
            return false;
        }

        match self {
            Self::Empty() => false,
            Self::Bool(value) => *value == (*rhs).into(),
            Self::Int(value) => *value == (*rhs).into(),
            Self::Float(value) => *value == (*rhs).into(),
        }
    }

    pub fn is_unequal(&self, rhs: &Self) -> bool {
        !self.is_equal(rhs)
    }

    pub fn is_lesser(&self, rhs: &Self) -> bool {
        if !self.check_type_code(rhs) {
            return false;
        }

        match self {
            Self::Int(value) => *value < (*rhs).into(),
            Self::Float(value) => *value < (*rhs).into(),
            _ => false,
        }
    }

    pub fn is_greater(&self, rhs: &Self) -> bool {
        if !self.check_type_code(rhs) {
            return false;
        }

        match self {
            Self::Int(value) => *value > (*rhs).into(),
            Self::Float(value) => *value > (*rhs).into(),
            _ => false,
        }
    }

    pub fn add(&self, rhs: &Self) -> Value {
        if !self.check_type_code(rhs) {
            return Value::Empty();
        }

        match self {
            Self::Int(value) => {
                let lhs_int = *value;
                let rhs_int: i32 = (*rhs).into();

                Value::Int(lhs_int + rhs_int)
            },
            Self::Float(value) => {
                let lhs_float = *value;
                let rhs_float: f32 = (*rhs).into();

                Value::Float(lhs_float + rhs_float)
            },
            _ => Value::Empty()
        }
    }

    pub fn sub(&self, rhs: &Self) -> Value {
        if !self.check_type_code(rhs) {
            return Value::Empty();
        }

        match self {
            Self::Int(value) => {
                let lhs_int = *value;
                let rhs_int: i32 = (*rhs).into();

                Value::Int(lhs_int - rhs_int)
            },
            Self::Float(value) => {
                let lhs_float = *value;
                let rhs_float: f32 = (*rhs).into();

                Value::Float(lhs_float - rhs_float)
            },
            _ => Value::Empty()
        }
    }

    pub fn mul(&self, rhs: &Self) -> Value {
        if !self.check_type_code(rhs) {
            return Value::Empty();
        }

        match self {
            Self::Int(value) => {
                let lhs_int = *value;
                let rhs_int: i32 = (*rhs).into();

                Value::Int(lhs_int * rhs_int)
            },
            Self::Float(value) => {
                let lhs_float = *value;
                let rhs_float: f32 = (*rhs).into();

                Value::Float(lhs_float * rhs_float)
            },
            _ => Value::Empty()
        }
    }

    /// NOTE: Division is checked for illegal operations such as returning `Value::Empty`` on `1 / 0`.
    pub fn div(&self, rhs: &Self) -> Value {
        if !self.check_type_code(rhs) {
            return Value::Empty();
        }

        if !rhs.test() {
            return Value::Empty();
        }

        match self {
            Self::Int(value) => {
                let lhs_int = *value;
                let rhs_int: i32 = (*rhs).into();

                Value::Int(lhs_int / rhs_int)
            },
            Self::Float(value) => {
                let lhs_float = *value;
                let rhs_float: f32 = (*rhs).into();

                Value::Float(lhs_float / rhs_float)
            },
            _ => Value::Empty()
        }
    }
}
