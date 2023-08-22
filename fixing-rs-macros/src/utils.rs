use fixing_rs_base::{
    gensrc::GrammarTypes,
    grammar::{Grammar, GrammarArena, Symbol, SymbolMap, SymbolType},
    parsing::{ast::GrammarFile, parser_grammar::GrammarFileParser},
};
use proc_macro2::{self, Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse::Parser, punctuated::Punctuated, spanned::Spanned, Error, GenericParam, Generics,
    ImplItemFn, ItemImpl, LitStr, Result, ReturnType, Signature, Token, Type, WhereClause,
};

pub struct TypeResult {
    pub return_ty: Type,
    pub args_before: Vec<Type>,
    pub args_calling: Vec<Type>,
}

#[cfg(nightly)]
pub fn warn(s: impl Spanned, t: impl Into<String>) {
    let t = t.into();
    s.span().unwrap().warning(t).emit();
}

#[cfg(not(nightly))]
pub fn warn(_s: impl Spanned, t: impl Into<String>) {
    drop(t.into());
}

pub fn warn_mismatch(target: &TokenStream, expected: &TokenStream) {
    warn(
        target,
        format!("Type mismatch: expected {}, got {}", expected, target),
    );
}

pub fn is_ty_match(target: &TokenStream, expected: &TokenStream) -> bool {
    target.to_string() == expected.to_string()
}

pub fn check_type_match(target: &Type, expected: &Type, add_ref: bool) {
    let target = quote! {#target};
    let expected = if add_ref {
        quote! {&#expected}
    } else {
        quote! {#expected}
    };
    if !is_ty_match(&target, &expected) {
        warn_mismatch(&target, &expected);
    }
}

pub fn check_method(method: &ImplItemFn, generic_life_time: bool) {
    if method.defaultness.is_some() {
        warn(&method.defaultness, "Default method is not supported");
    }
    match method.vis {
        syn::Visibility::Inherited => {}
        _ => warn(&method.vis, "Visibility is not inherited"),
    }
    if method.sig.asyncness.is_some() {
        warn(&method.sig.asyncness, "Async method is not supported");
    }
    if method.sig.variadic.is_some() {
        warn(&method.sig.variadic, "Variadic method is not supported");
    }
    if generic_life_time {
        if method.sig.generics.params.len() != 1 {
            warn(&method.sig.generics, "Generic parameter is not supported");
        } else {
            match &method.sig.generics.params[0] {
                GenericParam::Lifetime(lt) => {
                    if lt.lifetime.ident != "s" {
                        warn(&lt.lifetime, "Generic lifetime is not 's'");
                    }
                }
                _ => warn(&method.sig.generics, "Generic parameter is not supported"),
            }
        }
    } else {
        if method.sig.generics.params.len() != 0 {
            warn(&method.sig.generics, "Generic parameter is not supported");
        }
    }
}

fn check_return_type_match(target: &Signature, expected: &Type) {
    let target = match &target.output {
        ReturnType::Default => {
            warn(target, "Return type is not specified");
            return;
        }
        ReturnType::Type(_, ty) => ty.as_ref(),
    };
    let target = quote! {#target};
    let expected = quote! {#expected};
    if is_ty_match(&target, &expected) {
        return;
    }
    let option_expected = quote! {Option<#expected>}.into();
    if is_ty_match(&target, &option_expected) {
        return;
    }
    let vec_excepted = quote! {Vec<#expected>};
    if is_ty_match(&target, &vec_excepted) {
        return;
    }
    warn_mismatch(&target, &expected);
}

pub fn try_process<'a, 'b, const N: usize, P>(
    method: &ImplItemFn,
    prefix_pattern: &str,
    symbols: &'a SymbolMap<'a>,
    m: &mut HashMap<usize, Vec<(Ident, usize, [usize; N])>>,
    add_ref_on_before: bool,
    generic_life_time: bool,
    len_processor: P,
) -> Result<bool>
where
    P: Fn(&'a Symbol<'a>, [usize; N]) -> Result<TypeResult>,
{
    let name = &method.sig.ident;
    let name_str = name.to_string();
    if !name_str.starts_with(prefix_pattern) {
        return Ok(false);
    }

    let mut remaining = &name_str[prefix_pattern.len()..];

    let mut params = [0; N];
    for i in (0..N).rev() {
        if let Some(loc) = remaining.rfind("_") {
            let p = &remaining[loc + 1..];
            match p.parse() {
                Ok(p) => params[i] = p,
                Err(_) => {
                    return Err(Error::new(
                        name.span(),
                        format!("Cannot parse param {} in {}", p, name),
                    ))
                }
            }
            remaining = &remaining[..loc];
        }
    }

    let symbol_name = remaining;
    if let Some(symbol) = symbols.get(symbol_name) {
        let symbol_id = symbol.symbol_id();
        let call_types = len_processor(symbol, params)?;

        check_method(method, generic_life_time);
        check_return_type_match(&method.sig, &call_types.return_ty);

        let inputs = &method.sig.inputs;
        let total_len = call_types.args_before.len() + call_types.args_calling.len() + 1;
        if inputs.len() != total_len {
            warn(
                inputs,
                format!(
                    "Argument length mismatch: expected {}, got {}",
                    total_len,
                    inputs.len()
                ),
            );
        }
        for (i, input) in inputs.iter().enumerate() {
            if i == 0 {
                match input {
                    syn::FnArg::Receiver(receiver) => {
                        if receiver.reference.is_none() {
                            warn(receiver, "Self should be a reference");
                        }
                        if receiver.mutability.is_some() {
                            warn(receiver, "Self soould be immutable");
                        }
                    }
                    _ => warn(input, "First argument is not self"),
                }
            } else {
                match input {
                    syn::FnArg::Typed(pat_type) => {
                        let ty = pat_type.ty.as_ref();
                        if i - 1 < call_types.args_before.len() {
                            check_type_match(ty, &call_types.args_before[i - 1], false);
                        } else if i - 1 - call_types.args_before.len()
                            < call_types.args_calling.len()
                        {
                            check_type_match(
                                ty,
                                &call_types.args_calling[i - 1 - call_types.args_before.len()],
                                add_ref_on_before,
                            );
                        } else {
                            break;
                        }
                    }
                    _ => warn(input, "Argument is not typed"),
                }
            }
        }

        let current = (name.clone(), call_types.args_calling.len(), params);
        if !m.contains_key(&symbol_id) {
            m.insert(symbol_id, vec![current]);
        } else {
            m.get_mut(&symbol_id).unwrap().push(current);
        }
    } else {
        return Err(Error::new(
            name.span(),
            format!("Symbol not found: {} {}", prefix_pattern, symbol_name),
        ));
    }

    Ok(true)
}

pub struct ImplBodyGenerator {
    pub input: ItemImpl,
    pub input_stream: TokenStream,
    pub lt_token: Option<Token!(<)>,
    pub params: Punctuated<GenericParam, Token!(,)>,
    pub gt_token: Option<Token!(>)>,
    pub self_token: Box<Type>,
    pub where_clause: Option<WhereClause>,
    pub target_trait: TokenStream,
}

impl ImplBodyGenerator {
    pub fn new(item: proc_macro::TokenStream, target_trait: TokenStream) -> Result<Self> {
        let input_stream = item;
        let input = syn::parse::<ItemImpl>(input_stream.clone())?;
        let input_stream = input_stream.into();
        let self_token = input.self_ty.clone();
        let Generics {
            lt_token,
            params,
            gt_token,
            where_clause,
        } = input.generics.clone();
        Ok(Self {
            input,
            input_stream,
            lt_token,
            params,
            gt_token,
            self_token,
            where_clause,
            target_trait,
        })
    }

    pub fn generate(self, content: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream> {
        let Self {
            input_stream,
            lt_token,
            params,
            gt_token,
            self_token,
            where_clause,
            target_trait,
            ..
        } = self;
        Ok(quote! {
            #input_stream

            impl #lt_token #params #gt_token #target_trait for #self_token #where_clause {
                #content
            }
        })
    }
}

pub fn find_types<const N: usize>(
    ast: &GrammarFile<'_>,
    types: &GrammarTypes<'_>,
    symbol: &Symbol<'_>,
    rule_id: usize,
    ident: &Ident,
) -> Result<(Type, Vec<Type>)> {
    for node in ast.rules.iter() {
        if node.sym == symbol.name() {
            for rule in node.alternatives.iter() {
                if rule.id == rule_id {
                    let return_ty = types.get::<N>(SymbolType::NonTerminal, node.sym).clone();
                    let params_ty = rule
                        .elements
                        .iter()
                        .map(|element| types.get_ele::<N>(element).clone())
                        .collect();
                    return Ok((return_ty, params_ty));
                }
            }
            break;
        }
    }
    Err(Error::new(
        ident.span(),
        format!("Rule not found: {}", symbol.name()),
    ))
}

pub fn parse_args<'a, const N: usize>(
    args: proc_macro::TokenStream,
    arena: &'a GrammarArena<'a>,
    grammar_str_storage: &'a mut String,
    params: [&str; N],
) -> Result<(
    [TokenStream; N],
    GrammarFile<'a>,
    Grammar<'a>,
    GrammarTypes<'a>,
)> {
    let mut grammar_file: Option<LitStr> = None;
    let mut grammar_file_span = None;
    let mut values: Vec<Option<LitStr>> = (0..N).map(|_| None).collect();
    let meta_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("grammar_file") {
            let val = meta.value()?;
            grammar_file_span = Some(val.span());
            grammar_file = Some(val.parse()?);
            Ok(())
        } else {
            let mut parsed = false;
            for (i, p) in params.iter().enumerate() {
                if meta.path.is_ident(p) {
                    if values[i].is_some() {
                        return Err(meta.error("Duplicate property"));
                    }
                    values[i] = Some(meta.value()?.parse()?);
                    parsed = true;
                    break;
                }
            }
            if parsed {
                Ok(())
            } else {
                Err(meta.error("Unsupported proprety"))
            }
        }
    });
    meta_parser.parse(args)?;

    let grammar_file = if let Some(g) = grammar_file {
        g.value()
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            "No property: grammar_file",
        ));
    };

    let grammar_file_span = grammar_file_span.unwrap();

    let mut result = Vec::new();
    for (param, value) in params.iter().zip(values.iter()) {
        if let Some(v) = value {
            result.push(v.value().parse()?);
        } else {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("No property: {}", param),
            ));
        }
    }

    let grammar_file = std::fs::read_to_string(grammar_file);
    let grammar_file = match grammar_file {
        Ok(s) => s,
        Err(e) => {
            return Err(syn::Error::new(
                grammar_file_span,
                format!("Failed to read grammar file: {}", e),
            ))
        }
    };
    *grammar_str_storage = grammar_file;
    let grammar_ast = GrammarFileParser::new().parse(grammar_str_storage);
    let grammar_ast = match grammar_ast {
        Ok(g) => g,
        Err(e) => {
            return Err(syn::Error::new(
                grammar_file_span,
                format!("Failed to parse grammar file, syntax error: {}", e),
            ))
        }
    };
    let grammar_types = GrammarTypes::new(&grammar_ast);
    let grammar_types = match grammar_types {
        Ok(g) => g,
        Err(e) => {
            return Err(syn::Error::new(
                grammar_file_span,
                format!(
                    "Failed to parse grammar file, failed to create GrammarTypes: {}",
                    e
                ),
            ))
        }
    };
    let grammar = Grammar::from_grammar_ast(arena, &grammar_ast);
    let grammar = match grammar {
        Ok(g) => g,
        Err(e) => {
            return Err(syn::Error::new(
                grammar_file_span,
                format!(
                    "Failed to parse grammar file, failed to generate Grammar: {}",
                    e
                ),
            ))
        }
    };

    Ok((
        result.try_into().unwrap(),
        grammar_ast,
        grammar,
        grammar_types,
    ))
}

pub fn if_camel_case_to_snake_case(s: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        if c == '_' {
            if let Some(c) = chars.next() {
                if c.is_ascii_uppercase() {
                    result.push(c.to_ascii_lowercase());
                } else {
                    return None;
                }
            }
        } else {
            if c.is_ascii_uppercase() {
                result.push(c.to_ascii_lowercase());
            } else {
                return None;
            }
        }
    } else {
        return None;
    }
    for c in chars {
        if c.is_ascii_uppercase() {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    Some(result)
}
