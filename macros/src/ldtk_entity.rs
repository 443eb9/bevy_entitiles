static LDTK_DEFAULT_ATTR: &str = "ldtk_default";
static LDTK_NAME_ATTR: &str = "ldtk_name";
static SPAWN_SPRITE_ATTR: &str = "spawn_sprite";
static GLOBAL_ENTITY_ATTR: &str = "global_entity";
static CALLBACK_ATTR: &str = "callback";
static LDTK_TAG_ATTR: &str = "ldtk_tag";

pub fn expand_ldtk_entity_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = input.ident;
    let attrs = &input.attrs;

    let spawn_sprite_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == SPAWN_SPRITE_ATTR);
    let spawn_sprite = {
        if spawn_sprite_attr.is_some() {
            quote::quote!(
                entity_instance.generate_sprite(commands, ldtk_manager);
            )
        } else {
            quote::quote!()
        }
    };

    let global_entity_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == GLOBAL_ENTITY_ATTR);
    let global_entity = {
        if global_entity_attr.is_some() {
            quote::quote!(
                commands.insert(bevy_entitiles::serializing::ldtk::components::GlobalEntity);
            )
        } else {
            quote::quote!(
                use bevy::prelude::BuildChildren;
                let new_entity = commands.id();
                commands.commands().entity(level_entity).add_child(new_entity);
            )
        }
    };

    let callback_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == CALLBACK_ATTR);
    let callback = {
        if let Some(attr) = callback_attr {
            match &attr.meta {
                syn::Meta::List(meta) => {
                    let func = &meta.tokens;
                    quote::quote!(
                        #func(level_entity, commands, entity_instance, fields, asset_server, ldtk_manager);
                    )
                }
                _ => {
                    panic!("Callback attribute must be a list of functions!");
                }
            }
        } else {
            quote::quote!()
        }
    };

    let tag = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == LDTK_TAG_ATTR);

    let ctor = if tag.is_none() {
        let fields = match &input.data {
            syn::Data::Struct(data) => match &data.fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => panic!(
                    "LdtkEntity can only be derived for structs with named fields! \
                Or add #[ldtk_tag] for struct without named fields!"
                ),
            },
            _ => panic!("LdtkEntity can only be derived for structs"),
        };
        let mut fields_cton = Vec::new();

        for field in fields.iter() {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;

            let attr = field
                .attrs
                .iter()
                .find(|attr| attr.path().get_ident().unwrap() == LDTK_DEFAULT_ATTR);
            if attr.is_some() {
                continue;
            }

            let attr = field
                .attrs
                .iter()
                .find(|attr| attr.path().get_ident().unwrap() == LDTK_NAME_ATTR);
            if let Some(attr) = attr {
                fields_cton.push(expand_entity_fields_rename(
                    field_name, field_type, &attr.meta,
                ));
                continue;
            }

            fields_cton.push(expand_entity_fields(field_name, field_type));
        }

        let default = if fields_cton.len() < fields.len() {
            quote::quote!(..Default::default())
        } else {
            quote::quote!()
        };

        quote::quote!(
            Self {
                #(#fields_cton)*
                #default
            }
        )
    } else {
        quote::quote!(
            Self
        )
    };

    quote::quote! {
        impl LdtkEntity for #ty {
            fn initialize(
                level_entity: bevy::ecs::entity::Entity,
                commands: &mut bevy::ecs::system::EntityCommands,
                entity_instance: &bevy_entitiles::serializing::ldtk::json::level::EntityInstance,
                fields: &bevy::utils::HashMap<String, bevy_entitiles::serializing::ldtk::json::field::FieldInstance>,
                asset_server: &bevy::prelude::AssetServer,
                ldtk_manager: &bevy_entitiles::serializing::ldtk::resources::LdtkLevelManager,
            ) {
                #callback
                #spawn_sprite
                #global_entity

                commands.insert(#ctor);
            }
        }
    }
    .into()
}

pub fn expand_entity_fields(
    field_name: &syn::Ident,
    field_type: &syn::Type,
) -> proc_macro2::TokenStream {
    match field_type {
        syn::Type::Path(_) => {
            quote::quote!(
                #field_name: fields[&stringify!(#field_name).to_string()].clone().into(),
            )
        }
        _ => panic!("LdtkEntity attribute must be a path!"),
    }
}

pub fn expand_entity_fields_rename(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    ldtk_name: &syn::Meta,
) -> proc_macro2::TokenStream {
    let name = match ldtk_name {
        syn::Meta::NameValue(value) => &value.value,
        _ => panic!("LdtkEnum attribute must be a name value!"),
    };

    match field_type {
        syn::Type::Path(_) => {
            quote::quote!(
                #field_name: fields[#name].clone().into(),
            )
        }
        _ => panic!("LdtkEntity attribute must be a path!"),
    }
}
