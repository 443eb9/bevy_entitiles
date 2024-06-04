static TILED_DEFAULT_ATTR: &str = "tiled_default";
static CALLBACK_ATTR: &str = "callback";

pub fn expand_tiled_custom_tiles_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = input.ident;
    let attrs = &input.attrs;

    let callback_attr = attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == CALLBACK_ATTR);

    let callback = {
        if let Some(attr) = callback_attr {
            match &attr.meta {
                syn::Meta::List(meta) => {
                    let func = &meta.tokens;
                    quote::quote!(
                        #func(commands, custom_tile_instance, components, asset_server, tiled_assets, tiled_map);
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
        panic!("TiledCustomTile can only be derived for structs");
    };

    let ctor = {
        let syn::Fields::Named(fields) = &data_struct.fields else {
            panic!("TiledCustomTile can only be derived for structs with named fields!");
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

            fields_cton.push(expand_custom_tile_fields(field_name, field_type));
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
    };

    quote::quote! {
        impl bevy_entitiles::tiled::traits::TiledCustomTile for #ty {
            fn initialize(
                commands: &mut bevy::ecs::system::EntityCommands,
                custom_tile_instance: &bevy_entitiles::tiled::resources::TiledCustomTileInstance,
                components: &bevy::utils::HashMap<
                    String,
                    bevy_entitiles::tiled::xml::property::ClassInstance,
                >,
                asset_server: &bevy::prelude::AssetServer,
                tiled_assets: &bevy_entitiles::tiled::resources::TiledAssets,
                tiled_map: String,
            ) {
                #callback

                commands.insert(#ctor);
            }
        }
    }
    .into()
}

fn expand_custom_tile_fields(
    field_name: &syn::Ident,
    field_type: &syn::Type,
) -> proc_macro2::TokenStream {
    quote::quote!(
        #field_name: <#field_type as bevy_entitiles::tiled::traits::TiledClass>::create(&components),
    )
}
