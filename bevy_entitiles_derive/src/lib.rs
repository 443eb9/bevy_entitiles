use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(LdtkEntity)]
pub fn derive_ldtk_entity(token: TokenStream) -> TokenStream {
    let input = syn::parse::<syn::DeriveInput>(token).unwrap();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let struct_name = input.ident;

    let tk = quote!(
        impl #impl_generics bevy_entitiles::serializing::ldtk::entity::LdtkEntity for #struct_name #ty_generics #where_clause {
            fn spawn(data: &bevy_entitiles::serializing::ldtk::json::level::EntityInstance, dyn_struct: &mut bevy::reflect::DynamicStruct) -> Self {
                let mut entity = Self::default();
                entity.spawn(data, dyn_struct);
                entity
            }
        }
    );
}
