mod ldtk_entity;
mod ldtk_enum;

#[proc_macro_derive(LdtkEntity, attributes(ldtk_default))]
pub fn derive_ldtk_entitiles(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ldtk_entity::expand_ldtk_entity_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(LdtkEnum, attributes(ldtk_name))]
pub fn derive_ldtk_enums(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ldtk_enum::expand_ldtk_enum_derive(syn::parse(input).unwrap())
}
