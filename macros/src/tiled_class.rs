static TILED_NAME_ATTR: &str = "tiled_name";

pub fn expand_tiled_class_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let syn::Data::Struct(data_struct) = input.data else {
        panic!("TiledClass can only be derived for structs!");
    };

    let ty = &input.ident;

    let ctor = {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            let fields = &fields.named;
            let mut fields_cton = Vec::new();

            for field in fields {
                let field_name = field.ident.as_ref().unwrap();
                let name = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().get_ident().unwrap() == TILED_NAME_ATTR);
                if let Some(attr) = name {
                    fields_cton.push(expand_class_fields_rename(field_name, &attr.meta));
                } else {
                    fields_cton.push(expand_class_fields(&field_name));
                }
            }

            quote::quote!(
                if let Some(data) = &classes.get(stringify!(#ty)) {
                    Self {
                        #(#fields_cton)*
                    }
                } else { // Class not found: use default
                    Self::default()
                }
            )
        } else {
            quote::quote!(Self)
        }
    };

    quote::quote!(
        impl bevy_entitiles::tiled::traits::TiledClass for #ty {
            fn create(
                classes: &bevy::utils::HashMap<String, bevy_entitiles::tiled::xml::property::ClassInstance>,
            ) -> Self {
                #ctor
            }
        }
    )
    .into()
}

fn expand_class_fields(field_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        #field_name: if data.properties.contains_key(stringify!(#field_name)) {
            data.properties
                .get(stringify!(#field_name))
                .unwrap()
                .clone()
                .into()
        } else { // Field not found: use default value
            Self::default().#field_name
        },
    )
}

fn expand_class_fields_rename(
    field_name: &syn::Ident,
    meta: &syn::Meta,
) -> proc_macro2::TokenStream {
    let name = match meta {
        syn::Meta::NameValue(value) => &value.value,
        _ => panic!("tiled_name attribute must be a named value!"),
    };

    quote::quote!(
        #field_name: if data.properties.contains_key(stringify!(#name)) {
            data.properties
                .get(stringify!(#name))
                .unwrap()
                .clone()
                .into()
        } else { // Field not found: use default value
            Self::default().#field_name
        },
    )
}
