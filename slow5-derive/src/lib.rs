use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Ident};

#[proc_macro_derive(FieldExt)]
#[proc_macro_error]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let output = match input.data {
        Data::Struct(ds) => derive_header_init(&name, &ds),
        Data::Enum(_) | Data::Union(_) => {
            abort_call_site!("#[derive(FieldExt)] only derived for struct")
        }
    };

    let expanded = quote! {
        impl #name {
            pub fn with_data() -> Self {

            }
        }

        impl FieldExt for #name {
            fn set_header_aux_fields(header: &Header) -> Result<(), Box<dyn std::error::Error>> {
                #output
            }
        }
    };
    TokenStream::from(expanded)
}

fn derive_header_init(name: &Ident, ds: &DataStruct) -> proc_macro2::TokenStream {
    match &ds.fields {
        syn::Fields::Named(fields) => {
            let fs = fields.named.iter().map(|f| {
                let fname = f.ident.as_ref().unwrap();
                let sfname = format!("{fname}");
                let ty = &f.ty;
                quote! {
                    let #fname = header.add_aux_field_t<#ty>(#sfname)?;
                }
            });

            quote! { #(#fs)* }
        }
        syn::Fields::Unnamed(_) | syn::Fields::Unit => {
            abort_call_site!("#[derive(FieldExt)] only available for named fields")
        }
    }
}

fn derive_init_args(name: &Ident, ds: &DataStruct) -> proc_macro2::TokenStream {
    match &ds.fields {
        syn::Fields::Named(fields) => {
            let fs = fields.named.iter().map(|f| {
                let fname = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                quote! {
                    #fname: #ty
                }
            });

            quote! { #(#fs),* }
        }
        syn::Fields::Unnamed(_) | syn::Fields::Unit => {
            abort_call_site!("#[derive(FieldExt)] only available for named fields")
        }
    }
}
struct StructDerived {
    header_init: TokenStream,
    fn_args: TokenStream,
}