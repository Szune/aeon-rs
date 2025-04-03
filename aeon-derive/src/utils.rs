use aeon_derive_utils::Type;

fn is_type_builtin(typ: &Type) -> bool {
    matches!(
        typ.ident.to_string().as_str(),
        "bool"
            | "String"
            | "i64"
            | "i32"
            | "i16"
            | "i8"
            | "u64"
            | "u32"
            | "u16"
            | "u8"
            | "f64"
            | "f32"
    )
}
pub(crate) fn get_macro_types_from_type(typ: &Type) -> Vec<String> {
    if typ.generics.is_empty() {
        let name = typ.to_full_path();
        if !is_type_builtin(typ) {
            return vec![name];
        }
        return Vec::new();
    }

    let mut types = Vec::new();
    for g in &typ.generics {
        if g.generics.is_empty() {
            if !is_type_builtin(g) {
                types.push(g.to_full_path());
            }
        } else {
            let additional_types = get_macro_types_from_type(g);
            types.extend(additional_types);
        }
    }

    types
}
