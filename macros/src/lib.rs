mod ldtk_entity;
mod ldtk_entity_tag;
mod ldtk_enum;
mod tiled_class;
mod tiled_enum;
mod tiled_object;
mod tiled_custom_tile;

#[proc_macro_derive(
    LdtkEntity,
    attributes(ldtk_default, ldtk_name, spawn_sprite, global_entity, callback)
)]
pub fn derive_ldtk_entities(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ldtk_entity::expand_ldtk_entity_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(LdtkEnum, attributes(ldtk_name, wrapper_derive))]
pub fn derive_ldtk_enums(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ldtk_enum::expand_ldtk_enum_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(LdtkEntityTag)]
pub fn derive_ldtk_entity_tags(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ldtk_entity_tag::expand_ldtk_entity_tag_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(
    TiledObject,
    attributes(
        tiled_default,
        instantiate_shape,
        spawn_sprite,
        global_object,
        callback
    )
)]
pub fn derive_tiled_objects(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_object::expand_tiled_objects_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(TiledCustomTile, attributes(tiled_default, callback))]
pub fn derive_tiled_custom_tiles(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_custom_tile::expand_tiled_custom_tiles_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(TiledClass, attributes(tiled_name))]
pub fn derive_tiled_classes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_class::expand_tiled_class_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(TiledEnum, attributes(tiled_name))]
pub fn derive_tiled_enums(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_enum::expand_tiled_enum_derive(syn::parse(input).unwrap())
}