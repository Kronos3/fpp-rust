use proc_macro::TokenStream;
use quote::quote;
use std::any::Any;
use syn::{Fields, ItemEnum, ItemStruct, Type};

pub(crate) fn annotated_struct(input: &ItemStruct) -> TokenStream {
    // Ensure it's a named-field struct (not tuple or unit)
    let new_struct = input.clone();
    let name = &input.ident; // struct name

    let debug_fields = match &new_struct.fields {
        Fields::Named(fields_named) => {
            // Validate that the node_id field exists
            let node_id_type: Type = syn::parse_quote!(fpp_core::Node);
            match fields_named.named.iter().find(|f| match &f.ident {
                None => false,
                Some(ident) => ident.to_string() == "node_id",
            }) {
                Some(node_id_field) => {
                    if node_id_field.ty.type_id() != node_id_type.type_id() {
                        let err = syn::Error::new_spanned(
                            node_id_field,
                            "node_id member is not fpp_core::Node",
                        )
                        .to_compile_error();
                        return err.into();
                    }
                }
                None => {
                    let err = syn::Error::new_spanned(
                        input,
                        "`annotated` must be placed after `ast_node`",
                    )
                    .to_compile_error();
                    return err.into();
                }
            }

            let field_names = fields_named.named.iter().map(|f| f.ident.as_ref().unwrap());
            quote! {
                let mut debug_struct = f.debug_struct(stringify!(#name));
                #(debug_struct.field(stringify!(#field_names), &self.#field_names);)*
                let pre = &self.pre_annotation();
                if !pre.is_empty() {
                    debug_struct.field("pre_annotation", pre);
                }

                let post = &self.post_annotation();
                if !post.is_empty() {
                    debug_struct.field("post_annotation", post);
                }
                debug_struct.finish()
            }
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
    };

    let struct_ident = &new_struct.ident;
    let generics = &new_struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        #new_struct

        impl #impl_generics fpp_core::Annotated for #struct_ident #ty_generics #where_clause {
            fn pre_annotation(&self) -> Vec<String> {
                self.node_id.pre_annotation()
            }
            fn post_annotation(&self) -> Vec<String> {
                self.node_id.post_annotation()
            }
        }

        impl #impl_generics std::fmt::Debug for #struct_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #debug_fields
            }
        }
    };

    output.into()
}

pub(crate) fn annotated_enum(input: &ItemEnum) -> TokenStream {
    let enum_ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Build match arms for variants that have exactly one unnamed field.
    let mut pre_arms = vec![];
    let mut post_arms = vec![];

    for v in &input.variants {
        let var_ident = &v.ident;
        match &v.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                pre_arms.push(quote! { #enum_ident::#var_ident(inner) => inner.pre_annotation(), });
                post_arms
                    .push(quote! { #enum_ident::#var_ident(inner) => inner.post_annotation(), });
            }
            _ => {
                // Return a compile_error if user applied macro on unsupported struct
                let err = syn::Error::new_spanned(
                    var_ident,
                    format!(
                        "Variant {:?} does not wrap a type implementing AstNode + Annotated",
                        var_ident
                    ),
                )
                .to_compile_error();
                return err.into();
            }
        }
    }

    let output = quote! {
        #input

        impl #impl_generics fpp_core::Annotated for #enum_ident #ty_generics #where_clause {
            fn pre_annotation(&self) -> Vec<String> {
                match self {
                    #( #pre_arms )*
                }
            }
            fn post_annotation(&self) -> Vec<String> {
                match self {
                    #( #post_arms )*
                }
            }
        }
    };

    output.into()
}
