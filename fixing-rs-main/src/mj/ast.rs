use super::mjenv::MJParseError;

pub struct MJAstCls<'input> {
    pub name: &'input str,
    pub inh: &'input str,
    pub fields: Vec<MJAstFieldDecl<'input>>,
    pub constructor: MJAstConDecl<'input>,
    pub methods: Vec<MJAstMethodDecl<'input>>,
}

pub struct MJAstFieldDecl<'input> {
    pub name: &'input str,
    pub ty: &'input str,
}

pub struct MJAstMethodDecl<'input> {
    pub name: &'input str,
    pub params: Vec<&'input str>,
    pub ret_ty: Option<&'input str>,
}

pub struct MJAstConDecl<'input> {
    pub name: &'input str,
    pub params: Vec<&'input str>,
}

lalrpop_mod!(mj_parser, "/grammars/mj_parser.rs");

pub fn parse_ast<'a>(input: &'a str) -> Result<Vec<MJAstCls<'a>>, MJParseError> {
    Ok(mj_parser::CompilationUnitParser::new().parse(input)?)
}
