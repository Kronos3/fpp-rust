use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct, Fields, Field, Type, Visibility};

///
/// Converts a struct into an AstNode by adding a public node_id field
/// and implementing the proper traits.
///
/// This macro applies the following checks:
/// 1. No field in the struct definition is named 'node_id'
/// 2. All fields are 'pub'
///
/// # Examples
///
/// ```
/// #[ast_node]
/// pub struct TlmChannelIdentifier {
///    pub component_instance: QualIdent,
///    pub channel_name: Ident,
/// }
/// ```
#[proc_macro_attribute]
pub fn ast_node(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemStruct);

    // Ensure it's a named-field struct (not tuple or unit)
    let mut new_struct = input.clone();
    match &mut new_struct.fields {
        Fields::Named(fields_named) => {
            for field in &fields_named.named {
                match &field.ident {
                    Some(ident) => {
                        if ident.to_string() == "node_id" {
                            let err = syn::Error::new_spanned(
                                field,
                                "ast_node reserves the 'node_id' field name",
                            )
                                .to_compile_error();
                            return err.into()
                        }
                    }
                    _ => {}
                }

                match field.vis {
                    Visibility::Public(_) => {}
                    _ => {
                        let err = syn::Error::new_spanned(
                            field,
                            "all members of ast_node must be 'pub'",
                        )
                            .to_compile_error();
                        return err.into()
                    }
                }
            }

            let node_id_field_ident = format_ident!("node_id");
            let node_id_field_type: Type = syn::parse_quote!(fpp_core::NodeId);
            let node_id_field = Field {
                attrs: Vec::new(),
                vis: syn::Visibility::Public(syn::parse_quote!(pub)),
                mutability: syn::FieldMutability::None,
                ident: Some(node_id_field_ident.clone()),
                colon_token: Some(<syn::Token![:]>::default()),
                ty: node_id_field_type,
            };
            fields_named.named.push(node_id_field);
        }
        Fields::Unit | Fields::Unnamed(_) => {
            // Return a compile_error if user applied macro on unsupported struct
            let err = syn::Error::new_spanned(
                input,
                "#[ast_node] only supports structs with named fields (braced structs).",
            )
                .to_compile_error();
            return err.into();
        }
    }

    let struct_ident = &new_struct.ident;
    let generics = &new_struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        #new_struct

        impl #impl_generics fpp_core::Spanned for #struct_ident #ty_generics #where_clause {
            fn span(&self) -> fpp_core::Span {
                fpp_core::Spanned::span(&self.node_id)
            }
        }
    };

    output.into()
}
