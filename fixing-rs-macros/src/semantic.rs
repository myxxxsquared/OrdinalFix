use crate::utils::{
    check_method, check_type_match, find_types, is_ty_match, parse_args, try_process, warn,
    ImplBodyGenerator, TypeResult,
};
use fixing_rs_base::{
    gensrc::{
        NON_TERMINAL_INH_PREFIX, NON_TERMINAL_SYN_PREFIX, ROOT_INH_NAME,
        SYMBOLIC_TERMINAL_GEN_PREFIX, SYMBOLIC_TERMINAL_SYN_PREFIX,
    },
    grammar::{GrammarArena, GrammarSymbolsRef, SymbolType},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{spanned::Spanned, ImplItem, Result, Type};

pub fn impl_semantic_processor_inner(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<TokenStream> {
    let impl_body_gen =
        ImplBodyGenerator::new(item, quote! { ::fixing_rs_base::reachability::SProcessor })?;

    let arena = GrammarArena::new();
    let mut grammar_str_storage = String::new();
    let ([gprop_type, siprop_type, ssprop_type], grammar_ast, grammar, grammar_types) = parse_args(
        args,
        &arena,
        &mut grammar_str_storage,
        ["g_prop", "si_prop", "ss_prop"],
    )?;
    let GrammarSymbolsRef {
        non_terminals,
        symbolic_terminals,
        start_symbol,
        ..
    } = grammar.get_symbol_ref();

    let mut m_non_terminal_syn = HashMap::new();
    let mut m_non_terminal_inh = HashMap::new();
    let mut m_symbolic_terminal = HashMap::new();
    let mut m_symbolic_terminal_gen = HashMap::new();
    let mut root_inh = None;

    let mut func_names = HashSet::new();

    let gprop_ty = syn::parse2::<Type>(quote! {&PropArray< #gprop_type >}).unwrap();
    let str_type = syn::parse_str::<Type>("Option<&str>").unwrap();
    let string_type = syn::parse_str::<Type>("String").unwrap();

    for impl_item in impl_body_gen.input.items.iter() {
        match impl_item {
            ImplItem::Fn(ref method) => {
                let sig = &method.sig;
                let name = &sig.ident;
                func_names.insert(name.to_string());
                if try_process::<1, _>(
                    method,
                    NON_TERMINAL_SYN_PREFIX,
                    non_terminals,
                    &mut m_non_terminal_syn,
                    true,
                    false,
                    |symbol, [induction_id]| {
                        let (return_ty, args_calling) = find_types::<2>(
                            &grammar_ast,
                            &grammar_types,
                            symbol,
                            induction_id,
                            &name,
                        )?;
                        let inh = grammar_types.get_non_inh(symbol.name());
                        let inh = syn::parse2::<Type>(quote! { &#inh }).unwrap();
                        Ok(TypeResult {
                            return_ty,
                            args_calling,
                            args_before: vec![gprop_ty.clone(), inh],
                        })
                    },
                )? {
                } else if try_process::<2, _>(
                    method,
                    NON_TERMINAL_INH_PREFIX,
                    non_terminals,
                    &mut m_non_terminal_inh,
                    true,
                    false,
                    |symbol, [induction_id, induction_loc]| {
                        let (_, mut args_calling) = find_types::<2>(
                            &grammar_ast,
                            &grammar_types,
                            symbol,
                            induction_id,
                            &name,
                        )?;
                        let inh = grammar_types.get_non_inh(symbol.name());
                        let inh = syn::parse2::<Type>(quote! { &#inh }).unwrap();
                        let return_ty = find_types::<1>(
                            &grammar_ast,
                            &grammar_types,
                            symbol,
                            induction_id,
                            &name,
                        )?
                        .1[induction_loc]
                            .clone();
                        if induction_loc > args_calling.len() {
                            return Err(syn::Error::new(
                                name.span(),
                                format!(
                                    "Invalid induction location: {}:{}",
                                    symbol.name(),
                                    induction_id
                                ),
                            ));
                        }
                        args_calling.truncate(induction_loc);
                        Ok(TypeResult {
                            return_ty,
                            args_calling,
                            args_before: vec![gprop_ty.clone(), inh],
                        })
                    },
                )? {
                } else if try_process::<0, _>(
                    method,
                    SYMBOLIC_TERMINAL_SYN_PREFIX,
                    symbolic_terminals,
                    &mut m_symbolic_terminal,
                    true,
                    false,
                    |symbol, _| {
                        let inh =
                            grammar_types.get::<1>(SymbolType::SymbolicTerminal, symbol.name());
                        let inh = syn::parse2::<Type>(quote! { &#inh }).unwrap();
                        let syn =
                            grammar_types.get::<2>(SymbolType::SymbolicTerminal, symbol.name());
                        let syn = syn.clone();
                        Ok(TypeResult {
                            return_ty: syn,
                            args_calling: vec![],
                            args_before: vec![gprop_ty.clone(), inh, str_type.clone()],
                        })
                    },
                )? {
                } else if try_process::<0, _>(
                    method,
                    SYMBOLIC_TERMINAL_GEN_PREFIX,
                    symbolic_terminals,
                    &mut m_symbolic_terminal_gen,
                    true,
                    false,
                    |symbol, _| {
                        let inh =
                            grammar_types.get::<1>(SymbolType::SymbolicTerminal, symbol.name());
                        let inh = syn::parse2::<Type>(quote! { &#inh }).unwrap();
                        let syn =
                            grammar_types.get::<2>(SymbolType::SymbolicTerminal, symbol.name());
                        let syn = syn::parse2::<Type>(quote! { &#syn }).unwrap();
                        Ok(TypeResult {
                            return_ty: string_type.clone(),
                            args_calling: vec![],
                            args_before: vec![gprop_ty.clone(), inh, syn, str_type.clone()],
                        })
                    },
                )? {
                } else if name.to_string() == ROOT_INH_NAME {
                    root_inh = Some(name.clone());
                    check_method(&method, false);
                    let root_inh_ty = grammar_types.get_non_inh(start_symbol.name());
                    let root_inh_imp = &method.sig.output;
                    match root_inh_imp {
                        syn::ReturnType::Default => {
                            warn(&method.sig, "Return type is not specified");
                        }
                        syn::ReturnType::Type(_, ty) => {
                            check_type_match(ty.as_ref(), root_inh_ty, false);
                        }
                    }
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

    let mut non_terminal_inh_quotes = quote! {};
    for (symbol_id, procs) in m_non_terminal_inh.into_iter() {
        let mut procs_group = HashMap::new();
        for (func_name, params, [induction_id, induction_loc]) in procs {
            if !procs_group.contains_key(&induction_id) {
                procs_group.insert(induction_id, Vec::new());
            }
            procs_group
                .get_mut(&induction_id)
                .unwrap()
                .push((func_name, params, induction_loc));
        }

        let mut induction_quotes = quote! {};
        for (induction_id, procs) in procs_group.into_iter() {
            let mut induction_loc_quotes = quote! {};
            for (func_name, params, induction_loc) in procs {
                let mut params_quotes = quote! {};
                for i in 0..params {
                    params_quotes.extend(quote! {
                        sub_types[ #i ].into_single_prop(stringify!(#func_name)),
                    })
                }
                induction_loc_quotes.extend(quote! {
                    #induction_loc => {
                        ::fixing_rs_base::utils::slice_check_len(sub_types, #params, stringify!(#func_name));
                        self. #func_name ( gprop, inh.into_single_prop(stringify!(#func_name)), #params_quotes ).assembly_prop_result()
                    },
                });
            }
            induction_quotes.extend(quote! {
                #induction_id => match induction_loc {
                    #induction_loc_quotes
                    _ => inh.clone().into_prop_result(),
                },
            });
        }

        non_terminal_inh_quotes.extend(quote! {
            #symbol_id =>  {
                match induction_id {
                    #induction_quotes
                    _ => inh.clone().into_prop_result(),
                }
            },
        });
    }

    let mut non_terminal_syn_quotes = quote! {};
    for (symbol_id, procs) in m_non_terminal_syn.into_iter() {
        let mut induction_quotes = quote! {};
        for (ref func_name, ref params, [ref induction_id]) in procs {
            let mut params_quotes = quote! {};
            for i in 0..*params {
                params_quotes.extend(quote! {
                    sub_types[ #i ].into_single_prop(stringify!(#func_name)),
                })
            }
            induction_quotes.extend( quote! {
                #induction_id => {
                    ::fixing_rs_base::utils::slice_check_len(sub_types, #params, stringify!(#func_name));
                    self. #func_name ( gprop, inh.into_single_prop(stringify!(#func_name)), #params_quotes ).assembly_prop_result()
                }
            });
        }
        non_terminal_syn_quotes.extend(quote! {
            #symbol_id =>  {
                match induction_id {
                    #induction_quotes
                    _ => Self::PSS::default().into_prop_result(),
                }
            },
        });
    }

    let mut symbolic_terminal_quotes = quote! {};
    for (symbol_id, procs) in m_symbolic_terminal.into_iter() {
        let (ref func_name, _, _) = procs[0];
        symbolic_terminal_quotes.extend(quote! {
            #symbol_id => { self. #func_name ( gprop, inh.into_single_prop(stringify!(#func_name)), literal).assembly_prop_result() },
        });
    }

    let mut symbolic_terminal_gen_quotes = quote! {};
    for (symbol_id, procs) in m_symbolic_terminal_gen.into_iter() {
        let (ref func_name, _, _) = procs[0];
        symbolic_terminal_gen_quotes.extend(quote! {
            #symbol_id => { self. #func_name ( gprop, inh.into_single_prop(stringify!(#func_name)), syn.into_single_prop(stringify!(#func_name)), literal) },
        });
    }

    let default_ty = grammar_types.default_type();
    let default_ty = quote! { #default_ty };

    let mut root_inh_quotes = quote! {};
    if let Some(root_inh) = root_inh {
        root_inh_quotes.extend(quote! {
            use ::fixing_rs_base::props::IntoUnionProp;
            self. #root_inh ().into_union_prop()
        });
    } else {
        root_inh_quotes.extend(quote! {
            <Self::PSI as Default>::default()
        });
        let root_inh_ty = grammar_types.get_non_inh(start_symbol.name());
        let root_inh_ty = quote! { #root_inh_ty };
        if !is_ty_match(&root_inh_ty, &default_ty) {
            warn(
                Span::call_site(),
                format!(
                    "Function rooti is not implemented ({}:{}).",
                    start_symbol.name(),
                    root_inh_ty
                ),
            );
        }
    }

    for node in grammar_ast.rules.iter() {
        let return_ty = grammar_types.get::<2>(SymbolType::NonTerminal, node.sym);
        let return_ty = quote! { #return_ty };
        let inh_ty = grammar_types.get::<1>(SymbolType::NonTerminal, node.sym);
        let inh_ty = quote! { #inh_ty };
        if !is_ty_match(&return_ty, &default_ty) {
            for rule in node.alternatives.iter() {
                let nts_name = format!("{}{}_{}", NON_TERMINAL_SYN_PREFIX, node.sym, rule.id);
                if !func_names.contains(&nts_name) {
                    warn(
                        Span::call_site(),
                        format!(
                            "Function {} is not implemented ({}:{}).",
                            nts_name, node.sym, return_ty
                        ),
                    );
                }
            }
        }
        for rule in node.alternatives.iter() {
            for (i, element) in rule.elements.iter().enumerate() {
                match element.element_type {
                    SymbolType::LiteralTerminal => continue,
                    _ => {}
                }
                let ele_ty = grammar_types.get_ele_inh(element);
                let ele_ty = quote! { #ele_ty };
                if !is_ty_match(&ele_ty, &inh_ty) {
                    let nti_name =
                        format!("{}{}_{}_{}", NON_TERMINAL_INH_PREFIX, node.sym, rule.id, i);
                    if !func_names.contains(&nti_name) {
                        warn(
                            Span::call_site(),
                            format!(
                                "Function {} is not implemented ({}:{}, {}:{}).",
                                nti_name, node.sym, inh_ty, element.element_value, ele_ty
                            ),
                        );
                    }
                }
            }
        }
    }
    for anno in grammar_ast.annos.iter() {
        let return_ty = grammar_types.get::<2>(SymbolType::SymbolicTerminal, anno.name);
        let return_ty = quote! { #return_ty };
        if !is_ty_match(&return_ty, &default_ty) {
            let sts_name = format!("{}{}", SYMBOLIC_TERMINAL_SYN_PREFIX, anno.name);
            if !func_names.contains(&sts_name) {
                warn(
                    Span::call_site(),
                    format!(
                        "Function {} is not implemented ({}:{}).",
                        sts_name, anno.name, return_ty
                    ),
                );
            }
        }
    }
    for st in symbolic_terminals.values() {
        let stg_name = format!("{}{}", SYMBOLIC_TERMINAL_GEN_PREFIX, st.name());
        if !func_names.contains(&stg_name) {
            warn(
                Span::call_site(),
                format!("Function {} is not implemented.", stg_name,),
            );
        }
    }

    Ok(impl_body_gen.generate(quote!(
        type PG = #gprop_type;
        type PSI = #siprop_type;
        type PSS = #ssprop_type;

        fn process_non_terminal_inh(
            &self,
            symbol: ::fixing_rs_base::grammar::SymbolRef<'_>,
            gprop: &::fixing_rs_base::props::PropArray<Self::PG>,
            induction_id: usize,
            induction_loc: usize,
            inh: &Self::PSI,
            sub_types: &[Self::PSS],
        ) -> ::fixing_rs_base::props::PropResult<Self::PSI> {
            use ::fixing_rs_base::props::{IntoPropResult, IntoUnionProp, IntoSingleProp, AssemblyPropResult};
            match symbol.symbol_type() {
                ::fixing_rs_base::grammar::SymbolType::NonTerminal => {
                    match symbol.symbol_id() {
                        #non_terminal_inh_quotes
                        _ => inh.clone().into_prop_result(),
                    }
                }
                _ => panic!("Not a non-termial but invoked process_non_terminal: {:?}", symbol)
            }
        }

        fn process_non_terminal_syn(
            &self,
            symbol: ::fixing_rs_base::grammar::SymbolRef<'_>,
            gprop: &::fixing_rs_base::props::PropArray<Self::PG>,
            induction_id: usize,
            inh: &Self::PSI,
            sub_types: &[Self::PSS],
        ) -> ::fixing_rs_base::props::PropResult<Self::PSS> {
            use ::fixing_rs_base::props::{IntoPropResult, IntoUnionProp, IntoSingleProp, AssemblyPropResult};
            match symbol.symbol_type() {
                ::fixing_rs_base::grammar::SymbolType::NonTerminal => {
                    match symbol.symbol_id() {
                        #non_terminal_syn_quotes
                        _ => Self::PSS::default().into_prop_result(),
                    }
                }
                _ => panic!("Not a non-termial but invoked process_non_terminal: {:?}", symbol)
            }
        }

        fn process_symbolic_terminal_syn(
            &self,
            symbol: ::fixing_rs_base::grammar::SymbolRef<'_>,
            gprop: &::fixing_rs_base::props::PropArray<Self::PG>,
            inh: &Self::PSI,
            literal: Option<&str>,
        ) -> ::fixing_rs_base::props::PropResult<Self::PSS> {
            use ::fixing_rs_base::props::{IntoPropResult, IntoUnionProp, IntoSingleProp, AssemblyPropResult};
            match symbol.symbol_type() {
                ::fixing_rs_base::grammar::SymbolType::SymbolicTerminal => {
                    match symbol.symbol_id() {
                        #symbolic_terminal_quotes
                        _ => Self::PSS::default().into_prop_result(),
                    }
                }
                _ => panic!("Not a symbolic-termial but invoked process_symbolic_terminal: {:?}", symbol)
            }
        }

        fn process_symbolic_terminal_gen(
            &self,
            symbol: ::fixing_rs_base::grammar::SymbolRef<'_>,
            gprop: &::fixing_rs_base::props::PropArray<Self::PG>,
            inh: &Self::PSI,
            syn: &Self::PSS,
            literal: Option<&str>,
        ) -> String {
            use ::fixing_rs_base::props::{IntoSingleProp};
            match symbol.symbol_type() {
                ::fixing_rs_base::grammar::SymbolType::SymbolicTerminal => {
                    match symbol.symbol_id() {
                        #symbolic_terminal_gen_quotes
                        _ => match literal {
                            Some(literal) => literal.to_string(),
                            None => symbol.name().to_string(),
                        },
                    }
                }
                _ => panic!("Not a symbolic-termial but invoked process_symbolic_terminal: {:?}", symbol)
            }
        }

        fn process_root_inh(&self) -> Self::PSI {
            #root_inh_quotes
        }
    ))?)
}
