use syn::{Field, GenericArgument, Ident, PathArguments, Type};

pub fn get_field_name(field: &Field) -> String {
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

pub fn field_contains_rename(field: &Field) -> bool {
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

pub fn optional_sub_ident(ty: &Type) -> Option<Ident> {
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

pub fn vec_sub_ident(ty: &Type) -> Option<Ident> {
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
pub fn has_json_attr(field: &Field) -> bool {
    for attr in &field.attrs {
        if attr.path.is_ident("json") {
            return true;
        }
    }
    false
}

pub fn get_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
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

pub fn apply_rename_all(convention: &str, field_name: &str) -> String {
    match convention {
        "snake_case" => convert_to_snake_case(field_name),
        "camelCase" => convert_to_camel_case(field_name),
        "PascalCase" => convert_to_pascal_case(field_name),
        "SCREAMING_SNAKE_CASE" => convert_to_screaming_snake_case(field_name),
        _ => field_name.to_string(),
    }
}

pub fn convert_to_snake_case(name: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            snake_case.push('_');
        }
        snake_case.push(c.to_ascii_lowercase());
    }
    snake_case
}

pub fn convert_to_camel_case(name: &str) -> String {
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
    camel_case
}

pub fn convert_to_pascal_case(name: &str) -> String {
    let camel_case = convert_to_camel_case(name);
    camel_case
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}

pub fn convert_to_screaming_snake_case(name: &str) -> String {
    convert_to_snake_case(name).to_ascii_uppercase()
}
