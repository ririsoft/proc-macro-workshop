use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, Fields, Ident};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let expanded = match input.data {
        Data::Struct(data) => impl_debug(&input.ident, &data),
        _ => unimplemented!(),
    };

    //eprintln!("TOKEN: {}", expanded);
    expanded.into()
}

fn impl_debug(ident: &Ident, data: &DataStruct) -> TokenStream {
    let fields_dbg = match data.fields {
        Fields::Named(ref fields) => fields.named.iter().map(|f| {
            let ident = &f.ident;
            quote!(.field(stringify!(#ident), &self.#ident))
        }),
        _ => unimplemented!(),
    };

    quote!(
        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
                f.debug_struct(stringify!(#ident))
                    #(#fields_dbg)*
                    .finish()
            }
        }
    )
}
