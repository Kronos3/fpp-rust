use crate::semantics::{
    AbsTypeValue, AnonArrayValue, AnonStructValue, ArrayValue, BooleanValue, EnumConstantValue,
    FloatValue, Format, IntegerValue, PrimitiveIntegerValue, StringValue, StructValue, Value,
};
use fpp_ast::{FloatKind, IntegerKind};
use fpp_core::Diagnostic;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Write};
use std::num::NonZeroU32;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Type {
    PrimitiveInt(IntegerKind),
    Float(FloatKind),
    String(Option<fpp_ast::Expr>),
    Boolean,
    /** The type of arbitrary-width integers */
    Integer,
    AbsType(AbsType),
    AliasType(AliasType),
    Array(ArrayType),
    AnonArray(AnonArrayType),
    Enum(EnumType),
    Struct(StructType),
    AnonStruct(AnonStructType),
}

impl Type {
    pub fn underlying_type(ty: &Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
        match ty.borrow().deref() {
            Type::AliasType(alias) => Type::underlying_type(&alias.alias_type),
            _ => ty.clone(),
        }
    }

    /** Get the default value */
    pub fn default_value(&self) -> Option<Value> {
        match self {
            Type::PrimitiveInt(kind) => Some(Value::PrimitiveInteger(PrimitiveIntegerValue {
                value: 0,
                kind: kind.clone(),
            })),
            Type::Float(kind) => Some(Value::Float(FloatValue {
                value: 0.0,
                kind: kind.clone(),
            })),
            Type::String(_) => Some(Value::String(StringValue("".to_string()))),
            Type::Boolean => Some(Value::Boolean(BooleanValue(false))),
            Type::Integer => Some(Value::Integer(IntegerValue(0))),
            Type::AliasType(ty) => ty.alias_type.borrow().default_value().clone(),
            Type::AbsType(ty) => Some(Value::AbsType(ty.default_value.clone()?)),
            Type::Array(array) => Some(Value::Array(array.default.clone()?)),
            Type::AnonArray(arr) => Some(Value::AnonArray(AnonArrayValue {
                elements: std::iter::repeat_n(
                    arr.elt_type.borrow().default_value()?,
                    arr.size?.get() as usize,
                )
                .collect(),
            })),
            Type::Enum(ty) => Some(Value::EnumConstant(ty.default.clone()?)),
            Type::Struct(def) => Some(Value::Struct(def.default.clone()?)),
            Type::AnonStruct(struct_) => {
                let mut members = vec![];
                for (name, ty) in &struct_.members {
                    members.push((name.clone(), ty.borrow().default_value()?))
                }

                Some(Value::AnonStruct(AnonStructValue {
                    members: HashMap::from_iter(members.into_iter()),
                }))
            }
        }
    }

    /** Get the array size */
    pub fn array_size(&self) -> Option<ArraySize> {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().array_size(),
            Type::AnonArray(arr) => arr.size,
            Type::Array(arr) => arr.anon_array.size,
            _ => None,
        }
    }

    /** Get the definition node identifier, if any */
    pub fn def_node_id(&self) -> Option<fpp_core::Node> {
        match self {
            Type::AbsType(ty) => Some(ty.node.node_id),
            Type::AliasType(ty) => Some(ty.node.node_id),
            Type::Array(ty) => Some(ty.node.node_id),
            Type::Enum(ty) => Some(ty.node.node_id),
            Type::Struct(ty) => Some(ty.node.node_id),
            _ => None,
        }
    }

    /** Does this type have numeric members? */
    pub fn has_numeric_members(&self) -> bool {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().has_numeric_members(),
            Type::Array(ty) => ty.anon_array.elt_type.borrow().has_numeric_members(),
            Type::AnonArray(ty) => ty.elt_type.borrow().has_numeric_members(),
            Type::Struct(ty) => ty
                .anon_struct
                .members
                .values()
                .all(|member| member.borrow().has_numeric_members()),
            Type::AnonStruct(ty) => ty
                .members
                .values()
                .all(|member| member.borrow().has_numeric_members()),
            _ => self.is_numeric(),
        }
    }

    /** Is this type convertible to a numeric type? */
    pub fn is_convertible_to_numeric(&self) -> bool {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().is_convertible_to_numeric(),
            Type::Enum(_) => true,
            _ => self.is_numeric(),
        }
    }

    /** Is this type promotable to an array type? */
    pub fn is_promotable_to_array(&self) -> bool {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().is_promotable_to_array(),
            Type::String(_) => true,
            Type::Boolean => true,
            Type::Enum(_) => true,
            _ => self.is_numeric(),
        }
    }

    /** Is this type displayable? */
    pub fn is_displayable(&self) -> bool {
        match self {
            Type::PrimitiveInt(_) => true,
            Type::Float(_) => true,
            Type::String(_) => true,
            Type::Boolean => true,
            Type::Integer => false,
            Type::AbsType(_) => false,
            Type::AliasType(alias) => alias.alias_type.borrow().is_displayable(),
            Type::Array(arr) => arr.anon_array.elt_type.borrow().is_displayable(),
            Type::AnonArray(arr) => arr.elt_type.borrow().is_displayable(),
            Type::Enum(_) => true,
            Type::Struct(ty) => ty
                .anon_struct
                .members
                .values()
                .all(|member| member.borrow().is_displayable()),
            Type::AnonStruct(ty) => ty
                .members
                .values()
                .all(|member| member.borrow().is_displayable()),
        }
    }

    /** Is this type a float type? */
    pub fn is_float(&self) -> bool {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().is_float(),
            Type::Float(_) => true,
            _ => false,
        }
    }

    /** Is this type an int type? */
    pub fn is_int(&self) -> bool {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().is_int(),
            Type::PrimitiveInt(_) => true,
            Type::Integer => true,
            _ => false,
        }
    }

    /** Is this type a primitive type? */
    pub fn is_primitive(&self) -> bool {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().is_primitive(),
            Type::PrimitiveInt(_) => true,
            Type::Float(_) => true,
            Type::Boolean => true,
            _ => false,
        }
    }

    /** Is this type a canonical (non-aliased) type? */
    pub fn is_canonical(&self) -> bool {
        match self {
            Type::AliasType(_) => false,
            _ => true,
        }
    }

    /** Is this type promotable to a struct type? */
    pub fn is_promotable_to_struct(&self) -> bool {
        self.is_promotable_to_array()
    }

    /** Is this type numeric? */
    pub fn is_numeric(&self) -> bool {
        match self {
            Type::AliasType(ty) => ty.alias_type.borrow().is_numeric(),
            _ => self.is_int() || self.is_float(),
        }
    }

    pub fn convert(from: &Rc<RefCell<Type>>, to: &Rc<RefCell<Type>>) -> TypeConversionResult {
        Type::convert_impl(
            Type::underlying_type(from).borrow().deref(),
            Type::underlying_type(to).borrow().deref(),
        )
    }

    fn convert_impl(from: &Type, to: &Type) -> TypeConversionResult {
        assert!(from.is_canonical());
        assert!(to.is_canonical());

        if Self::identical(from, to) {
            return Ok(());
        }

        if from.is_convertible_to_numeric() && to.is_numeric() {
            return Ok(());
        }

        match (from, to) {
            // String -> String
            (Type::String(_), Type::String(_)) => Ok(()),

            // Array -> Array
            (
                Type::Array(ArrayType {
                    anon_array: from_arr,
                    ..
                })
                | Type::AnonArray(from_arr),
                Type::Array(ArrayType {
                    anon_array: to_arr, ..
                })
                | Type::AnonArray(to_arr),
            ) => {
                // Check the sizes match
                match (&from_arr.size, &to_arr.size) {
                    (Some(from_size), Some(to_size)) => {
                        if from_size != to_size {
                            return Err(TypeConversionError::ArraySizeMismatch {
                                from: from_size.clone(),
                                to: to_size.clone(),
                            });
                        }
                    }
                    _ => {}
                }

                match Type::convert(&from_arr.elt_type, &to_arr.elt_type) {
                    Ok(()) => Ok(()),
                    Err(err) => Err(TypeConversionError::ArrayElement(Box::new(err))),
                }
            }

            // Convert a single element to an array
            (
                _,
                Type::Array(ArrayType {
                    anon_array: to_arr, ..
                })
                | Type::AnonArray(to_arr),
            ) => {
                if !from.is_promotable_to_array() {
                    return Err(TypeConversionError::NotPromotableToArray(from.clone()));
                }

                match Type::convert_impl(
                    from,
                    Type::underlying_type(&to_arr.elt_type).borrow().deref(),
                ) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(TypeConversionError::ArrayElement(Box::new(err))),
                }
            }

            // Struct -> Struct
            (
                Type::Struct(StructType {
                    anon_struct: from_struct,
                    ..
                })
                | Type::AnonStruct(from_struct),
                Type::Struct(StructType {
                    anon_struct: to_struct,
                    ..
                })
                | Type::AnonStruct(to_struct),
            ) => {
                // May sure all the members in 'from' can fit into 'to'
                for (name, from_member_ty) in &from_struct.members {
                    match to_struct.members.get(name) {
                        None => return Err(TypeConversionError::MissingStructMember(name.clone())),
                        Some(to_member_ty) => match Type::convert(from_member_ty, to_member_ty) {
                            Ok(_) => {}
                            Err(err) => {
                                return Err(TypeConversionError::StructMember {
                                    name: name.clone(),
                                    err: Box::new(err),
                                });
                            }
                        },
                    }
                }

                Ok(())
            }

            // Convert a single element to a struct
            (
                _,
                Type::Struct(StructType {
                    anon_struct: to_struct,
                    ..
                })
                | Type::AnonStruct(to_struct),
            ) => {
                if !from.is_promotable_to_struct() {
                    return Err(TypeConversionError::NotPromotableToStruct(from.clone()));
                }

                // Make sure that 'from' can fit all members in to_struct
                for (name, to_member_ty) in &to_struct.members {
                    let to_member_ty_underlying = Type::underlying_type(to_member_ty);
                    match Type::convert_impl(from, to_member_ty_underlying.borrow().deref()) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(TypeConversionError::StructMember {
                                name: name.clone(),
                                err: Box::new(err),
                            });
                        }
                    }
                }

                Ok(())
            }

            _ => Err(TypeConversionError::Mismatch {
                from: from.clone(),
                to: to.clone(),
            }),
        }
    }

    /** Check for type identity */
    pub fn identical(t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::PrimitiveInt(k1), Type::PrimitiveInt(k2)) => k1 == k2,
            (Type::Float(k1), Type::Float(k2)) => k1 == k2,
            (Type::Integer, Type::Integer) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::String(None), Type::String(None)) => true,
            (Type::String(Some(_e1)), Type::String(Some(_e2))) => {
                todo!("implement string size comparisons")
            }
            _ => match (t1.def_node_id(), t1.def_node_id()) {
                (Some(n1), Some(n2)) => n1 == n2,
                _ => false,
            },
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::PrimitiveInt(kind) => Debug::fmt(kind, f),
            Type::Float(kind) => Debug::fmt(kind, f),
            Type::String(_) => f.write_str("string"),
            Type::Boolean => f.write_str("boolean"),
            Type::Integer => f.write_str("Integer"),
            Type::AbsType(ty) => f.write_fmt(format_args!("{} (abstract type)", ty.node.name.data)),
            Type::AliasType(ty) => {
                f.write_fmt(format_args!("{} (", ty.node.name.data))?;
                Display::fmt(&Type::underlying_type(&ty.alias_type).borrow(), f)?;
                f.write_char(')')
            }
            Type::Array(arr) => f.write_fmt(format_args!("{} (array type)", arr.node.name.data)),
            Type::AnonArray(anon_arr) => {
                match anon_arr.size {
                    None => f.write_str("[] ")?,
                    Some(size) => f.write_fmt(format_args!("[{}] ", size))?,
                }

                anon_arr.fmt(f)
            }
            Type::Enum(ty) => f.write_fmt(format_args!("{} (enum type)", ty.node.name.data)),
            Type::Struct(ty) => f.write_fmt(format_args!("{} (struct type)", ty.node.name.data)),
            Type::AnonStruct(anon_struct) => {
                let mut s = f.debug_struct("anonymous struct");
                for (name, member_ty) in &anon_struct.members {
                    s.field(name, member_ty.borrow().deref());
                }

                s.finish()
            }
        }
    }
}

#[derive(Debug)]
pub enum TypeConversionError {
    ArraySizeMismatch {
        from: ArraySize,
        to: ArraySize,
    },
    ArrayElement(Box<TypeConversionError>),
    NotPromotableToArray(Type),
    NotPromotableToStruct(Type),
    MissingStructMember(String),
    StructMember {
        name: String,
        err: Box<TypeConversionError>,
    },
    Mismatch {
        from: Type,
        to: Type,
    },
}

impl TypeConversionError {
    pub fn annotate(&self, diagnostic: Diagnostic) -> Diagnostic {
        match self {
            TypeConversionError::ArraySizeMismatch { from, to } => {
                diagnostic.note(format!("array sizes do not match {} != {}", from, to))
            }
            TypeConversionError::ArrayElement(err) => {
                err.annotate(diagnostic.note("array element type cannot be converted:"))
            }
            TypeConversionError::NotPromotableToArray(ty) => {
                diagnostic.note(format!("{} cannot be promoted to array", ty))
            }
            TypeConversionError::NotPromotableToStruct(ty) => {
                diagnostic.note(format!("{} cannot be promoted to struct", ty))
            }
            TypeConversionError::MissingStructMember(name) => {
                diagnostic.note(format!("struct missing member `{}`", name))
            }
            TypeConversionError::StructMember { name, err } => err.annotate(diagnostic.note(
                format!("struct member `{}` type cannot be converted:", name),
            )),
            TypeConversionError::Mismatch { from, to } => {
                diagnostic.note(format!("{} cannot be converted to {}", from, to))
            }
        }
    }
}

pub type TypeConversionResult = Result<(), TypeConversionError>;

pub trait PrimitiveType {
    fn bit_width(&self) -> u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveIntSignedness {
    Signed,
    Unsigned,
}

pub fn int_kind_signedness(kind: IntegerKind) -> PrimitiveIntSignedness {
    match kind {
        IntegerKind::I8 | IntegerKind::I16 | IntegerKind::I32 | IntegerKind::I64 => {
            PrimitiveIntSignedness::Signed
        }
        IntegerKind::U8 | IntegerKind::U16 | IntegerKind::U32 | IntegerKind::U64 => {
            PrimitiveIntSignedness::Unsigned
        }
    }
}

impl PrimitiveType for IntegerKind {
    fn bit_width(&self) -> u32 {
        match self {
            IntegerKind::I8 => 8,
            IntegerKind::U8 => 8,
            IntegerKind::I16 => 16,
            IntegerKind::U16 => 16,
            IntegerKind::I32 => 32,
            IntegerKind::U32 => 32,
            IntegerKind::I64 => 64,
            IntegerKind::U64 => 64,
        }
    }
}

impl PrimitiveType for FloatKind {
    fn bit_width(&self) -> u32 {
        match self {
            FloatKind::F32 => 32,
            FloatKind::F64 => 64,
        }
    }
}

/** An abstract type */
#[derive(Debug, Clone)]
pub struct AbsType {
    /** The AST node giving the definition */
    pub node: fpp_ast::DefAbsType,
    pub default_value: Option<AbsTypeValue>,
}

/** An alias type */
#[derive(Debug, Clone)]
pub struct AliasType {
    /** The AST node giving the definition */
    pub node: fpp_ast::DefAliasType,
    /** Type that this typedef points to */
    pub alias_type: Rc<RefCell<Type>>,
}

/** A named array type */
#[derive(Debug, Clone)]
pub struct ArrayType {
    /** The AST node giving the definition */
    pub node: fpp_ast::DefArray,
    /** The structurally equivalent anonymous array */
    pub anon_array: AnonArrayType,
    /** The specified default value, if any */
    pub default: Option<ArrayValue>,
    /** The specified format, if any */
    pub format: Option<Format>,
}

type ArraySize = NonZeroU32;

/** An anonymous array type */
#[derive(Debug, Clone)]
pub struct AnonArrayType {
    /** The array size */
    pub size: Option<ArraySize>,
    /** The element type */
    pub elt_type: Rc<RefCell<Type>>,
}

/** An enum type */
#[derive(Debug, Clone)]
pub struct EnumType {
    /** The AST node giving the definition */
    pub node: fpp_ast::DefEnum,
    /** The representation type */
    pub rep_type: IntegerKind,
    /** The default value */
    pub default: Option<EnumConstantValue>,
}

/** A named struct type */
#[derive(Debug, Clone)]
pub struct StructType {
    /** The AST node giving the definition */
    pub node: fpp_ast::DefStruct,
    /** The structurally equivalent anonymous struct type */
    pub anon_struct: AnonStructType,
    /** The default value */
    pub default: Option<StructValue>,
    /** The member sizes */
    pub sizes: HashMap<String, u32>,
    /** The member formats */
    pub formats: HashMap<String, Format>,
}

/** An anonymous struct type */
#[derive(Debug, Clone)]
pub struct AnonStructType {
    /** The members */
    pub members: HashMap<String, Rc<RefCell<Type>>>,
}
