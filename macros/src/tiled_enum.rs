static TILED_NAME_ATTR: &str = "tiled_name";

pub fn expand_tiled_enum_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let variants = match input.data {
        syn::Data::Enum(data) => data.variants,
        _ => panic!("TiledEnum can only be derived for enums"),
    };

    let mut variants_cton = Vec::new();
    for variant in variants.iter() {
        let variant_name = &variant.ident;

        let attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == TILED_NAME_ATTR);
        if let Some(attr) = attr {
            variants_cton.push(expand_enum_variant_rename(variant_name, &attr.meta));
        }

        variants_cton.push(expand_enum_variant(variant_name));
    }

    quote::quote!(
        impl bevy_entitiles::tiled::traits::TiledEnum for #ty {
            fn get_identifier(ident: &str) -> Self {
                match ident {
                    #(#variants_cton)*
                    _ => panic!("Unknown enum variant: {}", ident),
                }
            }
        }

        impl Into<#ty> for bevy_entitiles::tiled::xml::property::PropertyInstance {
            fn into(self) -> #ty {
                match self.value {
                    bevy_entitiles::tiled::xml::property::PropertyValue::Enum(_, x) => {
                        <#ty as bevy_entitiles::tiled::traits::TiledEnum>::get_identifier(&x)
                    }
                    _ => panic!("Expected Enum value!"),
                }
            }
        }
    )
    .into()
}

fn expand_enum_variant_rename(
    variant_name: &syn::Ident,
    ldtk_name: &syn::Meta,
) -> proc_macro2::TokenStream {
    let name = match ldtk_name {
        syn::Meta::NameValue(value) => &value.value,
        _ => panic!("TiledEnum attribute must be a name value!"),
    };

    quote::quote!(
        #name => Self::#variant_name,
    )
    .into()
}

fn expand_enum_variant(variant_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        stringify!(#variant_name) => Self::#variant_name,
    )
    .into()
}
