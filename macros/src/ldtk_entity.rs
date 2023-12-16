pub fn impl_ldtk_entity(input: syn::DeriveInput) -> proc_macro::TokenStream {
    quote::quote! {
        impl LdtkEntity for s {
            fn initialize(
                commands: &mut EntityCommands,
                entity_instance: &EntityInstance,
                asset_server: &AssetServer,
            ) -> Self {
                let mut entity = Self {
                    iid: entity_instance.identifier.clone(),
                    local_pos: entity_instance.px,
                    width: entity_instance.width,
                    height: entity_instance.height,
                };
                entity.initialize(commands, entity_instance, asset_server);
                entity
            }
        }
    }.into()
}
