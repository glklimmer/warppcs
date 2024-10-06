use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

#[proc_macro_attribute]
pub fn enum_as_f32(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    let enum_ident = &input.ident;
    let variants = input.variants.iter().enumerate().map(|(i, v)| {
        let ident = &v.ident;
        let index = i as f32;
        quote! {
            #enum_ident::#ident => #index,
        }
    });

    let gen = quote! {
        #input

        impl #enum_ident {
            pub fn as_f32(&self) -> f32 {
                match self {
                    #(#variants)*
                }
            }
        }
    };
    gen.into()
}
