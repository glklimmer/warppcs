use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Mappable)]
pub fn derive_mappable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let data_enum = match ast.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new_spanned(name, "Mappable can only be derived on enums")
                .to_compile_error()
                .into();
        }
    };

    let mut as_index_arms = Vec::new();
    let mut count_expr_parts = Vec::new();
    let mut all_variants_parts = Vec::new();
    let mut current_base_index = quote! { 0 };

    for variant in data_enum.variants.iter() {
        let variant_ident = &variant.ident;

        match &variant.fields {
            Fields::Unit => {
                as_index_arms.push(quote! {
                    #name::#variant_ident => #current_base_index
                });
                all_variants_parts.push(quote! {
                    variants.push(#name::#variant_ident);
                });
                count_expr_parts.push(quote! { 1 });
                current_base_index = quote! { #current_base_index + 1 };
            }
            Fields::Named(fields) => {
                if fields.named.len() == 1 {
                    let field = fields.named.first().unwrap();
                    let field_name = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;

                    as_index_arms.push(quote! {
                        #name::#variant_ident { #field_name } => #current_base_index + #field_name.as_index()
                    });
                    all_variants_parts.push(quote! {
                        for &inner_variant in <#field_ty as EnumIter>::all_variants() {
                            variants.push(#name::#variant_ident { #field_name: inner_variant });
                        }
                    });
                    count_expr_parts.push(quote! { <#field_ty as EnumIter>::COUNT });
                    current_base_index = quote! { #current_base_index + <#field_ty as EnumIter>::COUNT };
                } else {
                    return syn::Error::new_spanned(
                        fields,
                        "Mappable on enums with fields currently only supports one field per variant",
                    )
                    .to_compile_error()
                    .into();
                }
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let field = fields.unnamed.first().unwrap();
                    let field_ty = &field.ty;

                    as_index_arms.push(quote! {
                        #name::#variant_ident(inner) => #current_base_index + inner.as_index()
                    });
                    all_variants_parts.push(quote! {
                        for &inner_variant in <#field_ty as EnumIter>::all_variants() {
                            variants.push(#name::#variant_ident(inner_variant));
                        }
                    });
                    count_expr_parts.push(quote! { <#field_ty as EnumIter>::COUNT });
                    current_base_index = quote! { #current_base_index + <#field_ty as EnumIter>::COUNT };
                } else {
                    return syn::Error::new_spanned(
                        fields,
                        "Mappable on enums with fields currently only supports one field per variant",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    let count_expr = quote! {
        0 #(+ #count_expr_parts)*
    };

    let all_variants_body = quote! {
        let mut variants = Vec::new();
        #(#all_variants_parts)*
        variants
    };

    let expanded = quote! {
        impl EnumIter for #name {
            const COUNT: usize = #count_expr;

            fn all_variants() -> &'static [Self] {
                use std::sync::Once;

                static mut VARIANTS: &'static [#name] = &[];
                static ONCE: Once = Once::new();

                ONCE.call_once(|| {
                    let variants = { #all_variants_body };
                    unsafe {
                        VARIANTS = Box::leak(variants.into_boxed_slice());
                    }
                });

                unsafe { VARIANTS }
            }

            fn as_index(&self) -> usize {
                match *self {
                    #(#as_index_arms),*
                }
            }
        }
    };

    expanded.into()
}
