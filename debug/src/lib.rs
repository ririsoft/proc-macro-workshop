use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, Error, Fields, GenericParam, Generics, Ident,
    Lit, Meta, MetaNameValue,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let expanded = match input.data {
        Data::Struct(data) => impl_debug(&input.ident, &input.generics, &data),
        _ => unimplemented!(),
    };

    //eprintln!("TOKEN: {}", expanded);
    expanded.into()
}

fn impl_debug(ident: &Ident, generics: &Generics, data: &DataStruct) -> TokenStream {
    let fields_dbg = match data.fields {
        Fields::Named(ref fields) => fields.named.iter().map(|f| {
            let ident = &f.ident;
            match get_fmt_attr(&f.attrs) {
                Ok(Some(fmt_attr)) => {
                    quote!(.field(stringify!(#ident), &format_args!(#fmt_attr, &self.#ident)))
                }
                Ok(None) => quote!(.field(stringify!(#ident), &self.#ident)),
                Err(err) => err,
            }
        }),
        _ => unimplemented!(),
    };

    let impl_debug_header = if generics.params.is_empty() {
        quote!(impl ::std::fmt::Debug for #ident)
    } else {
        impl_debug_header_generic(ident, generics)
    };

    quote!(
        #impl_debug_header {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
                f.debug_struct(stringify!(#ident))
                    #(#fields_dbg)*
                    .finish()
            }
        }
    )
}

fn impl_debug_header_generic(ident: &Ident, generics: &Generics) -> TokenStream {
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    let where_clauses = generics.params.iter().filter_map(|p| match p {
        GenericParam::Type(ty) => Some(quote!(#ty: ::std::fmt::Debug)),
        _ => None,
    });

    quote!(
        impl #impl_generics ::std::fmt::Debug for #ident #ty_generics where #(#where_clauses),*
    )
}

fn get_fmt_attr(attrs: &[Attribute]) -> Result<Option<String>, TokenStream> {
    for attr in attrs {
        match attr.parse_meta() {
            Ok(ref meta) => match meta {
                Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                    if path.is_ident("debug") {
                        match lit {
                            Lit::Str(lit) => return Ok(Some(lit.value())),
                            _ => unimplemented!(),
                        }
                    } else {
                        return std::result::Result::Err(
                            Error::new_spanned(meta, "expected `debug \"...\"`").to_compile_error(),
                        );
                    }
                }
                Meta::List(_) => unimplemented!(),
                Meta::Path(_) => unimplemented!(),
            },
            Err(_) => unimplemented!(),
        }
    }
    Ok(None)
}
