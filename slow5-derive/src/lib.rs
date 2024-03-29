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
        // let set_fname = format_ident!("set_{fname}");
        let ty = &f.ty;
        quote! {
            fn #fname(rec: &slow5::typed::record::RecordT<#name>) -> Result<#ty, slow5::Slow5Error> {
                rec.get_aux_field(#sfname)
            }

            // fn #set_fname(rb: &mut slow5::RecordT<#name>, val: #ty) -> Result<(), slow5::Slow5Error> {
            //     todo!()
            // }
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
        impl slow5::typed::FieldExt for #name {
            fn set_header_aux_fields(header: &slow5::typed::Header<Self>) {
                #(#fs)*
            }
        }
    }
}

#[proc_macro_derive(AuxEnumExt)]
#[proc_macro_error]
pub fn derive_enums(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let Data::Enum(ds) = input.data else { abort_call_site!("#[derive(AuxEnumExt)] only allowed for enums")};
    for variant in ds.variants.iter() {
        let Fields::Unit = variant.fields else { abort_call_site!("Only unit variants allowed, fields cannot contain data")};
        if variant.discriminant.is_some() {
            abort_call_site!("Variants not allowed to have discriminants");
        }
        let snake_ident = casey::snake!(&variant.ident);
    }
    todo!()
}
