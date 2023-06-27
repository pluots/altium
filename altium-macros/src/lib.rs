extern crate proc_macro;

use convert_case::{Case, Casing};

use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse2, Attribute, Data, DeriveInput, Meta, Type};

/// Super crude implementation, but it's for our internal use so panics are
/// alright
#[proc_macro_derive(FromRecord, attributes(from_record))]
pub fn derive_fromrecord(tokens: TokenStream) -> TokenStream {
    inner(tokens.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn inner(tokens: TokenStream2) -> syn::Result<TokenStream2> {
    let parsed: DeriveInput = parse2(tokens)?;
    let name = parsed.ident;
    let Data::Struct(data) = parsed.data else {
        panic!("only usable on structs");
    };

    let id = parse_attr(parsed.attrs, "id").expect("record ID required");

    let mut match_stmts = Vec::new();
    for field in data.fields {
        // Convert to pascal case
        let Type::Path(path) = field.ty else {
            panic!("invalid type")
        };

        let fname = field.ident.unwrap();
        let match_str = match parse_attr(field.attrs, "rename") {
            Some(v) => v,
            None => Literal::byte_string(fname.to_string().to_case(Case::Pascal).as_bytes()),
        };

        let optional = path.path.segments.first().unwrap().ident == "Option";
        let quoted = if optional {
            quote! { #match_str => ret.#fname = Some(val.parse_utf8()?), }
        } else {
            quote! { #match_str => ret.#fname = val.parse_utf8()?, }
        };

        match_stmts.push(quoted);
    }

    let ret = quote! {
        impl FromRecord for #name {
            const RECORD_ID: u32 = #id;

            fn from_record<'a, I: Iterator<Item = (&'a [u8], &'a [u8])>>(
                records: I,
            ) -> Result<SchRecord, Error> {
                let mut ret = Self::default();
                for (key, val) in records {
                    match key {
                        #(#match_stmts)*
                        _ => eprintln!(
                                "unsupported key for `{}`: `{}={}`",
                                stringify!(#name),
                                buf2lstring(key), buf2lstring(val)
                            ),
                    }
                }

                Ok(SchRecord::#name(ret))
            }
        }
    };

    Ok(ret)
}

fn parse_attr(attrs: Vec<Attribute>, key: &str) -> Option<Literal> {
    let attr = attrs
        .into_iter()
        .find(|a| a.path().is_ident("from_record"))?;
    let Meta::List(list) = attr.meta else {
        panic!("invalid usage; use `#[from_record(...=...)]`");
    };
    let mut tsiter = list.tokens.into_iter();
    let TokenTree::Ident(idtoken) = tsiter.next().unwrap() else {
        panic!("first field must be `id`");
    };
    assert_eq!(idtoken, key);
    let TokenTree::Punct(eqpunct) = tsiter.next().unwrap() else {
        panic!("second field must be `=`");
    };
    assert_eq!(eqpunct.as_char(), '=');
    let TokenTree::Literal(val) = tsiter.next().unwrap() else {
        panic!("last field must be a literal");
    };

    Some(val)
}
