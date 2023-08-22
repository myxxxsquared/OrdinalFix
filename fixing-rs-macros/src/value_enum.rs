use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Error, Result};

pub fn derive_value_enum_inner(input: proc_macro::TokenStream) -> Result<TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;

    let e = match &input.data {
        syn::Data::Enum(e) => e,
        _ => Err(Error::new(
            Span::call_site(),
            "ValueEnum can only be derived for enums.",
        ))?,
    };

    for v in e.variants.iter() {
        match v.fields {
            syn::Fields::Unit => {}
            _ => Err(Error::new(
                v.span(),
                "ValueEnum can only be derived for enums with unit variants.",
            ))?,
        }
        match v.discriminant {
            None => {}
            Some(_) => Err(Error::new(
                v.span(),
                "ValueEnum can only be derived for enums with no discriminant.",
            ))?,
        }
    }

    if input.generics.lt_token.is_some() {
        Err(Error::new(
            input.generics.span(),
            "ValueEnum can only be derived for enums without generic parameters.",
        ))?;
    }

    if input.generics.where_clause.is_some() {
        Err(Error::new(
            input.generics.span(),
            "ValueEnum can only be derived for enums without where clauses.",
        ))?;
    }

    let name = &input.ident;
    let variants = &e.variants;
    let num_variants = variants.len();

    let mut value_tokens = quote! {};
    let mut from_value_tokens = quote! {};

    for (value, name) in e.variants.iter().map(|v| &v.ident).enumerate() {
        value_tokens.extend(quote! {
            Self::#name => #value,
        });
        from_value_tokens.extend(quote! {
            #value => Some(Self::#name),
        });
    }

    let expanded = quote! {
        impl ::fixing_rs_base::utils::ValueEnum for #name {
            const N: usize = #num_variants;

            fn value(&self) -> usize {
                match self {
                    #value_tokens
                    _ => unreachable!(),
                }
            }

            fn from_value(value: usize) -> Option<Self> {
                match value {
                    #from_value_tokens
                    _ => None,
                }
            }

        }
    };
    Ok(expanded)
}
