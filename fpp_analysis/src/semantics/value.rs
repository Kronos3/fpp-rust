use std::cell::RefCell;
use crate::semantics::{AbsType, ArrayType, EnumType, StructType};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    PrimitiveInteger(PrimitiveIntegerValue),
    AbsType(AbsTypeValue),
    Integer(IntegerValue),
    Float(FloatValue),
    Boolean(BooleanValue),
    String(StringValue),
    EnumConstant(EnumConstantValue),
    AnonArray(AnonArrayValue),
    Array(ArrayValue),
    AnonStruct(AnonStructValue),
    Struct(StructValue),
}

impl Value {}

/** Primitive integer values */
#[derive(Debug, Clone)]
pub struct PrimitiveIntegerValue {
    pub value: i128,
    pub kind: fpp_ast::IntegerKind,
}

/** Integer values */
#[derive(Debug, Clone)]
pub struct IntegerValue(pub i128);

/** Floating-point values */
#[derive(Debug, Clone)]
pub struct FloatValue {
    pub value: f64,
    pub kind: fpp_ast::FloatKind,
}

/** Boolean values */
#[derive(Debug, Clone)]
pub struct BooleanValue(pub bool);

/** String values */
#[derive(Debug, Clone)]
pub struct StringValue(pub String);

/** Anonymous array values */
#[derive(Debug, Clone)]
pub struct AnonArrayValue {
    pub elements: Vec<Value>,
}

/** Array values */
#[derive(Debug, Clone)]
pub struct ArrayValue {
    pub anon_array: AnonArrayValue,
    pub ty: Rc<RefCell<ArrayType>>,
}

/** Enum constant values */
#[derive(Debug, Clone)]
pub struct EnumConstantValue {
    pub value: (String, i128),
    pub ty: Rc<RefCell<EnumType>>,
}

#[derive(Debug, Clone)]
pub struct AnonStructValue {
    pub members: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct StructValue {
    pub anon_struct: AnonStructValue,
    pub ty: Box<StructType>,
}

#[derive(Debug, Clone)]
pub struct AbsTypeValue {
    pub ty: Rc<RefCell<AbsType>>
}
