extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Field, GenericArgument, Ident, PathArguments, Type,
};

fn get_field_name(field: &Field) -> String {
    for attr in &field.attrs {
        if attr.path.is_ident("serde") {
            if let Ok(meta) = attr.parse_meta() {
                if let syn::Meta::List(meta_list) = meta {
                    for nested_meta in meta_list.nested {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(m)) = nested_meta {
                            if m.path.is_ident("rename") {
                                if let syn::Lit::Str(lit_str) = &m.lit {
                                    return lit_str.value();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    field.ident.as_ref().unwrap().to_string()
}

fn field_contains_rename(field: &Field) -> bool {
    for attr in &field.attrs {
        if attr.path.is_ident("serde") {
            if let Ok(meta) = attr.parse_meta() {
                if let syn::Meta::List(meta_list) = meta {
                    for nested_meta in meta_list.nested {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(m)) = nested_meta {
                            if m.path.is_ident("rename") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn optional_sub_ident(ty: &Type) -> Option<Ident> {
    if let Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &type_path.path.segments[0].arguments {
                if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                    if let Type::Path(inner_type_path) = inner_ty {
                        if inner_type_path.path.segments.len() == 1 {
                            return Some(inner_type_path.path.segments[0].clone().ident);
                        }
                    }
                }
            }
        }
    }

    None
}

fn vec_sub_ident(ty: &Type) -> Option<Ident> {
    if let Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Vec" {
            if let PathArguments::AngleBracketed(args) = &type_path.path.segments[0].arguments {
                if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                    if let Type::Path(inner_type_path) = inner_ty {
                        if inner_type_path.path.segments.len() == 1 {
                            return Some(inner_type_path.path.segments[0].clone().ident);
                        }
                    }
                }
            }
        }
        if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &type_path.path.segments[0].arguments {
                if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                    if let Type::Path(inner_type_path) = inner_ty {
                        if inner_type_path.path.segments.len() == 1
                            && inner_type_path.path.segments[0].ident == "Vec"
                        {
                            if let PathArguments::AngleBracketed(args) =
                                &inner_type_path.path.segments[0].arguments
                            {
                                if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                                    if let Type::Path(inner_inner_type_path) = inner_ty {
                                        if inner_inner_type_path.path.segments.len() == 1 {
                                            return Some(
                                                inner_inner_type_path.path.segments[0]
                                                    .clone()
                                                    .ident,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

// Check if a field has the #[json] attribute.
fn has_json_attr(field: &Field) -> bool {
    for attr in &field.attrs {
        if attr.path.is_ident("json") {
            return true;
        }
    }
    false
}

fn get_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident("serde") {
            if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
                for nested_meta in meta_list.nested {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(m)) = nested_meta {
                        if m.path.is_ident("rename_all") {
                            if let syn::Lit::Str(lit_str) = &m.lit {
                                return Some(lit_str.value());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn apply_rename_all(convention: &str, field_name: &str) -> String {
    match convention {
        "snake_case" => convert_to_snake_case(field_name),
        "camelCase" => convert_to_camel_case(field_name),
        "PascalCase" => convert_to_pascal_case(field_name),
        "SCREAMING_SNAKE_CASE" => convert_to_screaming_snake_case(field_name),
        _ => field_name.to_string(),
    }
}

fn convert_to_snake_case(name: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            snake_case.push('_');
        }
        snake_case.push(c.to_ascii_lowercase());
    }
    snake_case
}

fn convert_to_camel_case(name: &str) -> String {
    let mut camel_case = String::new();
    let mut uppercase_next = false;
    for c in name.chars() {
        if c == '_' {
            uppercase_next = true;
        } else if uppercase_next {
            camel_case.push(c.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            camel_case.push(c);
        }
    }
    println!("{} {}", name, camel_case);
    camel_case
}

fn convert_to_pascal_case(name: &str) -> String {
    let camel_case = convert_to_camel_case(name);
    camel_case
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}

fn convert_to_screaming_snake_case(name: &str) -> String {
    convert_to_snake_case(name).to_ascii_uppercase()
}

#[proc_macro_derive(ToJson, attributes(json))]
pub fn to_json_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let rename_all_convention = get_rename_all(&input.attrs);

    let expanded = match input.data {
        Data::Struct(data) => {
            let field_checks_ordered = data.fields.iter().map(|field| {
                let field_name_str = get_field_name(field);

                let final_name = if field_contains_rename(field) {
                    field_name_str
                } else {
                    if let Some(ref convention) = rename_all_convention {
                        apply_rename_all(convention, &field_name_str)
                    } else {
                        field_name_str
                    }
                };

                return quote! {
                    ordered_keys.push(#final_name.to_string());
                };
            });

            let field_checks = data.fields.iter().map(|field| {
                let field_name = &field.ident;
                let field_name_str = get_field_name(field);
                let field_type = &field.ty;

                let final_name = if field_contains_rename(field) {
                    field_name_str
                } else {
                    if let Some(ref convention) = rename_all_convention {
                        apply_rename_all(convention, &field_name_str)
                    }
                    else {
                        field_name_str
                    }

                };

                if let syn::Type::Path(type_path) = field_type {
                    if type_path.path.segments.len() == 1 {
                        let optional_ident = optional_sub_ident(field_type);

                        let mut is_optional= false;

                        let has_json = has_json_attr(field);

                        let ident = if optional_ident.is_some()
                        {
                            is_optional = true;
                            optional_ident.unwrap()
                        } else {
                            type_path.path.segments[0].ident.clone()
                        };


                        if ident == "ObjectId"
                        {
                            if is_optional {
                                return quote! {
                                    if let Some(ref value) = self.#field_name {
                                        map.insert(#final_name.to_string(), serde_json::json!(value.to_string()));
                                    } else {
                                        map.insert(#final_name.to_string(), serde_json::Value::Null);
                                    }
                                }
                            }
                            return quote! {
                                map.insert(#final_name.to_string(), serde_json::json!(self.#field_name.to_string()));
                            };
                        }

                        if  ident == "DateTime"
                        {
                            if is_optional {
                                return quote! {
                                    if let Some(ref value) = self.#field_name {
                                        map.insert(#final_name.to_string(), serde_json::json!(value.try_to_rfc3339_string().expect("try_to_rfc3339_string err")));
                                    } else {
                                        map.insert(#final_name.to_string(), serde_json::Value::Null);
                                    }
                                }
                            }
                            return quote! {
                                map.insert(#final_name.to_string(), serde_json::json!(self.#field_name.try_to_rfc3339_string().expect("try_to_rfc3339_string err")));
                            };
                        }

                        if  ident == "Vec"
                        {
                            if let Some(ident) = vec_sub_ident(field_type) {
                                if ident == "ObjectId"
                                {
                                    if is_optional {
                                        return quote! {
                                            if let Some(ref value) = self.#field_name {
                                                map.insert(#final_name.to_string(), value.unwrap().iter().map(|i|i.to_string()).collect::<serde_json::Value>());
                                            } else {
                                                map.insert(#final_name.to_string(), serde_json::Value::Null);
                                            }
                                        }
                                    }
                                    return quote! {
                                        map.insert(#final_name.to_string(), self.#field_name.iter().map(|i|i.to_string()).collect::<serde_json::Value>());
                                    };
                                }

                                if  ident == "DateTime"
                                {
                                    if is_optional {
                                        return quote! {
                                            if let Some(ref value) = self.#field_name {
                                                map.insert(#final_name.to_string(), self.#field_name.iter().map(|i|i.try_to_rfc3339_string().expect("try_to_rfc3339_string err")).collect::<serde_json::Value>());
                                            } else {
                                                map.insert(#final_name.to_string(), serde_json::Value::Null);
                                            }
                                        }
                                    }
                                    return quote! {
                                        map.insert(#final_name.to_string(), self.#field_name.iter().map(|i|i.try_to_rfc3339_string().expect("try_to_rfc3339_string err")).collect::<serde_json::Value>());
                                    };
                                }
                            }

                            if has_json == false {
                                return quote! {
                                    map.insert(#final_name.to_string(), serde_json::json!(self.#field_name));
                                };
                            }

                            if is_optional {
                                return quote! {
                                    if let Some(ref value) = self.#field_name {
                                        map.insert(#final_name.to_string(), value.unwrap().iter().map(|i|i.to_json()).collect::<serde_json::Value>());
                                    } else {
                                        map.insert(#final_name.to_string(), serde_json::Value::Null);
                                    }
                                }
                            }

                            return quote! {
                                map.insert(#final_name.to_string(), self.#field_name.iter().map(|i|i.to_json()).collect::<serde_json::Value>());
                            };
                        }

                        if has_json {
                            if is_optional {
                                return quote! {
                                    if let Some(ref value) = self.#field_name {
                                        map.insert(#final_name.to_string(), value.to_json());
                                    } else {
                                        map.insert(#final_name.to_string(), serde_json::Value::Null);
                                    }
                                };
                            }

                            return quote! {
                                map.insert(#final_name.to_string(), self.#field_name.to_json());
                            };
                        }
                    }
                }

                quote! {
                    map.insert(#final_name.to_string(), serde_json::json!(self.#field_name));
                }
            });

            quote! {
                impl #name {
                     fn to_json_string(&self) -> String {
                        use serde_json::json;
                        use mongodb::bson::oid::ObjectId;
                        use std::collections::HashMap;
                        use serde_json::Value;
                        use indexmap::IndexMap;

                        let mut map: HashMap<String, Value> = HashMap::new();
                        let mut ordered_keys: Vec<String> = vec![];

                        #( #field_checks_ordered )*
                        #( #field_checks )*

                        let mut ordered_map: IndexMap<String, Value> = IndexMap::new();

                        for key in ordered_keys {
                            if let Some(value) = map.remove(&key) {
                                ordered_map.insert(key.clone(), value);
                            }
                        }

                        serde_json::to_string(&ordered_map).expect("Failed to serialize to JSON")
                    }
                     fn to_json(&self) -> serde_json::Value {
                        serde_json::from_str(&self.to_json_string()).expect("Failed to deserialize from JSON")
                    }
                }

                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", self.to_string())
                    }
                }
            }
        }
        Data::Enum(_) => {
            quote! {
                impl #name {
                    pub fn get_string(&self) -> String {
                        self.to_json_string().replace("\"", "")
                    }
                    fn to_json_string(&self) -> String {
                        serde_json::to_string(self).expect("Failed to serialize to JSON")
                    }
                    fn to_json(&self) -> serde_json::Value {
                        serde_json::from_str(&self.to_json_string()).expect("Failed to deserialize from JSON")
                    }
                }

                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", self.get_string())
                    }
                }
            }
        }
        _ => quote! {
            compile_error!("ToJson macro can only be used with structs and enums");
        },
    };

    TokenStream::from(expanded)
}
