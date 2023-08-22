use crate::utils::{
    find_types, is_ty_match, parse_args, try_process, warn, ImplBodyGenerator, TypeResult,
};
use fixing_rs_base::{
    gensrc::{NON_TERMINAL_PREFIX, SYMBOLIC_TERMINAL_PREFIX},
    grammar::{GrammarArena, GrammarSymbolsRef, SymbolType},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{spanned::Spanned, ImplItem, Result, Type};

pub fn impl_syntactic_processor_inner(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<TokenStream> {
    let impl_body_gen =
        ImplBodyGenerator::new(item, quote! { ::fixing_rs_base::reachability::GProcessor })?;
    let arena = GrammarArena::new();
    let mut grammar_str_storage = String::new();
    let ([prop_type], grammar_ast, grammar, grammar_types) =
        parse_args(args, &arena, &mut grammar_str_storage, ["g_prop"])?;
    let GrammarSymbolsRef {
        non_terminals,
        symbolic_terminals,
        ..
    } = grammar.get_symbol_ref();

    let mut m_non_terminal = HashMap::new();
    let mut m_symbolic_terminal = HashMap::new();

    let mut func_names = HashSet::new();

    let str_type = syn::parse_str::<Type>("Option<&str>").unwrap();

    for impl_item in impl_body_gen.input.items.iter() {
        match impl_item {
            ImplItem::Fn(ref method) => {
                let sig = &method.sig;
                let name = &sig.ident;
                func_names.insert(name.to_string());
                if let true = try_process::<1, _>(
                    method,
                    NON_TERMINAL_PREFIX,
                    non_terminals,
                    &mut m_non_terminal,
                    true,
                    false,
                    |symbol, [induction_id]| {
                        let (return_ty, args_calling) = find_types::<0>(
                            &grammar_ast,
                            &grammar_types,
                            symbol,
                            induction_id,
                            &name,
                        )?;
                        Ok(TypeResult {
                            return_ty,
                            args_calling,
                            args_before: vec![],
                        })
                    },
                )? {
                } else if let true = try_process::<0, _>(
                    method,
                    SYMBOLIC_TERMINAL_PREFIX,
                    symbolic_terminals,
                    &mut m_symbolic_terminal,
                    true,
                    false,
                    |symbol, _| {
                        let prop = grammar_types
                            .get::<0>(SymbolType::SymbolicTerminal, symbol.name())
                            .clone();
                        Ok(TypeResult {
                            return_ty: prop,
                            args_calling: vec![],
                            args_before: vec![str_type.clone()],
                        })
                    },
                )? {
                } else {
                    return Err(syn::Error::new(
                        name.span(),
                        format!("Invalid function name: {}", name),
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new(impl_item.span(), "Invalid item"));
            }
        }
    }

    let mut non_terminal_quotes = quote! {};
    for (symbol_id, procs) in m_non_terminal.into_iter() {
        let mut induction_quotes = quote! {};
        for (ref func_name, ref params, [ref induction_id]) in procs {
            let mut params_quotes = quote! {};
            for i in 0..*params {
                params_quotes.extend(quote! {
                    sub_types[ #i ].into_single_prop( stringify!(#func_name) ),
                })
            }
            induction_quotes.extend(quote! {
                #induction_id => self. #func_name ( #params_quotes ).assembly_prop_result(),
            });
        }
        non_terminal_quotes.extend(quote! {
            #symbol_id =>  {
                match induction_id {
                    #induction_quotes
                    _ => Self::PG::default().into_prop_result(),
                }
            },
        });
    }

    let mut symbolic_terminal_quotes = quote! {};
    for (symbol_id, procs) in m_symbolic_terminal.into_iter() {
        let (ref func_name, _, _) = procs[0];
        symbolic_terminal_quotes.extend(quote! {
            #symbol_id => { self. #func_name (literal).assembly_prop_result() },
        });
    }

    let default_ty = grammar_types.default_type();
    let default_ty = quote! { #default_ty };
    for node in grammar_ast.rules.iter() {
        let return_ty = grammar_types.get::<0>(SymbolType::NonTerminal, node.sym);
        let return_ty = quote! { #return_ty };
        if !is_ty_match(&return_ty, &default_ty) {
            for rule in node.alternatives.iter() {
                let nt_name = format!("{}{}_{}", NON_TERMINAL_PREFIX, node.sym, rule.id);
                if !func_names.contains(&nt_name) {
                    warn(
                        Span::call_site(),
                        format!(
                            "Function {} is not implemented ({}:{}).",
                            nt_name, node.sym, return_ty
                        ),
                    );
                }
            }
        }
    }
    for anno in grammar_ast.annos.iter() {
        let return_ty = grammar_types.get::<0>(SymbolType::SymbolicTerminal, anno.name);
        let return_ty = quote! { #return_ty };
        if !is_ty_match(&return_ty, &default_ty) {
            let st_name = format!("{}{}", SYMBOLIC_TERMINAL_PREFIX, anno.name);
            if !func_names.contains(&st_name) {
                warn(
                    Span::call_site(),
                    format!(
                        "Function {} is not implemented ({}:{}).",
                        st_name, anno.name, return_ty
                    ),
                );
            }
        }
    }

    Ok(impl_body_gen.generate(quote! (
        type PG = #prop_type;
        fn process_non_terminal(
            &self,
            symbol: ::fixing_rs_base::grammar::SymbolRef<'_>,
            induction_id: usize,
            sub_types: &[Self::PG],
        ) -> ::fixing_rs_base::props::PropResult<Self::PG> {
            use ::fixing_rs_base::props::{IntoPropResult, IntoUnionProp, IntoSingleProp, AssemblyPropResult};
            match symbol.symbol_type() {
                ::fixing_rs_base::grammar::SymbolType::NonTerminal => {
                    match symbol.symbol_id() {
                        #non_terminal_quotes
                        _ => Self::PG::default().into_prop_result(),
                    }
                }
                _ => panic!("Not a non-termial but invoked process_non_terminal: {:?}", symbol)
            }
        }

        fn process_symbolic_terminal(
            &self,
            symbol: ::fixing_rs_base::grammar::SymbolRef<'_>,
            literal: Option<&str>,
        ) -> ::fixing_rs_base::props::PropResult<Self::PG> {
            use ::fixing_rs_base::props::{IntoPropResult, IntoUnionProp, AssemblyPropResult};
            match symbol.symbol_type() {
                ::fixing_rs_base::grammar::SymbolType::SymbolicTerminal => {
                    match symbol.symbol_id() {
                        #symbolic_terminal_quotes
                        _ => Self::PG::default().into_prop_result(),
                    }
                }
                _ => panic!("Not a symbolic-termial but invoked process_symbolic_terminal: {:?}", symbol)
            }
        }
    ))?.into())
}
