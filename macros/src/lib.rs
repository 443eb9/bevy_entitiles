mod ldtk_entity;

#[proc_macro_derive(LdtkEntity)]
pub fn derive_ldtk_entitiles(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ldtk_entity::impl_ldtk_entity(syn::parse(input).unwrap())
}
