pub enum CEnvAstItem<'input> {
    FuncDecl(&'input str, CEnvAstType, Vec<CEnvAstParam<'input>>),
    VarDecl(&'input str, CEnvAstType),
}

pub enum CEnvAstParam<'input> {
    WithName(&'input str, CEnvAstType),
    WithoutName(CEnvAstType),
    VaArgs,
}

pub enum CEnvAstTypeBase {
    Void,
    Int,
    Float,
}

pub enum CEnvAstTypeExtra {
    Const,
    Pointer,
    Array,
}

pub struct CEnvAstType {
    pub base: CEnvAstTypeBase,
    pub extra: Vec<CEnvAstTypeExtra>,
}
