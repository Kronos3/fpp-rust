mod annotated;
mod node;

use crate::annotated::{annotated_enum, annotated_struct};
use crate::node::{ast_node_enum, ast_node_struct};
use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};

///
/// Defines an AstNode from struct or enum
///
/// For structs, an additional field is added and traits are implemented.
///
/// The following checks are performed on structs:
/// 1. No field in the struct definition is named 'node_id'
/// 2. All fields are 'pub'
///
/// Enums require all variants to be AstNodes
///
/// # Examples
///
/// For structures:
/// ```ignore
/// use fpp_macros::ast_node;
///
/// #[ast_node]
/// pub struct TlmChannelIdentifier {
///    pub component_instance: QualIdent,
///    pub channel_name: Ident,
/// }
/// ```
///
/// For enums:
/// ```ignore
/// use fpp_macros::ast;
///
/// #[ast]
/// pub enum InterfaceMember {
///     SpecPortInstance(SpecPortInstance),
///     SpecImport(SpecImport),
/// }
/// ```
#[proc_macro_attribute]
pub fn ast(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let item = parse_macro_input!(input);

    match &item {
        Item::Struct(item_struct) => ast_node_struct(&item_struct),
        Item::Enum(item_enum) => ast_node_enum(&item_enum),
        other => {
            let err = syn::Error::new_spanned(
                other,
                "#[ast_node] #[derive(AstAnnotated)] only supports structs or enums",
            )
            .to_compile_error();
            err.into()
        }
    }
}


///
/// Derives wrapper trait for accessing ast node annotation which are
/// interned in the compiler context.
///
/// The following pre-requisites are checked:
/// 1. Struct or Enum
/// 2. Already inherits #[ast]
/// 3. If enum, all variants also derive from AstAnnotated
///
/// For structs
/// ```ignore
/// #[ast]
/// #[derive(AstAnnotated)]
/// pub struct SpecStateMachineInstance {
///     pub name: Ident,
///     pub state_machine: QualIdent,
///     pub priority: Option<Expr>,
///     pub queue_full: Option<QueueFull>,
/// }
/// ```
///
/// For enums:
/// ```ignore
/// use fpp_macros::ast_node;
///
/// #[ast]
/// #[derive(AstAnnotated)]
/// pub enum InterfaceMember {
///     SpecPortInstance(SpecPortInstance),
///     SpecImport(SpecImport),
/// }
/// ```
#[proc_macro_derive(AstAnnotated)]
pub fn ast_annotated(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let item = parse_macro_input!(input);

    (match item {
        Item::Struct(item_struct) => annotated_struct(&item_struct),
        Item::Enum(item_enum) => annotated_enum(&item_enum),
        other => {
            let err = syn::Error::new_spanned(other, "#[ast_node] only supports structs or enums")
                .to_compile_error();
            return err.into();
        }
    })
    .into()
}
