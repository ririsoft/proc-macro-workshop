use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match input.data {
        Data::Struct(data) => derive_struct(&input.ident, &data),
        _ => unimplemented!(),
    };

    //eprintln!("TOKEN: {}", expanded);
    expanded.into()
}

fn derive_struct(ident: &Ident, data: &DataStruct) -> proc_macro2::TokenStream {
    match data.fields {
        Fields::Named(ref fields) => derive_struct_named(ident, fields),
        _ => unimplemented!(),
    }
}

fn derive_struct_named(ident: &Ident, fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let builder_ident = format_ident!("{}Builder", ident);

    let builder = struct_named_struct_builder(ident, &builder_ident, fields);
    let impl_struct = struct_name_impl_struct(ident, &builder_ident, fields);
    quote! {
        #builder
        #impl_struct
    }
}

fn struct_named_struct_builder(
    ident: &Ident,
    builder_ident: &Ident,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let struct_def = struct_named_struct_builder_def(builder_ident, fields);
    let struct_impl_setters = struct_named_struct_builder_impl_setters(builder_ident, fields);
    let struct_impl_build = struct_named_struct_builder_impl_build(ident, builder_ident, fields);

    quote! {
        #struct_def
        #struct_impl_setters
        #struct_impl_build
    }
}

fn struct_named_struct_builder_def(
    builder_ident: &Ident,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let fields_def = fields.named.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if extract_inner_type(&f.ty, "Option").is_some() {
            quote!(#name: #ty)
        } else {
            quote!(#name: ::std::option::Option<#ty>)
        }
    });

    quote! {
        pub struct #builder_ident {
            #(#fields_def),*
        }
    }
}

fn struct_named_struct_builder_impl_setters(
    builder_ident: &Ident,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let setters_def = fields.named.iter().map(|f| {
        let name = &f.ident;
        let ty = match extract_inner_type(&f.ty, "Option") {
            Some(inner_ty) => inner_ty,
            None => &f.ty,
        };
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    quote! {
        impl #builder_ident {
            #(#setters_def)*
        }
    }
}

fn struct_named_struct_builder_impl_build(
    ident: &Ident,
    builder_ident: &Ident,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let setters_def = fields.named.iter().map(|f| {
        let name = &f.ident.as_ref().unwrap();
        let error = format!("missing {}", name);
        if extract_inner_type(&f.ty, "Option").is_some() {
            quote!(#name: self.#name.clone())
        } else {
            quote!(#name: self.#name.clone().ok_or(#error)?)
        }
    });

    quote! {
        impl #builder_ident {
            pub fn build(&mut self) -> ::std::result::Result<#ident, ::std::boxed::Box<dyn ::std::error::Error>> {
                Ok(#ident{
                    #(#setters_def),*
                })
            }
        }
    }
}

fn struct_name_impl_struct(
    ident: &Ident,
    builder_ident: &Ident,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let fields_val = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });

    quote! {
        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#fields_val),*
                }
            }
        }
    }
}

fn extract_inner_type<'t>(ty: &'t syn::Type, expected_ident: &str) -> Option<&'t syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    }) = ty
    {
        if let std::option::Option::Some(syn::PathSegment {
            ident,
            arguments:
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }),
        }) = segments.last()
        {
            if ident == expected_ident {
                if let std::option::Option::Some(syn::GenericArgument::Type(ty)) = args.last() {
                    return std::option::Option::Some(ty);
                }
            }
        }
    }
    None
}
