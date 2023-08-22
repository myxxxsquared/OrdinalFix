use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_prop_inner(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let generics_lt = &generics.lt_token;
    let generics_gt = &generics.gt_token;
    let generics_params = &generics.params;
    let generics_wheres = &generics.where_clause;
    let expanded = quote! {
        impl #generics_lt #generics_params #generics_gt ::fixing_rs_base::props::Prop for #name #generics_lt #generics_params #generics_gt #generics_wheres {}
    };
    expanded.into()
}
