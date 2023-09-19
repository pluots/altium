extern crate proc_macro;

use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Literal, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
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
    let struct_ident = parsed.ident;
    let Data::Struct(data) = parsed.data else {
        panic!("only usable on structs");
    };

    // Parse outer attributes
    let mut struct_attr_map = parse_attrs(parsed.attrs).expect("attribute with `id = n` required");
    let TokenTree::Literal(id) = struct_attr_map.remove("id").expect("record ID required") else {
        panic!("record id should be a literal");
    };

    // Handle cases where we want to box the struct
    let use_box = match struct_attr_map.remove("use_box") {
        Some(TokenTree::Ident(val)) if val == "true" => true,
        Some(TokenTree::Ident(val)) if val == "false" => true,
        Some(v) => panic!("Expected ident but got {v:?}"),
        None => false,
    };

    // Handle cases where our struct doesn't have the same name as the enum variant
    let record_variant = match struct_attr_map.remove("record_variant") {
        Some(TokenTree::Ident(val)) => val,
        Some(v) => panic!("Expected ident but got {v:?}"),
        None => struct_ident.clone(),
    };

    error_if_map_not_empty(&struct_attr_map);

    // Collect each match arm and flag initializers that we will concat
    // into our implementation
    let mut match_arms: Vec<TokenStream2> = Vec::new();
    let mut outer_flags: Vec<TokenStream2> = Vec::new();

    // Loop through each field in the struct
    for field in data.fields {
        let Type::Path(path) = field.ty else {
            panic!("invalid type")
        };

        let field_ident = field.ident.unwrap();

        // Parse attributes that exist on the field
        let mut field_attr_map = parse_attrs(field.attrs).unwrap_or_default();

        // Check if we need to parse an array
        if let Some(arr_val) = field_attr_map.remove("array") {
            let arr_val_str = arr_val.to_string();
            if arr_val_str == "true" {
                let count_ident = field_attr_map
                    .remove("count")
                    .expect("missing 'count' attribute");

                let arr_map = field_attr_map
                    .remove("map")
                    .expect("missing 'map' attribute");

                process_array(
                    &struct_ident,
                    &field_ident,
                    count_ident,
                    arr_map,
                    &mut match_arms,
                );
                error_if_map_not_empty(&field_attr_map);
                continue;
            } else if arr_val_str != "false" {
                panic!("array must be `true` or `false` but got {arr_val_str}");
            }
        }

        // We match a single literal, like `OwnerPartId`
        // Perform renaming if attribute requests it
        let match_pat = match field_attr_map.remove("rename") {
            Some(TokenTree::Literal(v)) => v,
            Some(v) => panic!("expected literal, got {v:?}"),
            None => create_key_name(&field_ident),
        };

        // If we haven't consumed all attributes, yell
        error_if_map_not_empty(&field_attr_map);

        let update_stmt = if path.path.segments.first().unwrap().ident == "Option" {
            // Wrap our field is an `Option<T>`
            quote! { ret.#field_ident = Some(parsed); }
        } else {
            quote! { ret.#field_ident = parsed; }
        };

        let path_str = path.to_token_stream().to_string();

        // Types `Location` and `LocationFract` are special cases
        let is_location_fract = path_str.contains("LocationFract");
        if is_location_fract || path_str.contains("Location") {
            process_location(
                &struct_ident,
                &field_ident,
                is_location_fract,
                &mut match_arms,
            );
            continue;
        }

        let Utf8Handler {
            arm: utf8_arm,
            define_flag: utf8_def_flag,
            check_flag: utf8_check_flag,
        } = if path_str.contains("String") || path_str.contains("str") {
            // Altium does this weird thing where it will create a normal key and a key
            // with `%UTF8%` if a value is utf8. We need to discard those redundant values
            make_utf8_handler(&match_pat, &field_ident, &struct_ident, &update_stmt)
        } else {
            Utf8Handler::default()
        };

        let ctx_msg = make_ctx_message(&match_pat, &field_ident, &struct_ident);

        let quoted = quote! {
            #utf8_arm

            #match_pat => {
                #utf8_check_flag

                let parsed = val.parse_as_utf8().context(#ctx_msg)?;
                #update_stmt
            },
        };

        outer_flags.push(utf8_def_flag);
        match_arms.push(quoted);
    }

    let ret_val = if use_box {
        quote! { Ok(SchRecord::#record_variant(Box::new(ret))) }
    } else {
        quote! { Ok(SchRecord::#record_variant(ret)) }
    };

    let ret = quote! {
        impl FromRecord for #struct_ident {
            const RECORD_ID: u32 = #id;

            fn from_record<'a, I: Iterator<Item = (&'a [u8], &'a [u8])>>(
                records: I,
            ) -> Result<SchRecord, crate::Error> {
                let mut ret = Self::default();

                // Boolean flags used to track what we have found throughout the loop
                #(#outer_flags)*

                for (key, val) in records {
                    match key {
                        #(#match_arms)*
                        _ => crate::logging::macro_unsupported_key(stringify!(#struct_ident), key, val)
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
enum AttrParseState {
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

    let mut state = AttrParseState::Key;
    let mut map = BTreeMap::new();

    for token in list.tokens {
        match state {
            AttrParseState::Key => {
                let TokenTree::Ident(idtoken) = token else {
                    panic!("expected an identifier at {token}");
                };
                state = AttrParseState::Eq(idtoken.to_string());
            }
            AttrParseState::Eq(key) => {
                if !matches!(&token, TokenTree::Punct(v) if v.as_char() == '=') {
                    panic!("expected `=` at {token}");
                }
                state = AttrParseState::Val(key);
            }
            AttrParseState::Val(key) => {
                map.insert(key, token);
                state = AttrParseState::Comma;
            }
            AttrParseState::Comma => {
                if !matches!(&token, TokenTree::Punct(v) if v.as_char() == ',') {
                    panic!("expected `,` at {token}");
                }
                state = AttrParseState::Key;
            }
        }
    }

    Some(map)
}

/// Next type of token we are expecting
#[derive(Clone, Debug, PartialEq)]
enum MapParseState {
    Key,
    /// Contains the last key we had
    Dash(Ident),
    Gt(Ident),
    Val(Ident),
    Comma,
}

/// Parse a `(X -> x, Y -> y)` map that tells us how to set members based on
/// found items in an array.
///
/// E.g. with the above, `X1` will set `record[1].x`
fn parse_attr_map(map: TokenTree) -> Vec<(Ident, Ident)> {
    let mut ret = Vec::new();

    let TokenTree::Group(group) = map else {
        panic!("expected group but got {map:?}")
    };

    if group.delimiter() != Delimiter::Parenthesis {
        panic!("expected parenthese but got {:?}", group.delimiter());
    };

    let mut state = MapParseState::Key;

    for token in group.stream() {
        match state {
            MapParseState::Key => {
                let TokenTree::Ident(idtoken) = token else {
                    panic!("expected an identifier at {token}");
                };
                state = MapParseState::Dash(idtoken);
            }
            MapParseState::Dash(key) => {
                if !matches!(&token, TokenTree::Punct(v) if v.as_char() == '-') {
                    panic!("expected `->` at {token}");
                }
                state = MapParseState::Gt(key);
            }
            MapParseState::Gt(key) => {
                if !matches!(&token, TokenTree::Punct(v) if v.as_char() == '>') {
                    panic!("expected `->` at {token}");
                }
                state = MapParseState::Val(key);
            }
            MapParseState::Val(key) => {
                let TokenTree::Ident(ident) = token else {
                    panic!("expcected ident but got {token}");
                };
                ret.push((key, ident));
                state = MapParseState::Comma;
            }
            MapParseState::Comma => {
                if !matches!(&token, TokenTree::Punct(v) if v.as_char() == ',') {
                    panic!("expected `,` at {token}");
                }

                state = MapParseState::Key;
            }
        }
    }

    ret
}

fn error_if_map_not_empty(map: &BTreeMap<String, TokenTree>) {
    assert!(map.is_empty(), "unexpected pairs {map:?}");
}

fn process_location(
    struct_ident: &Ident,
    field_ident: &Ident,
    is_location_fract: bool,
    match_stmts: &mut Vec<TokenStream2>,
) {
    let base_field_str = field_ident.to_string().to_case(Case::Pascal);
    let x_str = format!("{base_field_str}.X");
    let y_str = format!("{base_field_str}.Y");

    let check_patterns = if is_location_fract {
        let x_str_frac = format!("{base_field_str}.X_Frac");
        let y_str_frac = format!("{base_field_str}.Y_Frac");
        vec![
            (x_str, quote!(ret.#field_ident.x)),
            (y_str, quote!(ret.#field_ident.y)),
            (x_str_frac, quote!(ret.#field_ident.x_fract)),
            (y_str_frac, quote!(ret.#field_ident.y_fract)),
        ]
    } else {
        vec![
            (x_str, quote!(ret.#field_ident.x)),
            (y_str, quote!(ret.#field_ident.y)),
        ]
    };

    for (pat_str, assign_field) in check_patterns {
        let match_pat = Literal::byte_string(pat_str.as_bytes());
        let ctx_msg = make_ctx_message(&match_pat, field_ident, struct_ident);

        let match_arm = quote! {
            #match_pat => #assign_field = val.parse_as_utf8().context(#ctx_msg)?,
        };

        match_stmts.push(match_arm);
    }
}

/// Setup handling of `X1 = 1234, Y1 = 909`
fn process_array(
    struct_ident: &Ident,
    field_ident: &Ident,
    count_ident_tt: TokenTree,
    arr_map_tt: TokenTree,
    match_stmts: &mut Vec<TokenStream2>,
) {
    let TokenTree::Literal(match_pat) = count_ident_tt else {
        panic!("expected a literal for `count`");
    };
    let arr_map = parse_attr_map(arr_map_tt);

    let field_name_str = field_ident.to_string();
    let ctx_msg = make_ctx_message(&match_pat, field_ident, struct_ident);

    let count_match = quote! {
        // Set the length of our array once given
        #match_pat => {
            let count = val.parse_as_utf8().context(#ctx_msg)?;

            ret.#field_ident = vec![Default::default(); count].into();
        },
    };

    match_stmts.push(count_match);

    for (match_pfx, assign_value) in arr_map {
        let match_pfx_bstr = Literal::byte_string(match_pfx.to_string().as_bytes());

        let item_match = quote! {
            match_val if crate::common::is_number_pattern(match_val, #match_pfx_bstr) => {
                let idx: usize = match_val.strip_prefix(#match_pfx_bstr).unwrap()
                    .parse_as_utf8()
                    .or_context(|| format!(
                        "while extracting `{}` (`{}`) for `{}` (via proc macro array)",
                        String::from_utf8_lossy(match_val), #field_name_str, stringify!(#struct_ident)
                    ))?;

                let parsed_val = val.parse_as_utf8().or_context(|| format!(
                    "while extracting `{}` (`{}`) for `{}` (via proc macro array)",
                    String::from_utf8_lossy(match_val), #field_name_str, stringify!(#struct_ident)
                ))?;

                ret.#field_ident[idx - 1].#assign_value = parsed_val;
            },
        };
        match_stmts.push(item_match);
    }
}

#[derive(Debug, Default)]
struct Utf8Handler {
    /// The match arm
    arm: TokenStream2,
    /// Definition of a `fieldname_found_utf8_field` flag
    define_flag: TokenStream2,
    /// Checking if a flag is set
    check_flag: TokenStream2,
}

fn make_utf8_handler(
    match_pat: &Literal,
    field_ident: &Ident,
    struct_ident: &Ident,
    update_stmt: &TokenStream2,
) -> Utf8Handler {
    let match_pat = create_key_name_utf8(match_pat);
    let field_name_str = field_ident.to_string();
    let flag_ident = Ident::new(
        &format!("{field_name_str}_found_utf8_field"),
        Span::call_site(),
    );

    let ctx_msg = make_ctx_message(&match_pat, field_ident, struct_ident);
    let arm = quote! {
        #match_pat => {
            let parsed = val.parse_as_utf8()
                // Add context of what we were trying to parse for errors
                .context(#ctx_msg)?;

            #flag_ident = true;
            #update_stmt
        },
    };
    let define_flag = quote! { let mut #flag_ident: bool = false; };
    let check_flag = quote! {
        if #flag_ident {
            ::log::trace!(concat!(
                "skipping ", #field_name_str, " after finding utf8 version"
            ));
            continue;
        }
    };

    Utf8Handler {
        arm,
        define_flag,
        check_flag,
    }
}

/// Make a message with our error context
fn make_ctx_message(pattern: &Literal, field_ident: &Ident, struct_ident: &Ident) -> Literal {
    let s = format!(
        "while matching `{}` (`{}`) for `{}` (via proc macro)",
        pattern, field_ident, struct_ident
    );
    Literal::string(&s)
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

    // Skip replacements if we have a full match
    const REPLACE_SKIP: &[&str] = &["CornerXRadius", "CornerYRadius"];

    // First, convert to Pascal case
    let mut id_str = id.to_string().to_case(Case::Pascal);

    if !REPLACE_SKIP.contains(&id_str.as_str()) {
        for (from, to) in REPLACE {
            id_str = id_str.replace(from, to);
        }
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

/// Convert `name` -> `%UTF8%name` for Altium's weird UTF8 pattern
fn create_key_name_utf8(lit: &Literal) -> Literal {
    let s = lit.to_string();
    let inner = s.strip_prefix("b\"").unwrap().strip_suffix('"').unwrap();
    Literal::byte_string(format!("%UTF8%{inner}").as_bytes())
}
