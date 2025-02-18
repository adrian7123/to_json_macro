mod utils;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

use utils::{
    apply_rename_all, field_contains_rename, get_field_name, get_rename_all, has_json_attr,
    optional_sub_ident, vec_sub_ident,
};

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
                  pub   fn to_json_string(&self) -> String {
                        use ::serde_json::json; // Usando o caminho absoluto
                        use ::mongodb::bson::oid::ObjectId; // Usando o caminho absoluto
                        use std::collections::HashMap;
                        use ::serde_json::Value; // Usando o caminho absoluto
                        use ::indexmap::IndexMap; // Usando o caminho absoluto

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
                  pub   fn to_json(&self) -> serde_json::Value {
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
                   pub fn to_json_string(&self) -> String {
                        serde_json::to_string(self).expect("Failed to serialize to JSON")
                    }
                   pub fn to_json(&self) -> ::serde_json::Value {
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
