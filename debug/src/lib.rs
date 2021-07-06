use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, Error, Fields, GenericArgument, Generics,
    Ident, Lit, Meta, MetaNameValue, PathArguments, Type, TypePath,
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
        impl_debug_header_generic(ident, generics, data)
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

fn impl_debug_header_generic(ident: &Ident, generics: &Generics, data: &DataStruct) -> TokenStream {
    let phantoms: std::collections::HashSet<Ident> = data
        .fields
        .iter()
        .filter_map(|f| match &f.ty {
            Type::Path(ty_path) => match ty_path.path.segments.last() {
                Some(seg) if seg.ident == "PhantomData" => match &seg.arguments {
                    PathArguments::None => None,
                    PathArguments::AngleBracketed(args) => match args.args.first() {
                        Some(GenericArgument::Type(Type::Path(TypePath { path, .. }))) => {
                            match path.segments.last() {
                                Some(seg) => Some(seg.ident.clone()),
                                None => unreachable!(),
                            }
                        }
                        None => unreachable!(),
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                },
                Some(_) => None,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        })
        .collect();

    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    let where_clauses = generics.type_params().filter_map(|ty| {
        if phantoms.iter().any(|ident| ident == &ty.ident) {
            None
        } else {
            Some(quote!(#ty: ::std::fmt::Debug))
        }
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
