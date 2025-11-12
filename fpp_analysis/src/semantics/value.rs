use crate::semantics::{ArrayType, EnumType, StructType, Type};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
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

impl Value {
    fn is_promotable_to_aggregate(&self) -> bool {
        match self {
            Value::PrimitiveInteger(_) => true,
            Value::Integer(_) => true,
            Value::Float(_) => true,
            Value::Boolean(_) => true,
            Value::String(_) => true,
            Value::EnumConstant(_) => true,
            _ => false,
        }
    }

    pub fn convert(&self, ty_a: &Rc<RefCell<Type>>) -> Option<Value> {
        match (self.convert_impl(ty_a), self.is_promotable_to_aggregate()) {
            (Some(value), _) => Some(value),
            (None, true) => {
                // Try to promote this single value an array/struct
                // (if that's what we are trying to convert to)
                let ty = Type::underlying_type(ty_a);
                match ty.borrow().deref() {
                    Type::Array(array_ty) => {
                        let elt_value = self.convert(&array_ty.anon_array.elt_type)?;
                        let size = array_ty.anon_array.size?;
                        Some(Value::Array(ArrayValue {
                            anon_array: AnonArrayValue {
                                elements: std::iter::repeat_n(elt_value, size).collect(),
                            },
                            ty: ty.clone(),
                        }))
                    }
                    Type::AnonArray(array_ty) => {
                        let elt_value = self.convert(&array_ty.elt_type)?;
                        let size = array_ty.size?;
                        Some(Value::AnonArray(AnonArrayValue {
                            elements: std::iter::repeat_n(elt_value, size).collect(),
                        }))
                    }
                    Type::Struct(struct_ty) => {
                        let mut out_value = HashMap::new();
                        for (name, member_ty) in &struct_ty.anon_struct.members {
                            out_value.insert(name.clone(), self.clone().convert(member_ty)?);
                        }

                        Some(Value::Struct(StructValue {
                            anon_struct: AnonStructValue { members: out_value },
                            ty: ty.clone(),
                        }))
                    }
                    Type::AnonStruct(struct_ty) => {
                        let mut out_value = HashMap::new();
                        for (name, member_ty) in &struct_ty.members {
                            out_value.insert(name.clone(), self.clone().convert(member_ty)?);
                        }

                        Some(Value::Struct(StructValue {
                            anon_struct: AnonStructValue { members: out_value },
                            ty: ty.clone(),
                        }))
                    }
                    _ => None,
                }
            }
            (None, false) => None,
        }
    }

    fn convert_impl(&self, ty_a: &Rc<RefCell<Type>>) -> Option<Value> {
        let ty_underlying = Type::underlying_type(ty_a);
        let ty = ty_underlying.borrow();

        match &self {
            Value::PrimitiveInteger(PrimitiveIntegerValue { value: from, .. })
            | Value::Integer(IntegerValue(from)) => match ty.deref() {
                Type::PrimitiveInt(to_kind) => {
                    Some(Value::PrimitiveInteger(PrimitiveIntegerValue {
                        value: from.clone(),
                        kind: to_kind.clone(),
                    }))
                }
                Type::Float(to_kind) => Some(Value::Float(FloatValue {
                    value: from.clone() as f64,
                    kind: to_kind.clone(),
                })),
                Type::Integer => Some(Value::Integer(IntegerValue(from.clone()))),
                _ => None,
            },

            Value::Float(from) => match ty.deref() {
                Type::PrimitiveInt(to_kind) => {
                    Some(Value::PrimitiveInteger(PrimitiveIntegerValue {
                        value: from.value as i128,
                        kind: to_kind.clone(),
                    }))
                }
                Type::Float(to_kind) => Some(Value::Float(FloatValue {
                    value: from.value,
                    kind: to_kind.clone(),
                })),
                Type::Integer => Some(Value::Integer(IntegerValue(from.value as i128))),
                _ => None,
            },

            Value::Boolean(BooleanValue(_)) => match ty.deref() {
                Type::Boolean => Some(self.clone()),
                _ => None,
            },

            Value::String(StringValue(_)) => match ty.deref() {
                Type::String(_) => Some(self.clone()),
                _ => None,
            },

            // Values that have a type to a definition
            // Check if they are the same as the type we are trying to convert to
            Value::Array(ArrayValue { ty: from_ty, .. })
            | Value::Struct(StructValue { ty: from_ty, .. })
            | Value::EnumConstant(EnumConstantValue { ty: from_ty, .. })
            | Value::AbsType(AbsTypeValue { ty: from_ty })
                if Type::identical(&ty, &Type::underlying_type(from_ty).borrow()) =>
            {
                Some(self.clone())
            }

            // Enum -> Integer
            Value::EnumConstant(value) => {
                let from_ty = value.ty();
                Value::PrimitiveInteger(PrimitiveIntegerValue {
                    value: value.value.1,
                    kind: from_ty.rep_type.clone(),
                })
                .convert(ty_a)
            }

            Value::AnonArray(anon_array) | Value::Array(ArrayValue { anon_array, .. }) => {
                let elements = match ty.deref() {
                    Type::Array(ArrayType {
                        anon_array: anon_array_ty,
                        ..
                    })
                    | Type::AnonArray(anon_array_ty) => {
                        match (anon_array_ty.size, anon_array.elements.len()) {
                            (Some(ty_size), value_size) if ty_size == value_size => {
                                let elements = std::iter::repeat_n(
                                    self.convert(&anon_array_ty.elt_type)?,
                                    value_size,
                                )
                                .collect();
                                Some(elements)
                            }
                            (None, value_size) => {
                                let elements = std::iter::repeat_n(
                                    self.convert(&anon_array_ty.elt_type)?,
                                    value_size,
                                )
                                .collect();
                                Some(elements)
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }?;

                match ty.deref() {
                    Type::Array(_) => Some(Value::Array(ArrayValue {
                        anon_array: AnonArrayValue { elements },
                        ty: ty_underlying.clone(),
                    })),
                    Type::AnonArray(_) => Some(Value::AnonArray(AnonArrayValue { elements })),
                    _ => None,
                }
            }

            Value::AnonStruct(anon_struct) | Value::Struct(StructValue { anon_struct, .. }) => {
                let mut members = HashMap::new();

                let to_ty = match ty.deref() {
                    // TODO(tumbar) default values need to come from struct type
                    Type::Struct(StructType { anon_struct, .. }) => anon_struct,
                    Type::AnonStruct(anon_struct) => anon_struct,
                    _ => return None,
                };

                for (name, ty) in &to_ty.members {
                    let member_value = match anon_struct.members.get(name) {
                        // Use the default value
                        None => ty.borrow().default_value()?,
                        Some(member_value) => member_value.convert(&ty)?,
                    };

                    members.insert(name.clone(), member_value);
                }

                match ty.deref() {
                    Type::Struct(_) => Some(Value::Struct(StructValue {
                        anon_struct: AnonStructValue { members },
                        ty: ty_a.clone(),
                    })),
                    Type::AnonStruct(_) => Some(Value::AnonStruct(AnonStructValue { members })),
                    _ => None,
                }
            }

            _ => None,
        }
    }
}

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
    pub ty: Rc<RefCell<Type>>,
}

/** Enum constant values */
#[derive(Debug, Clone)]
pub struct EnumConstantValue {
    pub value: (String, i128),
    ty: Rc<RefCell<Type>>,
}

impl EnumConstantValue {
    pub fn new(member_name: String, value: i128, ty: Rc<RefCell<Type>>) -> EnumConstantValue {
        match ty.borrow().deref() {
            Type::Enum(_) => (),
            _ => {
                panic!("expected enum type")
            }
        }

        EnumConstantValue {
            value: (member_name, value),
            ty,
        }
    }

    pub fn ty(&'_ self) -> Ref<'_, EnumType> {
        Ref::map(self.ty.borrow(), |t| match t {
            Type::Enum(e_ty) => e_ty,
            _ => {
                panic!("expected enum type")
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct AnonStructValue {
    pub members: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct StructValue {
    pub anon_struct: AnonStructValue,
    ty: Rc<RefCell<Type>>,
}

#[derive(Debug, Clone)]
pub struct AbsTypeValue {
    ty: Rc<RefCell<Type>>,
}
