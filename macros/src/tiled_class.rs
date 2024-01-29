static TILED_NAME_ATTR: &str = "tiled_name";
static TILED_DEFAULT_ATTR: &str = "tiled_default";

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

                let default = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().get_ident().unwrap() == TILED_DEFAULT_ATTR);
                if default.is_some() {
                    continue;
                }

                let name = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().get_ident().unwrap() == TILED_NAME_ATTR);
                if let Some(attr) = name {
                    fields_cton.push(expand_class_fields_rename(field_name, &attr.meta));
                    continue;
                }

                fields_cton.push(expand_class_fields(&field_name));
            }

            if fields_cton.len() < fields.len() {
                fields_cton.push(quote::quote!(..Default::default()));
            }

            quote::quote!(
                let data = &classes[stringify!(#ty)];
                Self {
                    #(#fields_cton)*
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
        #field_name: data.properties.get(stringify!(#field_name)).unwrap_or_else(|| panic!("{}", stringify!(#field_name))).clone().into(),
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
        #field_name: data.properties[#name].clone().into(),
    )
}
