use self::ast::{CEnvAstItem, CEnvAstParam, CEnvAstType, CEnvAstTypeBase, CEnvAstTypeExtra};
use super::{
    csymtab::CDeclaredFunc,
    types::{CFuncContentRef, CTypePool, CTypeRef},
};
use fixing_rs_base::{
    grammar::OwnedToken,
    utils::{RefArena, StringPool, StringRef},
};
use getset::{CopyGetters, Getters};
use lalrpop_util::lexer::Token;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{Debug, Display},
};

pub struct CEnvArena {
    strings: RefArena<String>,
}

impl CEnvArena {
    pub fn new() -> Self {
        Self {
            strings: RefArena::new(),
        }
    }
}

#[derive(Getters, CopyGetters)]
pub struct CEnv<'a> {
    #[get = "pub"]
    identifiers: Vec<StringRef<'a>>,
    #[get = "pub"]
    int_lits: HashMap<isize, StringRef<'a>>,
    #[get_copy = "pub"]
    types: &'a CTypePool<'a>,
    #[get_copy = "pub"]
    break_label: StringRef<'a>,
    #[get_copy = "pub"]
    continue_label: StringRef<'a>,
    #[get = "pub"]
    functions: HashMap<StringRef<'a>, CDeclaredFunc<'a>>,
    #[get = "pub"]
    default_functions: HashMap<StringRef<'a>, CDeclaredFunc<'a>>,
    #[get_copy = "pub"]
    current_func: CFuncContentRef<'a>,
    #[get = "pub"]
    str_pool: StringPool<'a>,
    #[get = "pub"]
    globals: HashMap<StringRef<'a>, CTypeRef<'a>>,
    #[get = "pub"]
    params: Vec<(StringRef<'a>, CTypeRef<'a>)>,
}

#[derive(Debug)]
pub enum CEnvBuildError {
    SyntaxError(lalrpop_util::ParseError<usize, OwnedToken, &'static str>),
    NoFunctions,
    FailedToDeriveConst,
    FailedToDerivePointer,
    FailedToDeriveArray,
    FailedToParseLiteralInt(String),
}

impl Display for CEnvBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for CEnvBuildError {}

impl<'input> From<lalrpop_util::ParseError<usize, Token<'input>, &'static str>> for CEnvBuildError {
    fn from(input: lalrpop_util::ParseError<usize, Token<'input>, &'static str>) -> Self {
        Self::SyntaxError(input.map_token(|x| x.into()))
    }
}

impl<'a> CEnv<'a> {
    fn gen_ty(ty: &CEnvAstType, types: &'a CTypePool<'a>) -> Result<CTypeRef<'a>, CEnvBuildError> {
        // TODO fix type base
        let mut result = match ty.base {
            CEnvAstTypeBase::Void => types.type_void(),
            CEnvAstTypeBase::Int => types.type_i32(),
            CEnvAstTypeBase::Float => types.type_f64(),
        };
        for extra in ty.extra.iter() {
            match extra {
                CEnvAstTypeExtra::Const => {
                    result = types
                        .derive_const(result)
                        .ok_or(CEnvBuildError::FailedToDeriveConst)?;
                }
                CEnvAstTypeExtra::Pointer => {
                    result = types
                        .derive_pointer(result)
                        .ok_or(CEnvBuildError::FailedToDerivePointer)?;
                }
                CEnvAstTypeExtra::Array => {
                    result = types
                        .derive_array(result)
                        .ok_or(CEnvBuildError::FailedToDeriveArray)?;
                }
            }
        }
        Ok(result)
    }

    fn gen_globals(
        vars: &HashMap<StringRef<'a>, CTypeRef<'a>>,
        functions: &HashMap<StringRef<'a>, CDeclaredFunc<'a>>,
        types: &'a CTypePool<'a>,
    ) -> HashMap<StringRef<'a>, CTypeRef<'a>> {
        let mut result = HashMap::new();
        for (name, ty) in vars.iter() {
            result.insert(*name, *ty);
        }
        for (name, func) in functions.iter() {
            let ty = types.derive_func(
                func.content.ret(),
                func.content.args().clone(),
                func.content.va_args(),
            );
            result.insert(*name, ty);
        }

        result
    }

    fn gen_params(
        current_func: &CFuncContentRef<'a>,
        current_func_args: &Vec<Option<StringRef<'a>>>,
    ) -> Vec<(StringRef<'a>, CTypeRef<'a>)> {
        std::iter::zip(current_func_args.iter(), current_func.args().iter())
            .flat_map(|(name, ty)| name.map(|name| (name, *ty)))
            .collect()
    }

    fn parse_literal_int(literal: &str) -> Result<isize, CEnvBuildError> {
        match literal.parse::<isize>() {
            Ok(x) => Ok(x),
            Err(_) => {
                let chars: Vec<_> = literal.chars().collect();
                if chars.len() >= 2 && chars[0] == '\'' && chars[chars.len() - 1] == '\'' {
                    let mut result = 0;
                    let mut last_slash = false;
                    for c in chars.iter().skip(1).take(chars.len() - 2) {
                        let c_processed = if last_slash {
                            match c {
                                'a' => 7,
                                'b' => 8,
                                'f' => 12,
                                'n' => 10,
                                'r' => 13,
                                't' => 9,
                                'v' => 11,
                                '\\' => 92,
                                '\'' => 39,
                                '"' => 34,
                                '?' => 63,
                                '0' => 0,
                                _ => 0,
                            }
                        } else {
                            if *c == '\\' {
                                last_slash = true;
                                continue;
                            } else {
                                *c as isize
                            }
                        };
                        result = result * 256 + c_processed as isize;
                    }
                    Ok(result)
                } else {
                    Err(CEnvBuildError::FailedToParseLiteralInt(literal.to_string()))
                }
            }
        }
    }

    pub fn is_true_id(&self, name: StringRef<'a>) -> bool {
        name != self.break_label && name != self.continue_label
    }

    pub fn build(
        arena: &'a CEnvArena,
        env: &str,
        types: &'a CTypePool<'a>,
        tokens: &Vec<fixing_rs_base::tokenizer::Token<'_, '_>>,
        max_new_id: usize,
    ) -> Result<Self, CEnvBuildError> {
        let env_ast = ast_parser::FileParser::new().parse(env)?;
        let mut identifiers = HashSet::new();
        let mut int_lits = HashMap::new();
        let mut str_pool = StringPool::new(&arena.strings);

        let break_label = str_pool.get_or_add("break");
        let continue_label = str_pool.get_or_add("continue");

        let mut functions = HashMap::new();
        let mut vars = HashMap::new();

        let mut current_function = None;
        let mut current_function_args = None;

        for token in tokens {
            if token.symbol.name() == "IDENTIFIER" {
                let name = str_pool.get_or_add(token.literal);
                identifiers.insert(name);
            } else if token.symbol.name() == "LITERAL_INT" {
                let literal = Self::parse_literal_int(token.literal)?;
                let name = format!("switch {}", token.literal);
                let name = str_pool.get_or_add(name.as_str());
                int_lits.insert(literal, name);
            }
        }

        for i in 0..max_new_id {
            let name = format!("__new_id_{}", i);
            let name = str_pool.get_or_add(name.as_str());
            identifiers.insert(name);
        }

        for (i, item) in env_ast.iter().enumerate() {
            match item {
                CEnvAstItem::FuncDecl(name, ty, args) => {
                    let name = str_pool.get_or_add(name);
                    identifiers.insert(name);
                    let ty = Self::gen_ty(&ty, &types)?;
                    let mut args_types = Vec::new();
                    let mut args_names = Vec::new();
                    let mut va_args = false;
                    for arg in args.iter() {
                        match arg {
                            CEnvAstParam::VaArgs => {
                                va_args = true;
                            }
                            CEnvAstParam::WithName(arg_name, arg_ty) => {
                                if i == env_ast.len() - 1 {
                                    let name = str_pool.get_or_add(arg_name);
                                    args_names.push(Some(name));
                                }
                                args_types.push(Self::gen_ty(arg_ty, &types)?);
                            }
                            CEnvAstParam::WithoutName(arg_ty) => {
                                if i == env_ast.len() - 1 {
                                    args_names.push(None);
                                }
                                args_types.push(Self::gen_ty(arg_ty, &types)?);
                            }
                        }
                    }
                    let func_type = types.derive_func(ty, args_types, va_args);
                    let func_content = types.get_func_content(func_type).unwrap();
                    let func = CDeclaredFunc::new(name, func_content);
                    functions.insert(name, func);
                    if i == env_ast.len() - 1 {
                        current_function = Some(func_content);
                        current_function_args = Some(args_names);
                    }
                }
                CEnvAstItem::VarDecl(name, ty) => {
                    let name = str_pool.get_or_add(name);
                    identifiers.insert(name);
                    let ty = Self::gen_ty(&ty, &types)?;
                    vars.insert(name, ty);
                }
            }
        }

        let current_func = current_function.ok_or(CEnvBuildError::NoFunctions)?;
        let current_func_args = current_function_args.unwrap();
        let identifiers: Vec<_> = identifiers.into_iter().collect();

        let mut default_functions = HashMap::new();
        let default_function = types.derive_func(types.type_i32(), vec![], true);
        let default_function = types.get_func_content(default_function).unwrap();
        for id in identifiers.iter() {
            let func = CDeclaredFunc::new(*id, default_function);
            default_functions.insert(*id, func);
        }

        let globals = Self::gen_globals(&vars, &functions, types);
        let params = Self::gen_params(&current_func, &current_func_args);

        Ok(Self {
            identifiers,
            int_lits,
            types,
            break_label,
            continue_label,
            functions,
            default_functions,
            str_pool,
            current_func,
            globals,
            params,
        })
    }
}

pub mod ast;
lalrpop_mod!(pub ast_parser, "/grammars/c_env.rs");
