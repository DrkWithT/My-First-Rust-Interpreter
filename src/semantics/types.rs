#[repr(i32)]
#[derive(Clone, Copy, PartialEq)]
pub enum PrimitiveTag {
    Unknown,
    Any,
    Boolean,
    Char,
    Integer,
    Floating,
    Varchar,
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq)]
pub enum ValueCategoryTag {
    /// undeclared thus invalid
    Unknown,

    /// possibly denotes type erasure
    Anything,

    /// name-bound values
    Identity,

    /// anonymous values
    Temporary,
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

    pub fn as_symbol(&self) -> &str {
        match self {
            Self::Noop => "(none)",
            Self::Access => ".",
            Self::Call => "(call)",
            Self::Negate => "- (negate)",
            Self::Increment => "++",
            Self::Decrement => "--",
            Self::Times => "*",
            Self::Slash => "/",
            Self::Plus => "+",
            Self::Minus => "- (subtract)",
            Self::Equality => "==",
            Self::Inequality => "!=",
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::Assign => "=",
        }
    }

    pub fn is_homogeneously_typed(&self) -> bool {
        matches!(self, Self::Noop | Self::Negate | Self::Times | Self::Slash | Self::Plus | Self::Minus | Self::Equality | Self::Inequality | Self::LessThan | Self::GreaterThan | Self::Assign)
    }

    pub fn is_value_group_sensitive(&self) -> bool {
        matches!(self, Self::Assign)
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
        let temp_tag = self.tag;

        match temp_tag {
            PrimitiveTag::Any => String::from("any"),
            PrimitiveTag::Boolean => String::from("bool"),
            PrimitiveTag::Char => String::from("char"),
            PrimitiveTag::Integer => String::from("int"),
            PrimitiveTag::Floating => String::from("float"),
            PrimitiveTag::Varchar => String::from("varchar"),
            _ => String::from("unknown"),
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
        format!("[{}:{}]", self.n, self.item.typename())
    }
}

pub struct FunctionInfo {
    inputs: Vec<Box<dyn TypeKind>>,
    output: Box<dyn TypeKind>,
}

impl FunctionInfo {
    pub fn new(input_types: Vec<Box<dyn TypeKind>>, output_type: Box<dyn TypeKind>) -> Self {
        Self {
            inputs: input_types,
            output: output_type,
        }
    }
}

impl TypeKind for FunctionInfo {
    fn is_primitive(&self) -> bool {
        false
    }

    fn is_sequence(&self) -> bool {
        false
    }

    fn is_callable(&self) -> bool {
        true
    }

    fn typename(&self) -> String {
        let mut result = String::new();

        result.push('(');

        for arg in &self.inputs {
            let arg_typename_string = arg.typename();
            let arg_typename: &str = arg_typename_string.as_str();
            result.push_str(arg_typename);
            result.push(' ');
        }

        result.push_str(") -> ");
        result.push_str(self.output.typename().as_str());

        result
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum ClassAccess {
    Public,
    Private,
}

pub struct ClassInfo {
    class_name: String,
}

impl ClassInfo {
    pub fn new(class_name_arg: String) -> Self {
        Self {
            class_name: class_name_arg
        }
    }
}

impl TypeKind for ClassInfo {
    fn is_primitive(&self) -> bool {
        false
    }

    fn is_sequence(&self) -> bool {
        false
    }

    fn is_callable(&self) -> bool {
        false
    }

    fn typename(&self) -> String {
        self.class_name.clone()
    }
}


