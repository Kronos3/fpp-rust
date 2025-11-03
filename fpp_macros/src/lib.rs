mod annotated;
mod node;
mod visitable;

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

// decl_derive!(
//     [Walkable, attributes(visitable)] =>
//     /// Derives `Walkable` for the annotated `struct` or `enum` (`union` is not supported).
//     ///
//     /// Each field of the struct or enum variant will be visited in definition order, using the
//     /// `Walkable` implementation for its type. However, if a field of a struct or an enum
//     /// variant is annotated with `#[visitable(ignore)]` then that field will not be
//     /// visited (and its type is not required to implement `Walkable`).
//     visitable::visitable_derive
// );

#[proc_macro_derive(Walkable, attributes(visitable))]
/// Derives `Walkable` for the annotated `struct` or `enum` (`union` is not supported).
///
/// Each field of the struct or enum variant will be visited in definition order, using the
/// `Walkable` implementation for its type. However, if a field of a struct or an enum
/// variant is annotated with `#[visitable(ignore)]` then that field will not be
/// visited (and its type is not required to implement `Walkable`).
pub fn walk_derive(i: TokenStream) -> TokenStream {
    let clone_i = i.clone();
    let input = parse_macro_input!(i as syn::DeriveInput);

    // Check for `#[visitable(no_self)]`
    let mut no_self = false;
    for attr in &input.attrs {
        if attr.path().is_ident("visitable") {
            // Use `parse_nested_meta` to process inner items like (no_self)
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("no_self") {
                    no_self = true;
                }
                Ok(())
            });
        }
    }

    match ::synstructure::macros::parse::<::synstructure::macros::DeriveInput>(clone_i) {
        Ok(p) => match synstructure::Structure::try_new(&p) {
            Ok(s) => {
                synstructure::MacroResult::into_stream(visitable::visitable_derive(s, no_self))
            }
            Err(e) => ::core::convert::Into::into(e.to_compile_error()),
        },
        Err(e) => ::core::convert::Into::into(e.to_compile_error()),
    }
}
