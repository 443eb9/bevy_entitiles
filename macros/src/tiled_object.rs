const TILED_DEFAULT_ATTR: &str = "tiled_default";
const INSTANTIATE_SHAPE_ATTR: &str = "instantiate_shape";
const SPAWN_SPRITE_ATTR: &str = "spawn_sprite";
const GLOBAL_OBJECT_ATTR: &str = "global_object";
const CALLBACK_ATTR: &str = "callback";

pub fn expand_tiled_objects_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = input.ident;
    let attrs = &input.attrs;

    let shape_as_collider_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == INSTANTIATE_SHAPE_ATTR);

    let spawn_sprite_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == SPAWN_SPRITE_ATTR);

    let global_object_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == GLOBAL_OBJECT_ATTR);

    let callback_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == CALLBACK_ATTR);

    let instantiate_shape = {
        if shape_as_collider_attr.is_some() {
            quote::quote!(
                object_instance.instantiate_shape(commands);
            )
        } else {
            quote::quote!()
        }
    };

    let spawn_sprite = {
        if spawn_sprite_attr.is_some() {
            quote::quote!(
                object_instance.spawn_sprite(commands, tiled_assets);
            )
        } else {
            quote::quote!()
        }
    };

    let global_object = {
        if global_object_attr.is_some() {
            quote::quote!(
                commands.insert(bevy_entitiles::tiled::components::TiledGlobalObject);
            )
        } else {
            quote::quote!()
        }
    };

    let callback = {
        if let Some(attr) = callback_attr {
            match &attr.meta {
                syn::Meta::List(meta) => {
                    let func = &meta.tokens;
                    quote::quote!(
                        #func(commands, object_instance, components, asset_server, tiled_assets, tiled_map);
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

    let syn::Data::Struct(data_struct) = &input.data else {
        panic!("TiledObject can only be derived for structs");
    };

    let ctor = generate_constructor(data_struct);

    quote::quote! {
        impl bevy_entitiles::tiled::traits::TiledObject for #ty {
            fn initialize(
                commands: &mut bevy::ecs::system::EntityCommands,
                object_instance: &bevy_entitiles::tiled::xml::layer::TiledObjectInstance,
                components: &bevy::utils::HashMap<
                    String,
                    bevy_entitiles::tiled::xml::property::ClassInstance,
                >,
                asset_server: &bevy::prelude::AssetServer,
                tiled_assets: &bevy_entitiles::tiled::resources::TiledAssets,
            ) {
                #callback
                #spawn_sprite
                #instantiate_shape
                #global_object

                commands.insert(#ctor);
            }
        }
    }
    .into()
}

fn expand_object_fields(
    field_name: &syn::Ident,
    field_type: &syn::Type,
) -> proc_macro2::TokenStream {
    quote::quote!(
        #field_name: <#field_type as bevy_entitiles::tiled::traits::TiledClass>::create(&components),
    )
}

fn generate_constructor(data_struct: &syn::DataStruct) -> proc_macro2::TokenStream {
    if data_struct.fields.is_empty() {
        return quote::quote!(Self);
    }

    let syn::Fields::Named(fields) = &data_struct.fields else {
        panic!("TiledObject can only be derived for structs with named fields or unit structs!");
    };
    let fields = &fields.named;
    let mut fields_cton = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let default = field
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == TILED_DEFAULT_ATTR);
        if default.is_some() {
            continue;
        }

        fields_cton.push(expand_object_fields(field_name, field_type));
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
}
