use crate::{
    fixing_info::FixingInfo,
    grammar::{OwnedToken, SymbolType},
    parsing::{
        ast::{AlternativeNode, Element, GrammarFile},
        parser_grammar::GrammarFileParser,
    },
};
use lalrpop_util::lexer::Token;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::{
    borrow::Borrow,
    collections::HashMap,
    error::Error,
    fmt::Write,
    fmt::{Debug, Display},
    string::FromUtf8Error,
};
use subprocess::{Exec, ExitStatus, PopenError, Redirection};
use syn::{parse_str, Type};

#[derive(Clone, Debug)]
pub enum GenerateError {
    SyntaxError(lalrpop_util::ParseError<usize, OwnedToken, &'static str>),
    TypeParseError(syn::Error, String),
    SymbolNotFound(String),
}

impl<'input> From<lalrpop_util::ParseError<usize, Token<'input>, &'static str>> for GenerateError {
    fn from(input: lalrpop_util::ParseError<usize, Token<'input>, &'static str>) -> Self {
        Self::SyntaxError(input.map_token(|x| x.into()))
    }
}

impl Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for GenerateError {}

pub const DEFAULT_TYPE: &'static str = "PropEmpty";
pub const DEFAULT_ENTITY: &'static str = "Pointer<'s, EmptyEntity>";

pub const NON_TERMINAL_PREFIX: &str = "nt_";
pub const NON_TERMINAL_SYN_PREFIX: &str = "nts_";
pub const NON_TERMINAL_INH_PREFIX: &str = "nti_";
pub const SYMBOLIC_TERMINAL_PREFIX: &str = "st_";
pub const SYMBOLIC_TERMINAL_SYN_PREFIX: &str = "sts_";
pub const SYMBOLIC_TERMINAL_GEN_PREFIX: &str = "stg_";
pub const ROOT_INH_NAME: &str = "rooti";

fn expand_str_type_literal<'a>(literal: &'a str) -> &'a str {
    if literal.len() < 2 {
        panic!("str literal should be longer than 2");
    }
    literal[1..literal.len() - 1].trim()
}

#[derive(Debug)]
pub enum RustFmtError {
    StartFailed(PopenError),
    RustFmtFailed(ExitStatus),
    EncodingError(FromUtf8Error),
}

impl Display for RustFmtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <RustFmtError as Debug>::fmt(self, f)
    }
}

impl Error for RustFmtError {}

pub fn rustfmt(input: &str) -> Result<String, RustFmtError> {
    let result = Exec::cmd("rustfmt")
        .args(&["--emit", "stdout"])
        .stdin(input)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::None)
        .capture()
        .map_err(|x| RustFmtError::StartFailed(x))?;
    if let ExitStatus::Exited(0) = result.exit_status {
        String::from_utf8(result.stdout).map_err(|x| RustFmtError::EncodingError(x))
    } else {
        Err(RustFmtError::RustFmtFailed(result.exit_status))
    }
}

type SymbolTypes = [Option<Type>; 5];

#[derive(Getters)]
pub struct GrammarTypes<'a> {
    pub nonterminals: HashMap<&'a str, SymbolTypes>,
    pub terminals: HashMap<&'a str, SymbolTypes>,
    #[get = "pub"]
    pub default_type: Type,
    #[get = "pub"]
    pub default_entity: Type,
}

fn value_to_string<'a>(m: &HashMap<&'a str, SymbolTypes>) -> HashMap<&'a str, String> {
    m.iter()
        .map(|(k, v)| {
            (
                *k,
                format!(
                    "[{:?}, {:?}, {:?}]",
                    v[0].as_ref().map(|x| x.to_token_stream().to_string()),
                    v[1].as_ref().map(|x| x.to_token_stream().to_string()),
                    v[2].as_ref().map(|x| x.to_token_stream().to_string())
                ),
            )
        })
        .collect()
}

impl Debug for GrammarTypes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrammarTypes")
            .field("nonterminals", &value_to_string(&self.nonterminals))
            .field("terminals", &value_to_string(&self.terminals))
            .field(
                "default_type",
                &self.default_type.to_token_stream().to_string(),
            )
            .finish()
    }
}

fn parse_grammar_type(t: &str) -> Result<Option<Type>, GenerateError> {
    match t.is_empty() {
        true => Ok(None),
        false => syn::parse_str(t)
            .map(Some)
            .map_err(|x| GenerateError::TypeParseError(x, t.to_string())),
    }
}

fn parse_grammar_types(t: &Vec<&str>) -> Result<SymbolTypes, GenerateError> {
    let mut result = [None, None, None, None, None];
    for (i, t) in t.iter().enumerate().take(5) {
        let t = expand_str_type_literal(t);
        if i < 3 {
            result[i] = parse_grammar_type(t)?;
        } else {
            result[i] = if t.is_empty() {
                None
            } else {
                parse_grammar_type(format!("Pointer<'s, {}>", t).as_str())?
            };
        }
    }
    Ok(result)
}

impl<'a> GrammarTypes<'a> {
    pub fn new(ast: &GrammarFile<'a>) -> Result<Self, GenerateError> {
        let mut nonterminals = HashMap::new();
        for rule in ast.rules.iter() {
            nonterminals.insert(rule.sym, parse_grammar_types(&rule.types)?);
        }
        let mut terminals = HashMap::new();
        for anno in ast.annos.iter() {
            terminals.insert(anno.name, parse_grammar_types(&anno.types)?);
        }
        let default_type = syn::parse_str::<Type>(DEFAULT_TYPE).unwrap();
        let default_entity = syn::parse_str::<Type>(DEFAULT_ENTITY).unwrap();
        Ok(Self {
            nonterminals,
            terminals,
            default_type,
            default_entity,
        })
    }
    pub fn get<const N: usize>(&self, element_type: SymbolType, element_value: &str) -> &Type {
        let default_type = if N < 3 {
            &self.default_type
        } else {
            &self.default_entity
        };
        match element_type {
            SymbolType::NonTerminal => {
                let types = self.nonterminals.get(element_value).unwrap();
                types.get(N).unwrap().as_ref().unwrap_or(default_type)
            }
            SymbolType::LiteralTerminal => default_type,
            SymbolType::SymbolicTerminal => {
                let types = self.terminals.get(element_value);
                match types {
                    Some(types) => types.get(N).unwrap().as_ref().unwrap_or(default_type),
                    None => default_type,
                }
            }
        }
    }
    pub fn get_ele<const N: usize>(&self, element: &Element<'_>) -> &Type {
        self.get::<N>(element.element_type, element.element_value)
    }
    pub fn get_ele_g(&self, element: &Element<'_>) -> &Type {
        self.get_ele::<0>(element)
    }
    pub fn get_ele_inh(&self, element: &Element<'_>) -> &Type {
        self.get_ele::<1>(element)
    }
    pub fn get_ele_syn(&self, element: &Element<'_>) -> &Type {
        self.get_ele::<2>(element)
    }
    pub fn get_ele_inh_s(&self, element: &Element<'_>) -> &Type {
        self.get_ele::<3>(element)
    }
    pub fn get_ele_syn_s(&self, element: &Element<'_>) -> &Type {
        self.get_ele::<4>(element)
    }
    pub fn get_non_g(&self, ele: &str) -> &Type {
        self.get::<0>(SymbolType::NonTerminal, ele)
    }
    pub fn get_non_inh(&self, ele: &str) -> &Type {
        self.get::<1>(SymbolType::NonTerminal, ele)
    }
    pub fn get_non_syn(&self, ele: &str) -> &Type {
        self.get::<2>(SymbolType::NonTerminal, ele)
    }
    pub fn get_non_inh_s(&self, ele: &str) -> &Type {
        self.get::<3>(SymbolType::NonTerminal, ele)
    }
    pub fn get_non_syn_s(&self, ele: &str) -> &Type {
        self.get::<4>(SymbolType::NonTerminal, ele)
    }
}

fn write_comment(
    head: &str,
    sym: &str,
    alternative: &AlternativeNode<'_>,
    writer: &mut impl Write,
) {
    write!(writer, "// {} {} : {}", head, sym, alternative.id).unwrap();
    for element in alternative.elements.iter() {
        write!(writer, " {}", element.element_value).unwrap();
    }
}

fn write_fn<'a, PIter, RefElement>(
    head: &str,
    sym: &str,
    func_name: &str,
    generic_list: &TokenStream,
    alternative: &AlternativeNode<'_>,
    params: PIter,
    pre_tokens_fn: impl FnOnce(&mut TokenStream),
    return_type: &Type,
    add_ref: bool,
    writer: &mut impl Write,
) where
    PIter: Iterator<Item = RefElement>,
    RefElement: Borrow<Type>,
{
    write_comment(head, sym, alternative, writer);
    let func_name = Ident::new(func_name, Span::call_site());
    let mut param_tokens = quote! {};
    pre_tokens_fn(&mut param_tokens);
    let and_token = if add_ref {
        quote! { & }
    } else {
        quote! {}
    };
    for (i, param) in params.enumerate() {
        let param_name = Ident::new(&format!("_s{}", i + 1), Span::call_site());
        let param_type = param.borrow();
        param_tokens.extend(quote! {
            #param_name: #and_token #param_type,
        });
    }
    let func_tokens = quote! {
        fn #func_name #generic_list (&self, #param_tokens) -> #return_type {
            todo!()
        }
    };
    write!(writer, "\n{}\n\n", func_tokens.to_string()).unwrap();
}

pub fn gen_g_src(info: &FixingInfo) -> Result<String, GenerateError> {
    let parser = GrammarFileParser::new();
    let ast = parser.parse(info.grammar)?;
    let types = GrammarTypes::new(&ast)?;
    let generic_list = quote! {};

    let mut result = String::new();
    for rule in ast.rules.iter() {
        let return_type = types.get_non_g(rule.sym);
        for alternative in rule.alternatives.iter() {
            let func_name = format!("{}{}_{}", NON_TERMINAL_PREFIX, rule.sym, alternative.id);
            write_fn(
                "nt",
                rule.sym,
                &func_name,
                &generic_list,
                alternative,
                alternative.elements.iter().map(|x| types.get_ele_g(x)),
                |_| {},
                &return_type,
                true,
                &mut result,
            );
        }
    }

    let result = match rustfmt(&result) {
        Ok(x) => x,
        Err(_) => result,
    };

    Ok(result)
}

fn s_writer<'a>(inh_type: &'a Type, info: &FixingInfo) -> impl FnOnce(&mut TokenStream) + 'a {
    let g_type = info.prop_g;
    let g_type = parse_str::<Type>(g_type).unwrap();
    move |tokens: &mut TokenStream| {
        tokens.extend(quote! {
            _g: &PropArray< #g_type >,
            _inh: & #inh_type,
        });
    }
}

pub fn gen_s_src(info: &FixingInfo) -> Result<String, GenerateError> {
    let parser = GrammarFileParser::new();
    let ast = parser.parse(info.grammar)?;
    let types = GrammarTypes::new(&ast)?;
    let generic_list = quote! {};

    let mut result = String::new();
    for rule in ast.rules.iter() {
        for alternative in rule.alternatives.iter() {
            let inh_type = types.get_non_inh(rule.sym);
            let syn_type = types.get_non_syn(rule.sym);
            for (loc, element) in alternative.elements.iter().enumerate() {
                if element.element_type == SymbolType::LiteralTerminal {
                    continue;
                }
                let return_type = types.get_ele_inh(element);
                let func_name = format!(
                    "{}{}_{}_{}",
                    NON_TERMINAL_INH_PREFIX, rule.sym, alternative.id, loc
                );
                write_fn(
                    format!("nti {}", loc).as_str(),
                    rule.sym,
                    &func_name,
                    &generic_list,
                    alternative,
                    alternative
                        .elements
                        .iter()
                        .take(loc)
                        .map(|x| types.get_ele_syn(x)),
                    s_writer(inh_type, info),
                    &return_type,
                    true,
                    &mut result,
                );
            }
            {
                let func_name =
                    format!("{}{}_{}", NON_TERMINAL_SYN_PREFIX, rule.sym, alternative.id);
                write_fn(
                    "nts",
                    rule.sym,
                    &func_name,
                    &generic_list,
                    alternative,
                    alternative.elements.iter().map(|x| types.get_ele_syn(x)),
                    s_writer(inh_type, info),
                    syn_type,
                    true,
                    &mut result,
                );
            }
        }
    }

    let result = match rustfmt(&result) {
        Ok(x) => x,
        Err(_) => result,
    };

    Ok(result)
}

fn ss_writer<'a>(inh_type: &'a Type, info: &FixingInfo) -> impl FnOnce(&mut TokenStream) + 'a {
    let g_type = info.prop_g;
    let si_type = info.container_i;
    let ss_type = info.container_s;
    let g_type = parse_str::<Type>(g_type).unwrap();
    let si_type = parse_str::<Type>(si_type).unwrap();
    let ss_type = parse_str::<Type>(ss_type).unwrap();
    move |tokens: &mut TokenStream| {
        tokens.extend(quote! {
            _world: &mut SymbolicWorld,
            _container_i: &'s #si_type,
            _container_s: &'s #ss_type,
            _g: &PropArray< #g_type >,
            _inh: #inh_type,
        });
    }
}

pub fn gen_s_symbolic_src(info: &FixingInfo) -> Result<String, GenerateError> {
    let parser = GrammarFileParser::new();
    let ast = parser.parse(info.grammar)?;
    let types = GrammarTypes::new(&ast)?;
    let generic_list = quote! {<'s>};

    let mut result = String::new();
    for rule in ast.rules.iter() {
        for alternative in rule.alternatives.iter() {
            let inh_type = types.get_non_inh_s(rule.sym);
            let syn_type = types.get_non_syn_s(rule.sym);
            for (loc, element) in alternative.elements.iter().enumerate() {
                if element.element_type == SymbolType::LiteralTerminal {
                    continue;
                }
                let return_type = types.get_ele_inh_s(element);
                let func_name = format!(
                    "{}{}_{}_{}",
                    NON_TERMINAL_INH_PREFIX, rule.sym, alternative.id, loc
                );
                write_fn(
                    format!("nti {}", loc).as_str(),
                    rule.sym,
                    &func_name,
                    &generic_list,
                    alternative,
                    alternative
                        .elements
                        .iter()
                        .take(loc)
                        .map(|x| types.get_ele_syn_s(x)),
                    ss_writer(inh_type, info),
                    &return_type,
                    false,
                    &mut result,
                );
            }
            {
                let func_name =
                    format!("{}{}_{}", NON_TERMINAL_SYN_PREFIX, rule.sym, alternative.id);
                write_fn(
                    "nts",
                    rule.sym,
                    &func_name,
                    &generic_list,
                    alternative,
                    alternative.elements.iter().map(|x| types.get_ele_syn_s(x)),
                    ss_writer(inh_type, info),
                    syn_type,
                    false,
                    &mut result,
                );
            }
        }
    }

    let result = match rustfmt(&result) {
        Ok(x) => x,
        Err(_) => result,
    };

    Ok(result)
}
