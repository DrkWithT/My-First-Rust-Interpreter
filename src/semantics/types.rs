#[repr(i32)]
#[derive(Clone)]
pub enum PrimitiveTag {
    Unknown,
    Boolean,
    Integer,
    Floating
}

#[repr(i32)]
#[derive(Clone, PartialEq)]
pub enum OperatorTag {
    Noop,
    Access,
    Call,
    Negate,
    Increment,
    Decrement,
    Times,
    Slash,
    Plus,
    Minus,
    Equality,
    Inequality,
    LessThan,
    GreaterThan,
    Assign
}

impl OperatorTag {
    pub fn arity(&self) -> i32 {
        match self {
            Self::Noop => 0,
            Self::Access => 2,
            Self::Call => 1,
            Self::Negate => 1,
            Self::Increment => 1,
            Self::Decrement => 1,
            Self::Times => 2,
            Self::Slash => 2,
            Self::Plus => 2,
            Self::Minus => 2,
            Self::Equality => 2,
            Self::Inequality => 2,
            Self::LessThan => 2,
            Self::GreaterThan => 2,
            Self::Assign => 2,
        }
    }
}

pub trait TypeKind {
    fn is_primitive(&self) -> bool;
    fn is_sequence(&self) -> bool;
    fn is_callable(&self) -> bool;
    fn typename(&self) -> String;
}

pub struct PrimitiveInfo {
    tag: PrimitiveTag
}

impl PrimitiveInfo {
    pub fn new(tag: PrimitiveTag) -> Self {
        Self { tag }
    }
}

impl TypeKind for PrimitiveInfo {
    fn is_primitive(&self) -> bool {
        true
    }

    fn is_sequence(&self) -> bool {
        false
    }

    fn is_callable(&self) -> bool {
        false
    }

    fn typename(&self) -> String {
        let temp_tag = self.tag.clone();

        match temp_tag {
            PrimitiveTag::Boolean => String::from("bool"),
            PrimitiveTag::Integer => String::from("int"),
            PrimitiveTag::Floating => String::from("float"),
            _ => String::from("any")
        }
    }
}

pub struct ArrayInfo {
    item: Box<dyn TypeKind>,
    n: usize
}

impl ArrayInfo {
    pub fn new(item: Box<dyn TypeKind>, n: usize) -> Self {
        Self { item, n }
    }
}

impl TypeKind for ArrayInfo {
    fn is_primitive(&self) -> bool {
        false
    }

    fn is_sequence(&self) -> bool {
        true
    }

    fn is_callable(&self) -> bool {
        false
    }

    fn typename(&self) -> String {
        format!("[ {} : {} ]", self.n, self.item.typename())
    }
}
