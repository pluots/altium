extern crate proc_macro;

use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse2, Attribute, Data, DeriveInput, Meta, Type};

/// Derive `FromRecord` for a type. See that trait for better information.
///
/// Accepts type attributes like:
///
/// ```skip
/// use altium_macros::FromRecord;
///
/// #[derive(FromRecord)]
/// #[from_record(id = 1, use_box = true)]
/// struct Foo {
///     #[from_record(rename = b"FooBar")]
///     foo: String
/// }
/// ```
///
/// This is a super crude implementation, but it's for our internal use so
/// panics are alright
#[proc_macro_derive(FromRecord, attributes(from_record))]
pub fn derive_fromrecord(tokens: TokenStream) -> TokenStream {
    inner(tokens.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn inner(tokens: TokenStream2) -> syn::Result<TokenStream2> {
    let parsed: DeriveInput = parse2(tokens)?;
    let name = parsed.ident;
    let Data::Struct(data) = parsed.data else {
        panic!("only usable on structs");
    };

    let mut attr_map = parse_attrs(parsed.attrs).expect("attribute with `id = n` required");
    let TokenTree::Literal(id) = attr_map.remove("id").expect("record ID required") else {
        panic!("record id should be a literal");
    };

    let use_box = match attr_map.remove("use_box") {
        Some(TokenTree::Ident(val)) if val == "true" => true,
        Some(TokenTree::Ident(val)) if val == "false" => true,
        Some(v) => panic!("Expected ident but got {v:?}"),
        None => false,
    };

    error_if_map_not_empty(&attr_map);

    let mut match_stmts = Vec::new();
    for field in data.fields {
        // Convert to pascal case
        let Type::Path(path) = field.ty else {
            panic!("invalid type")
        };

        let field_name = field.ident.unwrap();
        let mut field_map = parse_attrs(field.attrs).unwrap_or_default();

        // We match a single literal, like `OwnerPartId`
        let match_pat = match field_map.remove("rename") {
            Some(TokenTree::Literal(v)) => v,
            Some(v) => panic!("expected literal, got {v:?}"),
            None => create_key_name(&field_name),
        };
        error_if_map_not_empty(&field_map);

        let match_lit = match_pat.to_string();
        let ret_stmt = if path.path.segments.first().unwrap().ident == "Option" {
            // Optional return
            quote! { ret.#field_name = Some(parsed); }
        } else {
            quote! { ret.#field_name = parsed; }
        };

        let quoted = quote! {
            #match_pat => {
                let parsed = val.parse_as_utf8()
                    // Add context of what we were trying to parse
                    .context(concat!(
                        "while matching `",
                        #match_lit,
                        "` for `",
                        stringify!(#name),
                        "` (via proc macro)"
                    ))?;
                #ret_stmt
            },
        };

        match_stmts.push(quoted);
    }

    let ret_val = if use_box {
        quote! { Ok(SchRecord::#name(Box::new(ret))) }
    } else {
        quote! { Ok(SchRecord::#name(ret)) }
    };

    let ret = quote! {
        impl FromRecord for #name {
            const RECORD_ID: u32 = #id;

            fn from_record<'a, I: Iterator<Item = (&'a [u8], &'a [u8])>>(
                records: I,
            ) -> Result<SchRecord, crate::Error> {
                let mut ret = Self::default();
                for (key, val) in records {
                    match key {
                        #(#match_stmts)*
                        _ => crate::__private::macro_unsupported_key(stringify!(#name), key, val)
                    }
                }

                #ret_val
            }
        }
    };

    Ok(ret)
}

/// Next type of token we are expecting
#[derive(Clone, Debug, PartialEq)]
enum AttrState {
    Key,
    /// Contains the last key we had
    Eq(String),
    Val(String),
    Comma,
}

/// Parse an attribute to a hashmap if it exists
///
/// Only takes attributes with the name `from_record`
fn parse_attrs(attrs: Vec<Attribute>) -> Option<BTreeMap<String, TokenTree>> {
    let attr = attrs
        .into_iter()
        .find(|attr| attr.path().is_ident("from_record"))?;

    let Meta::List(list) = attr.meta else {
        panic!("invalid usage; use `#[from_record(...=..., ...)]`");
    };

    let mut state = AttrState::Key;
    let mut map = BTreeMap::new();

    for token in list.tokens {
        match state {
            AttrState::Key => {
                let TokenTree::Ident(idtoken) = token else {
                    panic!("expected an identifier at {token}");
                };
                state = AttrState::Eq(idtoken.to_string());
            }
            AttrState::Eq(key) => {
                match token {
                    TokenTree::Punct(v) if v.as_char() == '=' => (),
                    _ => panic!("expected `=` at {token}"),
                }

                state = AttrState::Val(key);
            }
            AttrState::Val(key) => {
                map.insert(key, token);
                state = AttrState::Comma;
            }
            AttrState::Comma => {
                match token {
                    TokenTree::Punct(v) if v.as_char() == ',' => (),
                    _ => panic!("expected `,` at {token}"),
                };
                state = AttrState::Key;
            }
        }
    }

    Some(map)
}

fn error_if_map_not_empty(map: &BTreeMap<String, TokenTree>) {
    assert!(map.is_empty(), "unexpected pairs {map:?}");
}

/// From a field in our struct, create the name we should match by
fn create_key_name(id: &Ident) -> Literal {
    // Replace these strings, just inconsistencies
    const REPLACE: &[(&str, &str)] = &[
        ("LocationX", "Location.X"),
        ("LocationY", "Location.Y"),
        ("CornerX", "Corner.X"),
        ("CornerY", "Corner.Y"),
        ("UniqueId", "UniqueID"),
        ("FontId", "FontID"),
        ("PartIdLocked", "PartIDLocked"),
        ("Accessible", "Accesible"),
        ("Frac", "_Frac"),
    ];

    // First, convert to Pascal case
    let mut id_str = id.to_string().to_case(Case::Pascal);

    for (from, to) in REPLACE {
        id_str = id_str.replace(from, to);
    }

    Literal::byte_string(id_str.as_bytes())

    // ID_RE.get_or_init(|| Regex::new(ID_RXP)).replace(&id_str, ID_REPL);
    // None if field_name == "unique_id" => Literal::byte_string(b"UniqueID"),
    // None if field_name == "font_id" => Literal::byte_string(b"FontID"),
    // None if field_name == "location_x" => Literal::byte_string(b"Location.X"),
    // None if field_name == "location_y" => Literal::byte_string(b"Location.Y"),
    // None if field_name == "corner_x" => Literal::byte_string(b"Corner.X"),
    // None if field_name == "corner_y" => Literal::byte_string(b"Corner.Y"),
    // // literal misspelling...
    // None if field_name == "is_not_accessible" => Literal::byte_string(b"IsNotAccesible"),
    // None => Literal::byte_string(field_name.to_string().to_case(Case::Pascal).as_bytes()),
}
