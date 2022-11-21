use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident};

#[proc_macro_derive(FieldExt)]
#[proc_macro_error]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let Data::Struct(ds) = input.data else { abort_call_site!("#[derive(FieldExt)] not allowed for enums or DataStructs")};

    let rec_aux = derive_record_auxiliary(&name, &ds);
    let hdr_init = derive_header_init(&name, &ds);

    let expanded = quote! {
        #rec_aux

        #hdr_init
    };
    TokenStream::from(expanded)
}

fn derive_record_auxiliary(name: &Ident, ds: &DataStruct) -> proc_macro2::TokenStream {
    let Fields::Named(ref fields) = ds.fields else { abort_call_site!("#[derive(FieldExt)] only for named fields") };
    let fs = fields.named.iter().map(|f| {
        let fname = f.ident.as_ref().unwrap();
        let sfname = format!("{fname}");
        let ty = &f.ty;
        quote! {
            fn #fname(rec: &slow5::RecordT<#name>) -> Result<#ty, slow5::Slow5Error> {
                rec.get_aux_field(#sfname)
            }
        }
    });
    let impl_record_aux = quote! {
        impl #name {
            #(#fs)*
        }
    };
    impl_record_aux
}

fn derive_header_init(name: &Ident, ds: &DataStruct) -> proc_macro2::TokenStream {
    let Fields::Named(ref fields) = ds.fields else { abort_call_site!("#[derive(FieldExt)] only for named fields") };
    let fs = fields.named.iter().map(|f| {
        let fname = f.ident.as_ref().unwrap();
        let sfname = fname.to_string();
        let ty = &f.ty;
        quote! {
            header.add_aux_field_t::<&'static str, #ty>(#sfname).unwrap();
        }
    });

    quote! {
        impl slow5::FieldExt for #name {
            fn set_header_aux_fields(header: &slow5::Header) {
                #(#fs)*
            }
        }
    }
}
