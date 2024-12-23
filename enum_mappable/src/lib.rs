use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

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

    let mut array_elems = Vec::new();
    let mut match_arms = Vec::new();

    for (index, variant) in data_enum.variants.iter().enumerate() {
        let variant_ident = &variant.ident;

        array_elems.push(quote! {
            #name::#variant_ident
        });

        match_arms.push(quote! {
            #name::#variant_ident => #index
        });
    }

    // 3) The total number of variants
    let len = array_elems.len();

    // 4) Build the final expanded code
    let expanded = quote! {
        impl #name {
            pub const ALL: [#name; #len] = [
                #(#array_elems),*
            ];
        }

        impl EnumIter for #name {
            const COUNT: usize = #len;

            fn all_variants() -> &'static [Self] {
                &Self::ALL
            }

            fn as_index(&self) -> usize {
                match *self {
                    #(#match_arms),*
                }
            }
        }
    };

    expanded.into()
}
