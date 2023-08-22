use crate::utils::if_camel_case_to_snake_case;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Fields, GenericParam, ItemEnum, Lifetime,
    LifetimeParam, Result, Token,
};

pub fn union_entity_inner(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<TokenStream> {
    let input = syn::parse::<ItemEnum>(item)?;

    let mut life_time_args = Punctuated::new();

    for g in input.generics.params.iter() {
        match g {
            GenericParam::Lifetime(l) => {
                if l.lifetime.to_string().as_str() == "'s" {
                    return Err(syn::Error::new(
                        l.span(),
                        "The lifetime parameter 's is reserved",
                    ));
                }
                life_time_args.push(GenericParam::Lifetime(LifetimeParam {
                    attrs: vec![],
                    lifetime: l.lifetime.clone(),
                    colon_token: None,
                    bounds: Punctuated::new(),
                }));
                life_time_args.push_punct(Token![,](l.span()));
            }
            _ => {
                return Err(syn::Error::new(
                    g.span(),
                    "Only lifetime parameters are allowed",
                ));
            }
        }
    }

    let mut variant_types = Vec::new();
    let mut variant_names = Vec::new();

    for v in input.variants.iter() {
        if v.discriminant.is_some() {
            return Err(syn::Error::new(v.span(), "Discriminant is not allowed"));
        }
        if v.ident.to_string().as_str() == "Empty" {
            return Err(syn::Error::new(
                v.span(),
                "The variant name 'Empty' is reserved",
            ));
        }
        match &v.fields {
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    variant_names.push(v.ident.clone());
                    variant_types.push(fields.unnamed.first().unwrap().ty.clone());
                } else {
                    return Err(syn::Error::new(
                        v.span(),
                        "Only one unnamed field is allowed",
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new(v.span(), "Only unnamed fields are allowed"));
            }
        }
    }

    let variant_names_snake_case = variant_names
        .iter()
        .map(|v| {
            let v_str = v.to_string();
            match if_camel_case_to_snake_case(v_str.as_str()) {
                Some(s) => Ident::new(s.as_str(), v.span()),
                None => v.clone(),
            }
        })
        .collect::<Vec<_>>();

    let input_attrs = &input.attrs;
    let input_vis = &input.vis;
    let input_name = &input.ident;
    let input_life_time = &input.generics.params;
    let input_where_clauses = &input.generics.where_clause;
    let symbolic_lifetime = Lifetime::new("'s", Span::call_site());

    let arena_name = Ident::new(
        &format!("{}Arena", input_name.to_string()),
        Span::call_site(),
    );

    let mut impl_provide_ref_arena = quote! {};
    for (variant_type, variant_name_snake_case) in
        variant_types.iter().zip(variant_names_snake_case.iter())
    {
        impl_provide_ref_arena.extend(
            quote! {
                impl <#input_life_time> ::fixing_rs_base::symbolic::ProvideRefArena<#variant_type> for #arena_name<#life_time_args> #input_where_clauses {
                    fn provide_ref_arena(&self) -> &::fixing_rs_base::utils::RefArena<#variant_type> {
                        &self.#variant_name_snake_case
                    }
                }
            }
        )
    }

    Ok(quote! {
        #(#input_attrs)*
        #[derive(Clone, Copy, Debug)]
        #input_vis enum #input_name<#symbolic_lifetime, #input_life_time> #input_where_clauses {
            Empty(::fixing_rs_base::utils::Pointer<#symbolic_lifetime, ::fixing_rs_base::symbolic::EmptyEntity>),
            #(#variant_names(::fixing_rs_base::utils::Pointer<#symbolic_lifetime, #variant_types>),)*
        }


        impl <#symbolic_lifetime, #input_life_time> #input_name<#symbolic_lifetime, #life_time_args> #input_where_clauses {
            pub fn variant_name(&self) -> &'static str {
                match self {
                    Self::Empty(_) => "Empty",
                    #(#input_name::#variant_names(_) => stringify!(#variant_names),)*
                }
            }
        }

        impl <#symbolic_lifetime, #input_life_time> Default for #input_name<#symbolic_lifetime, #life_time_args> #input_where_clauses {
            fn default() -> Self {
                Self::Empty(::fixing_rs_base::utils::Pointer::from_ptr(&::fixing_rs_base::symbolic::EmptyEntity))
            }
        }


        #(
            impl <#symbolic_lifetime, #input_life_time>
                ::fixing_rs_base::symbolic::ContainingSingleEntity<#symbolic_lifetime, #variant_types>
                for #input_name<#symbolic_lifetime, #life_time_args> #input_where_clauses {
                fn unwrap_single_entity(self) -> ::fixing_rs_base::utils::Pointer<'s, #variant_types> {
                    match self {
                        Self::#variant_names(e) => e,
                        _ => panic!("Expected variant {} but got {}", stringify!(#variant_names), self.variant_name()),
                    }
                }
                fn from_single_entity(entity: ::fixing_rs_base::utils::Pointer<'s, #variant_types>) -> Self {
                    Self::#variant_names(entity)
                }
            }
        )*

        impl <#symbolic_lifetime, #input_life_time>
            ::fixing_rs_base::symbolic::ContainingSingleEntity<#symbolic_lifetime, ::fixing_rs_base::symbolic::EmptyEntity>
            for #input_name<#symbolic_lifetime, #life_time_args> #input_where_clauses {
            fn unwrap_single_entity(self) -> ::fixing_rs_base::utils::Pointer<'s, ::fixing_rs_base::symbolic::EmptyEntity> {
                match self {
                    Self::Empty(e) => e,
                    _ => panic!("Expected variant {} but got {}", stringify!(Empty), self.variant_name()),
                }
            }
            fn from_single_entity(entity: ::fixing_rs_base::utils::Pointer<'s, ::fixing_rs_base::symbolic::EmptyEntity>) -> Self {
                Self::Empty(entity)
            }
        }


        #input_vis struct #arena_name<#input_life_time> #input_where_clauses {
            #(#variant_names_snake_case: ::fixing_rs_base::utils::RefArena<#variant_types>,)*
        }

        impl <#input_life_time> Default for #arena_name<#life_time_args> #input_where_clauses {
            fn default() -> Self {
                Self {
                    #(#variant_names_snake_case: ::fixing_rs_base::utils::RefArena::new(),)*
                }
            }
        }

        #impl_provide_ref_arena

        impl <#input_life_time> ::fixing_rs_base::symbolic::EntityUnionContainer for #arena_name<#life_time_args> #input_where_clauses {
        }

        impl<#symbolic_lifetime, #input_life_time>
            ::fixing_rs_base::symbolic::EntityUnion<#symbolic_lifetime>
            for #input_name<#symbolic_lifetime, #life_time_args>
            #input_where_clauses {
            type Container = #arena_name<#life_time_args>;

            fn construct_same(&self, world: &mut ::fixing_rs_base::symbolic::SymbolicWorld, container: &#symbolic_lifetime Self::Container) -> Self {
                use ::fixing_rs_base::symbolic::LogicEntity;
                match self {
                    Self::Empty(e) => *self,
                    #(#input_name::#variant_names(e) => Self::#variant_names(container.#variant_names_snake_case.alloc(e.construct_same(world))),)*
                }
            }

            fn assert_equals_to(&self, other: &Self) -> ::fixing_rs_base::symbolic::SymbolicRule{
                use ::fixing_rs_base::symbolic::LogicEntity;
                match (self, other) {
                    (Self::Empty(e1), Self::Empty(e2)) => ::fixing_rs_base::symbolic::SymbolicRule::new(),
                    #((#input_name::#variant_names(e1), #input_name::#variant_names(e2)) => e1.assert_equals_to(e2),)*
                    _ => {
                        let err = ::std::format!("assert equals to of different types: {} and {}.", self.variant_name(), other.variant_name());
                        ::log::error!("{}\n{:?}\n", err, ::backtrace::Backtrace::new());
                        panic!("{}", err)
                    },
                }
            }
        }
    })
}
