pub fn expand_ldtk_entity_tag_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    match input.data {
        syn::Data::Struct(data) => {
            assert!(
                data.fields.is_empty(),
                "LdtkEntityTag can only be derived for zero sized structs"
            );
        }
        _ => panic!("LdtkEntityTag can only be derived for zero sized structs"),
    }

    let ident = &input.ident;

    quote::quote!(
        impl bevy_entitiles::ldtk::traits::LdtkEntityTag for #ident {
            fn add_tag(commands: &mut bevy::ecs::system::EntityCommands) {
                commands.insert(Self);
            }
        }
    ).into()
}
