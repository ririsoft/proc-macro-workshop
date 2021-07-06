use proc_macro2::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _input = parse_macro_input!(input as syn::DeriveInput);

    //eprintln!("TOKEN: {}", expanded);
    TokenStream::new().into()
}
