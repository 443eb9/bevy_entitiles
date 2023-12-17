static LDTK_DEFAULT_ATTR: &str = "ldtk_default";

pub fn expand_ldtk_entity_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = input.ident;
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => panic!("LdtkEntity can only be derived for structs with named fields"),
        },
        _ => panic!("LdtkEntity can only be derived for structs"),
    };
    let mut fields_cton = Vec::new();
    let mut field_index = 0;

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

        fields_cton.push(expand_entity_fields(field_name, field_type, field_index));
        field_index += 1;
    }

    let default = if fields_cton.len() < fields.len() {
        quote::quote!(..Default::default())
    } else {
        quote::quote!()
    };

    quote::quote! {
        impl LdtkEntity for #ty {
            fn initialize(
                commands: &mut bevy::ecs::system::EntityCommands,
                sprite: Option<bevy_entitiles::serializing::ldtk::entity::LdtkSprite>,
                entity_instance: &bevy_entitiles::serializing::ldtk::json::level::EntityInstance,
                asset_server: &bevy::prelude::AssetServer,
            ) -> Self {
                Self {
                    #(#fields_cton)*
                    #default
                }
            }
        }
    }
    .into()
}

pub fn expand_entity_fields(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    field_index: usize,
) -> proc_macro2::TokenStream {
    match field_type {
        syn::Type::Path(_) => {
            quote::quote!(
                #field_name: entity_instance.field_instances[#field_index].clone().into(),
            )
        }
        _ => panic!("LdtkEntity attribute must be a path!"),
    }
}
