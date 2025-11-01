mod annotated;
mod node;

use crate::node::{ast_node_enum, ast_node_struct};
use proc_macro::TokenStream;
use syn::{Item, parse_macro_input};
use crate::annotated::{annotated_enum, annotated_struct};

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
/// ```
/// #[ast_node]
/// pub struct TlmChannelIdentifier {
///    pub component_instance: QualIdent,
///    pub channel_name: Ident,
/// }
/// ```
///
/// For enums:
/// ```
/// #[ast_node]
/// #[derive(Debug)]
/// pub enum InterfaceMember {
///     SpecPortInstance(SpecPortInstance),
///     SpecImport(SpecImport),
/// }
/// ```
#[proc_macro_attribute]
pub fn ast_node(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as Item);

    (match input {
        Item::Struct(item_struct) => ast_node_struct(&item_struct),
        Item::Enum(item_enum) => ast_node_enum(&item_enum),
        other => {
            let err = syn::Error::new_spanned(other, "#[ast_node] only supports structs or enums")
                .to_compile_error();
            return err.into();
        }
    })
    .into()
}

///
/// Implements Annotated trait for inserting and getting annotations associated with
/// an AST node.
///
/// Note: Annotated structs must first apply the ast_node macro
///
/// # Examples
///
/// ```
/// #[ast_node]
/// #[annotated]
/// #[derive(Debug)]
/// pub struct FormalParam {
///     pub kind: FormalParamKind,
///     pub name: Ident,
///     pub type_name: TypeName,
/// }
/// ```
#[proc_macro_attribute]
pub fn annotated(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as Item);

    (match input {
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
