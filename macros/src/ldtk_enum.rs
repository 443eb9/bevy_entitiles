static LDTK_NAME_ATTR: &str = "ldtk_name";
static WRAPPER_DERIVE_ATTR: &str = "wrapper_derive";

pub fn expand_ldtk_enum_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let variants = match input.data {
        syn::Data::Enum(data) => data.variants,
        _ => panic!("LdtkEnum can only be derived for enums"),
    };

    let derive_attrs = &input
        .attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == WRAPPER_DERIVE_ATTR);
    let derives = {
        if let Some(attr) = derive_attrs {
            let syn::Meta::List(meta) = &attr.meta else {
                panic!("Derive attribute must be a list of derives!");
            };

            let tokens = &meta.tokens;
            quote::quote!(
                #[derive(#tokens)]
            )
        } else {
            quote::quote!()
        }
    };

    let mut variants_cton = Vec::new();
    for variant in variants.iter() {
        let variant_name = &variant.ident;

        let attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == LDTK_NAME_ATTR);
        if let Some(attr) = attr {
            variants_cton.push(expand_enum_variant_rename(variant_name, &attr.meta));
        }

        variants_cton.push(expand_enum_variant(variant_name));
    }

    let wrapper_indets = vec![
        syn::Ident::new(&format!("{}Option", ty), ty.span()),
        syn::Ident::new(&format!("{}Vec", ty), ty.span()),
        syn::Ident::new(&format!("{}OptionVec", ty), ty.span()),
    ];
    let wrappers = create_wrappers(ty, &wrapper_indets, &derives);

    let impl_intos = vec![
        impl_into_enum(ty),
        impl_into_enum_opt(ty, &wrapper_indets[0]),
        impl_into_enum_vec(ty, &wrapper_indets[1]),
        impl_into_enum_opt_vec(ty, &wrapper_indets[2]),
    ];

    quote::quote!(
        impl bevy_entitiles::ldtk::traits::LdtkEnum for #ty {
            fn get_identifier(ident: &str) -> Self {
                match ident {
                    #(#variants_cton)*
                    _ => panic!("Unknown enum variant: {}", ident),
                }
            }
        }

        #(#wrappers)*
        #(#impl_intos)*
    )
    .into()
}

fn expand_enum_variant_rename(
    variant_name: &syn::Ident,
    ldtk_name: &syn::Meta,
) -> proc_macro2::TokenStream {
    let name = match ldtk_name {
        syn::Meta::NameValue(value) => &value.value,
        _ => panic!("LdtkEnum attribute must be a name value!"),
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

fn create_wrappers(
    ty: &syn::Ident,
    wrapper_idents: &Vec<syn::Ident>,
    derives: &proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    let mut wrappers = Vec::new();

    let ident_opt = &wrapper_idents[0];
    let ident_vec = &wrapper_idents[1];
    let ident_opt_vec = &wrapper_idents[2];

    wrappers.push(quote::quote!(
        #derives
        pub struct #ident_opt(pub Option<#ty>);
        #derives
        pub struct #ident_vec(pub Vec<#ty>);
        #derives
        pub struct #ident_opt_vec(pub Option<Vec<#ty>>);
    ));

    wrappers
}

fn impl_into_enum(ty: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        impl Into<#ty> for bevy_entitiles::ldtk::json::field::FieldInstance {
            fn into(self) -> #ty {
                match self.value {
                    Some(v) => match v {
                        bevy_entitiles::ldtk::json::field::FieldValue::LocalEnum(
                            (_, i),
                        ) => <#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i),
                        bevy_entitiles::ldtk::json::field::FieldValue::ExternEnum(
                            (_, i),
                        ) => <#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i),
                        _ => panic!("Expected value!"),
                    },
                    None => panic!("Expected value!"),
                }
            }
        }
    )
    .into()
}

fn impl_into_enum_opt(ty: &syn::Ident, wrapper: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        impl Into<#wrapper> for bevy_entitiles::ldtk::json::field::FieldInstance {
            fn into(self) -> #wrapper {
                match self.value {
                    Some(v) => match v {
                        bevy_entitiles::ldtk::json::field::FieldValue::LocalEnum(
                            (_, i),
                        ) => #wrapper(Some(<#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i))),
                        bevy_entitiles::ldtk::json::field::FieldValue::ExternEnum(
                            (_, i),
                        ) => #wrapper(Some(<#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i))),
                        _ => panic!("Expected value!"),
                    },
                    None => #wrapper(None),
                }
            }
        }
    )
    .into()
}

fn impl_into_enum_vec(ty: &syn::Ident, wrapper: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        impl Into<#wrapper> for bevy_entitiles::ldtk::json::field::FieldInstance {
            fn into(self) -> #wrapper {
                match self.value {
                    Some(v) => match v {
                        bevy_entitiles::ldtk::json::field::FieldValue::LocalEnumArray((_, i)) => {
                            #wrapper(
                                i
                                .iter()
                                .map(|i| <#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i)).collect()
                            )
                        }
                        bevy_entitiles::ldtk::json::field::FieldValue::ExternEnumArray((_, i)) => {
                            #wrapper(
                                i
                                .iter()
                                .map(|i| <#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i)).collect()
                            )
                        }
                        _ => panic!("Expected value!"),
                    },
                    None => panic!("Expected value!"),
                }
            }
        }
    ).into()
}

fn impl_into_enum_opt_vec(ty: &syn::Ident, wrapper: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        impl Into<#wrapper> for bevy_entitiles::ldtk::json::field::FieldInstance {
            fn into(self) -> #wrapper {
                match self.value {
                    Some(v) => match v {
                        bevy_entitiles::ldtk::json::field::FieldValue::LocalEnumArray((_, i)) => {
                            #wrapper(Some(
                                i
                                .iter()
                                .map(|i| <#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i)).collect())
                            )
                        }
                        bevy_entitiles::ldtk::json::field::FieldValue::ExternEnumArray((_, i)) => {
                            #wrapper(Some(
                                i
                                .iter()
                                .map(|i| <#ty as bevy_entitiles::ldtk::traits::LdtkEnum>::get_identifier(&i)).collect())
                            )
                        }
                        _ => panic!("Expected value!"),
                    },
                    None => #wrapper(None),
                }
            }
        }
    ).into()
}
