use proc_macro::{self, TokenStream};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields};
use convert_case::{Case, Casing};

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

#[proc_macro_derive(Command, attributes(state))]
pub fn derive_command(input: TokenStream) -> TokenStream {

    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let attrs = &input.attrs;

    let state = attrs.iter().find(|attr|
        attr.path().is_ident("state")
    );

    let state = match state {
        Some(s) => s,
        None => {return derive_error!("attribute 'state' is mandatory");}
    };

    let state_name: syn::Ident = state.parse_args().expect("cannot parse attribute state");

    // get enum name
    let name = &input.ident;
    let data = &input.data;
    let mut fn_core;

    match data {
        Data::Enum(data_enum) => {

            fn_core = TokenStream2::new();

            // Iterate over enum variants
            // `variants` if of type `Punctuated` which implements IntoIterator
            //
            // https://doc.servo.org/syn/punctuated/struct.Punctuated.html
            // https://doc.servo.org/syn/struct.Variant.html
            for variant in &data_enum.variants {

                // Variant's name
                let variant_name = &variant.ident;

                // Variant can have unnamed fields like `Variant(i32, i64)`
                // Variant can have named fields like `Variant {x: i32, y: i32}`
                // Variant can be named Unit like `Variant`
                let fields_in_variant = match &variant.fields {
                    Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
                    Fields::Unit => quote_spanned! { variant.span()=> },
                    Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
                };


                // Here we construct the function for the current variant
                let result = format!("CMD.{}", variant_name.to_string());
                fn_core.extend(quote! {
                    #name::#variant_name #fields_in_variant => format!( "{}.{}",state_name,#result),
                });
            }
        }
        _ => return derive_error!("Command is only implemented for enums"),
    }

    let output = quote! {
        impl Command for #name {
                fn command_name(&self) -> CommandName {
                    let state_name = #state_name::state_name();
                    match self {
                        #fn_core
                    }
                }
        }
    };
    output.into()
}

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {

    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    // get enum name
    let name = &input.ident;
    let data = &input.data;
    let mut fn_core;

    match data {
        Data::Enum(data_enum) => {

            fn_core = TokenStream2::new();

            // Iterate over enum variants
            // `variants` if of type `Punctuated` which implements IntoIterator
            //
            // https://doc.servo.org/syn/punctuated/struct.Punctuated.html
            // https://doc.servo.org/syn/struct.Variant.html
            for variant in &data_enum.variants {

                // Variant's name
                let variant_name = &variant.ident;

                // Variant can have unnamed fields like `Variant(i32, i64)`
                // Variant can have named fields like `Variant {x: i32, y: i32}`
                // Variant can be named Unit like `Variant`
                let fields_in_variant = match &variant.fields {
                    Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
                    Fields::Unit => quote_spanned! { variant.span()=> },
                    Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
                };


                // Here we construct the function for the current variant
                let result = format!("evt.{}",variant_name.to_string().to_case(Case::Snake));
                fn_core.extend(quote! {
                    #name::#variant_name #fields_in_variant => #result,
                });
            }
        }
        _ => return derive_error!("Event is only implemented for enums"),
    }

    let output = quote! {
        impl Event for #name {
                fn event_name(&self) -> EventName {
                    match self {
                        #fn_core
                    }
                }
        }
    };
    output.into()
}