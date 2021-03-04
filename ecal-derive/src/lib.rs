extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Lit, LitStr, Meta};

#[proc_macro_derive(Message, attributes(type_name, type_prefix))]
pub fn ecal_message_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let implementor = &input.ident;

    let type_name = find_type_name(&input.attrs).unwrap_or_else(|| implementor.to_string());
    let prefix = find_prefix(&input.attrs).unwrap_or_default();

    let full_type_name = format!("{}{}", prefix, type_name);

    let expanded = quote! {
        impl ecal::Message for #implementor {
            fn type_name() -> &'static str {
                #full_type_name
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn find_type_name(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs.iter().filter(|attr| attr.path.is_ident("type_name")) {
        if let Some(inner) = extract_str_lit(attr) {
            return Some(inner.value());
        } else {
            panic!("Please use #[type_name = \"...\"] attribute to specify a type name");
        }
    }

    None
}

fn find_prefix(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs
        .iter()
        .filter(|attr| attr.path.is_ident("type_prefix"))
    {
        if let Some(inner) = extract_str_lit(attr) {
            return Some(inner.value());
        } else {
            panic!("Please use #[type_prefix = \"...\"] attribute to specify a type prefix");
        }
    }

    None
}

fn extract_str_lit(attr: &Attribute) -> Option<LitStr> {
    if let Meta::NameValue(meta) = attr.parse_meta().ok()? {
        if let Lit::Str(inner) = meta.lit {
            return Some(inner);
        }
    }

    None
}
