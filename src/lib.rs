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

#[proc_macro_derive(ToJson, attributes(json))]
pub fn to_json_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match input.data {
        Data::Struct(data) => {
            let field_checks_ordered = data.fields.iter().map(|field| {
                let field_name_str = get_field_name(field);
                return quote! {
                    ordered_keys.push(#field_name_str.to_string());
                };
            });

            let field_checks = data.fields.iter().map(|field| {
                let field_name = &field.ident;
                let field_name_str = get_field_name(field);
                let field_type = &field.ty;

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
                                        map.insert(#field_name_str.to_string(), serde_json::json!(value.to_string()));
                                    } else {
                                        map.insert(#field_name_str.to_string(), serde_json::Value::Null);
                                    }
                                }
                            }
                            return quote! {
                                map.insert(#field_name_str.to_string(), serde_json::json!(self.#field_name.to_string()));
                            };
                        }

                        if  ident == "DateTime"
                        {
                            if is_optional {
                                return quote! {
                                    if let Some(ref value) = self.#field_name {
                                        map.insert(#field_name_str.to_string(), serde_json::json!(value.try_to_rfc3339_string().expect("try_to_rfc3339_string err")));
                                    } else {
                                        map.insert(#field_name_str.to_string(), serde_json::Value::Null);
                                    }
                                }
                            }
                            return quote! {
                                map.insert(#field_name_str.to_string(), serde_json::json!(self.#field_name.try_to_rfc3339_string().expect("try_to_rfc3339_string err")));
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
                                                map.insert(#field_name_str.to_string(), value.unwrap().iter().map(|i|i.to_string()).collect::<serde_json::Value>());
                                            } else {
                                                map.insert(#field_name_str.to_string(), serde_json::Value::Null);
                                            }
                                        }
                                    }
                                    return quote! {
                                        map.insert(#field_name_str.to_string(), self.#field_name.iter().map(|i|i.to_string()).collect::<serde_json::Value>());
                                    };
                                }

                                if  ident == "DateTime"
                                {
                                    if is_optional {
                                        return quote! {
                                            if let Some(ref value) = self.#field_name {
                                                map.insert(#field_name_str.to_string(), self.#field_name.iter().map(|i|i.try_to_rfc3339_string().expect("try_to_rfc3339_string err")).collect::<serde_json::Value>());
                                            } else {
                                                map.insert(#field_name_str.to_string(), serde_json::Value::Null);
                                            }
                                        }
                                    }
                                    return quote! {
                                        map.insert(#field_name_str.to_string(), self.#field_name.iter().map(|i|i.try_to_rfc3339_string().expect("try_to_rfc3339_string err")).collect::<serde_json::Value>());
                                    };
                                }
                            }

                            if has_json == false {
                                return quote! {
                                    map.insert(#field_name_str.to_string(), serde_json::json!(self.#field_name));
                                };
                            }

                            if is_optional {
                                return quote! {
                                    if let Some(ref value) = self.#field_name {
                                        map.insert(#field_name_str.to_string(), value.unwrap().iter().map(|i|i.to_json()).collect::<serde_json::Value>());
                                    } else {
                                        map.insert(#field_name_str.to_string(), serde_json::Value::Null);
                                    }
                                }
                            }

                            return quote! {
                                map.insert(#field_name_str.to_string(), self.#field_name.iter().map(|i|i.to_json()).collect::<serde_json::Value>());
                            };
                        }

                        if has_json {
                            if is_optional {
                                return quote! {
                                    if let Some(ref value) = self.#field_name {
                                        map.insert(#field_name_str.to_string(), value.to_json());
                                    } else {
                                        map.insert(#field_name_str.to_string(), serde_json::Value::Null);
                                    }
                                };
                            }

                            return quote! {
                                map.insert(#field_name_str.to_string(), self.#field_name.to_json());
                            };
                        }
                    }
                }

                quote! {
                    map.insert(#field_name_str.to_string(), serde_json::json!(self.#field_name));
                }
            });

            quote! {
                impl  #name {
                    pub fn to_json_string(&self) -> String {
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
                   pub  fn to_json(&self) -> serde_json::Value {
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
                impl  #name {
                  pub  fn get_string(&self) -> String {
                        self.to_json_string().replace("\"", "")
                    }
                  pub  fn to_json_string(&self) -> String {
                        serde_json::to_string(self).expect("Failed to serialize to JSON")
                    }
                  pub  fn to_json(&self) -> serde_json::Value {
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
