use super::{
    cenv::CEnv,
    csymtab::{CIdSelected, CIdSelector, CSymTab, CSymTabExt, VarInfo},
    syntactic::CProp,
    types::{CExprInfo, CFuncContentRef, CType, CTypeComposed, CTypeRef, CTypeToken},
};
use fixing_rs_base::{
    props::{PropArray, PropEmpty},
    union_prop,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Prop, ValueEnum)]
pub enum CExprSuffixType {
    Slice,
    SelfInc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Prop, ValueEnum)]
pub enum CExprPrefixType {
    Neg,
    Not,
    BitNot,
    DeRef,
    Ref,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Prop, ValueEnum)]
pub enum CBinOpType {
    Mult,
    Mod,
    Plus,
    Subtract,
    BitMove,
    Compare,
    Equals,
    BitOp,
    BoolOp,
    Assign,
    PlusAssign,
    MulAssign,
    ModAssign,
    MoveAssign,
    BitOpAssign,
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub enum CArgs<'a> {
    VaArgs(CSymTab<'a>),
    Args(CSymTab<'a>, CFuncContentRef<'a>, usize),
}

impl<'a> CArgs<'a> {
    pub fn new(sym_tab: CSymTab<'a>, func: CFuncContentRef<'a>) -> Self {
        CArgs::Args(sym_tab, func, 0)
    }
    pub fn can_zero(&self) -> bool {
        match self {
            CArgs::VaArgs(_) => true,
            CArgs::Args(_, func, _) => func.args().len() == 0,
        }
    }
    pub fn to_zero_more(&self) -> Option<Self> {
        match self {
            CArgs::VaArgs(_) => Some(self.clone()),
            CArgs::Args(inh, func, _) => {
                if func.args().len() > 0 {
                    Some(CArgs::Args(inh.clone(), *func, 0))
                } else if func.va_args() {
                    Some(CArgs::VaArgs(inh.clone()))
                } else {
                    None
                }
            }
        }
    }
    pub fn can_end_now(&self) -> bool {
        match self {
            CArgs::VaArgs(_) => true,
            CArgs::Args(_, func, i) => func.args().len() == *i + 1,
        }
    }
    pub fn can_produce_more(&self) -> bool {
        match self {
            CArgs::VaArgs(_) => true,
            CArgs::Args(_, func, i) => func.args().len() > *i + 1 || func.va_args(),
        }
    }
    pub fn next_param(&self) -> Self {
        match self {
            CArgs::VaArgs(sym_tab) => CArgs::VaArgs(sym_tab.clone()),
            CArgs::Args(inh, func, i) => {
                if func.args().len() > *i + 1 {
                    CArgs::Args(inh.clone(), *func, i + 1)
                } else {
                    CArgs::VaArgs(inh.clone())
                }
            }
        }
    }
    pub fn sym_tab(&self) -> CSymTab<'a> {
        match self {
            CArgs::VaArgs(sym_tab) => sym_tab.clone(),
            CArgs::Args(sym_tab, _, _) => sym_tab.clone(),
        }
    }
    pub fn can_take(&self, ty: CTypeRef<'a>) -> bool {
        if ty.is_void() {
            return false;
        }
        match self {
            CArgs::VaArgs(_) => true,
            CArgs::Args(_, func, i) => match func.args().get(*i) {
                None => true,
                Some(to_ty) => CType::can_cast_to(ty, *to_ty),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub struct CDeclInh<'a> {
    pub symtab: CSymTab<'a>,
    pub decl_type: CTypeRef<'a>,
    pub has_init: bool,
}

impl<'a> CDeclInh<'a> {
    pub fn new(symtab: CSymTab<'a>, decl_type: CTypeRef<'a>, has_init: bool) -> Self {
        Self {
            symtab,
            decl_type,
            has_init,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub struct CDeclIdSyn<'a> {
    pub symtab: CSymTab<'a>,
    pub decl_type: CTypeRef<'a>,
}

impl<'a> CDeclIdSyn<'a> {
    pub fn new(symtab: CSymTab<'a>, decl_type: CTypeRef<'a>) -> Self {
        Self { symtab, decl_type }
    }
}

union_prop!(
    CInhProp<'a>,
    Empty,
    {
        Empty(PropEmpty),
        SymTab(CSymTab<'a>),
        Args(CArgs<'a>),
        DeclInh(CDeclInh<'a>),
        IdSelector(CIdSelector<'a>)
    }
);

union_prop!(
    CSynProp<'a>,
    Empty,
    {
        Empty(PropEmpty),
        SymTab(CSymTab<'a>),
        Suffix(CExprSuffixType),
        Prefix(CExprPrefixType),
        BinOp(CBinOpType),
        Ty(CTypeRef<'a>),
        TyBasic(CTypeToken),
        TyComposed(CTypeComposed),
        IdSelected(CIdSelected<'a>),
        Expr(CExprInfo<'a>),
        DeclId(CDeclIdSyn<'a>)
    }
);

pub struct CSProcessor<'a> {
    env: &'a CEnv<'a>,
    void_info: VarInfo<'a>,
}

impl<'a> CSProcessor<'a> {
    pub fn new(env: &'a CEnv<'a>) -> Self {
        let void_info = VarInfo::new(env.types().type_void());
        Self { env, void_info }
    }
}

#[impl_semantic_processor(
    g_prop = "CProp",
    si_prop = "CInhProp<'a>",
    ss_prop = "CSynProp<'a>",
    grammar_file = "fixing-rs-main/src/c/c_grammar"
)]
#[allow(non_snake_case)]
impl<'a> CSProcessor<'a> {
    // nts exprsuffix : 0 ++
    fn nts_exprsuffix_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprSuffixType {
        CExprSuffixType::SelfInc
    }

    // nts exprsuffix : 1 --
    fn nts_exprsuffix_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprSuffixType {
        CExprSuffixType::SelfInc
    }

    // nts exprsuffix : 2 [ expr ]
    fn nts_exprsuffix_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s3: &PropEmpty,
    ) -> Option<CExprSuffixType> {
        if expr.ty.is_integer() {
            Some(CExprSuffixType::Slice)
        } else {
            None
        }
    }

    // nts exprprefixCast : 2 +
    fn nts_exprprefixCast_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprPrefixType {
        CExprPrefixType::Neg
    }

    // nts exprprefixCast : 3 -
    fn nts_exprprefixCast_3(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprPrefixType {
        CExprPrefixType::Neg
    }

    // nts exprprefixCast : 4 !
    fn nts_exprprefixCast_4(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprPrefixType {
        CExprPrefixType::Not
    }

    // nts exprprefixCast : 5 ~
    fn nts_exprprefixCast_5(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprPrefixType {
        CExprPrefixType::BitNot
    }

    // nts exprprefixCast : 7 *
    fn nts_exprprefixCast_7(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprPrefixType {
        CExprPrefixType::DeRef
    }

    // nts exprprefixCast : 8 &
    fn nts_exprprefixCast_8(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprPrefixType {
        CExprPrefixType::Ref
    }

    // nts exprbinop : 0 *
    fn nts_exprbinop_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Mult
    }

    // nts exprbinop : 1 /
    fn nts_exprbinop_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Mult
    }

    // nts exprbinop : 2 %
    fn nts_exprbinop_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Mod
    }

    // nts exprbinop : 3 +
    fn nts_exprbinop_3(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Plus
    }

    // nts exprbinop : 4 -
    fn nts_exprbinop_4(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Subtract
    }

    // nts exprbinop : 5 <<
    fn nts_exprbinop_5(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitMove
    }

    // nts exprbinop : 6 >>
    fn nts_exprbinop_6(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitMove
    }

    // nts exprbinop : 7 <
    fn nts_exprbinop_7(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Compare
    }

    // nts exprbinop : 8 <=
    fn nts_exprbinop_8(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Compare
    }

    // nts exprbinop : 9 >
    fn nts_exprbinop_9(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Compare
    }

    // nts exprbinop : 10 >=
    fn nts_exprbinop_10(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Compare
    }

    // nts exprbinop : 11 ==
    fn nts_exprbinop_11(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Equals
    }

    // nts exprbinop : 12 !=
    fn nts_exprbinop_12(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Equals
    }

    // nts exprbinop : 13 &
    fn nts_exprbinop_13(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitOp
    }

    // nts exprbinop : 14 ^
    fn nts_exprbinop_14(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitOp
    }

    // nts exprbinop : 15 |
    fn nts_exprbinop_15(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitOp
    }

    // nts exprbinop : 16 &&
    fn nts_exprbinop_16(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BoolOp
    }

    // nts exprbinop : 17 ||
    fn nts_exprbinop_17(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BoolOp
    }

    // nts exprbinop : 18 =
    fn nts_exprbinop_18(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Assign
    }

    // nts exprbinop : 19 +=
    fn nts_exprbinop_19(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::PlusAssign
    }

    // nts exprbinop : 20 -=
    fn nts_exprbinop_20(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::PlusAssign
    }

    // nts exprbinop : 21 *=
    fn nts_exprbinop_21(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::MulAssign
    }

    // nts exprbinop : 22 /=
    fn nts_exprbinop_22(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::MulAssign
    }

    // nts exprbinop : 23 %=
    fn nts_exprbinop_23(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::ModAssign
    }

    // nts exprbinop : 24 <<=
    fn nts_exprbinop_24(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::MoveAssign
    }

    // nts exprbinop : 25 >>=
    fn nts_exprbinop_25(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::MoveAssign
    }

    // nts exprbinop : 26 &=
    fn nts_exprbinop_26(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitOpAssign
    }

    // nts exprbinop : 27 ^=
    fn nts_exprbinop_27(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitOpAssign
    }

    // nts exprbinop : 28 |=
    fn nts_exprbinop_28(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::BitOpAssign
    }

    // nts exprbinop : 29 ,
    fn nts_exprbinop_29(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CBinOpType {
        CBinOpType::Comma
    }

    // nti 0 expr : 0 IDENTIFIER
    fn nti_expr_0_0(&self, _g: &PropArray<CProp>, inh: &CSymTab<'a>) -> CIdSelector<'a> {
        CIdSelector::Identifier(inh.clone())
    }

    // nts expr : 0 IDENTIFIER
    fn nts_expr_0(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        id: &CIdSelected<'a>,
    ) -> Option<CExprInfo<'a>> {
        match id {
            CIdSelected::Identifier(id) => match inh.get(*id) {
                Some(d) => Some(CExprInfo::new(d.ty(), true)),
                None => None,
            },
            _ => unreachable!(),
        }
    }

    // nts expr : 1 literal
    fn nts_expr_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        l: &CExprInfo<'a>,
    ) -> CExprInfo<'a> {
        l.clone()
    }

    // nts expr : 2 expr exprsuffix
    fn nts_expr_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        expr: &CExprInfo<'a>,
        op: &CExprSuffixType,
    ) -> Option<CExprInfo<'a>> {
        match op {
            CExprSuffixType::SelfInc => {
                if expr.is_lvalue && expr.ty.can_self_inc() {
                    Some(CExprInfo::new(expr.ty, false))
                } else {
                    None
                }
            }
            CExprSuffixType::Slice => match expr.ty.get_slice() {
                Some(ty) => Some(CExprInfo::new(ty, true)),
                None => None,
            },
        }
    }

    // nts expr : 3 exprprefix expr
    fn nts_expr_3(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        expr: &CExprInfo<'a>,
    ) -> Option<CExprInfo<'a>> {
        // ++ --
        if expr.is_lvalue && expr.ty.can_self_inc() {
            Some(CExprInfo::new(expr.ty, false))
        } else {
            None
        }
    }

    // nts expr : 4 expr exprbinop expr
    fn nts_expr_4(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        expr1: &CExprInfo<'a>,
        op: &CBinOpType,
        expr2: &CExprInfo<'a>,
    ) -> Option<CExprInfo<'a>> {
        match op {
            CBinOpType::Mult => {
                CType::mult_type(expr1.ty, expr2.ty).map(|ty| CExprInfo::new(ty, false))
            }
            CBinOpType::Mod => {
                CType::mod_type(expr1.ty, expr2.ty).map(|ty| CExprInfo::new(ty, false))
            }
            CBinOpType::Plus => CType::add_type(expr1.ty, expr2.ty, self.env.types())
                .map(|ty| CExprInfo::new(ty, false)),
            CBinOpType::Subtract => CType::subtract_type(expr1.ty, expr2.ty, self.env.types())
                .map(|ty| CExprInfo::new(ty, false)),
            CBinOpType::BitMove => {
                CType::shift_type(expr1.ty, expr2.ty).map(|ty| CExprInfo::new(ty, false))
            }
            CBinOpType::Compare => CType::compare_type(expr1.ty, expr2.ty, self.env.types())
                .map(|ty| CExprInfo::new(ty, false)),
            CBinOpType::Equals => CType::equals_type(expr1.ty, expr2.ty, self.env.types())
                .map(|ty| CExprInfo::new(ty, false)),
            CBinOpType::BitOp => {
                CType::bitop_type(expr1.ty, expr2.ty).map(|ty| CExprInfo::new(ty, false))
            }
            CBinOpType::BoolOp => CType::logicop_type(expr1.ty, expr2.ty, self.env.types())
                .map(|ty| CExprInfo::new(ty, false)),
            CBinOpType::Assign => {
                if expr1.is_lvalue {
                    CType::assign_type(expr1.ty, expr2.ty).map(|ty| CExprInfo::new(ty, false))
                } else {
                    None
                }
            }
            CBinOpType::PlusAssign => {
                if expr1.is_lvalue {
                    CType::plusassign_type(expr1.ty, expr2.ty).map(|ty| CExprInfo::new(ty, false))
                } else {
                    None
                }
            }
            CBinOpType::MulAssign => {
                if expr1.is_lvalue
                    && !expr1.ty.is_const()
                    && CType::mult_type(expr1.ty, expr2.ty).is_some()
                {
                    Some(CExprInfo::new(expr1.ty, false))
                } else {
                    None
                }
            }
            CBinOpType::ModAssign => {
                if expr1.is_lvalue
                    && !expr1.ty.is_const()
                    && CType::mod_type(expr1.ty, expr2.ty).is_some()
                {
                    Some(CExprInfo::new(expr1.ty, false))
                } else {
                    None
                }
            }
            CBinOpType::MoveAssign => {
                if expr1.is_lvalue
                    && !expr1.ty.is_const()
                    && CType::shift_type(expr1.ty, expr2.ty).is_some()
                {
                    Some(CExprInfo::new(expr1.ty, false))
                } else {
                    None
                }
            }
            CBinOpType::BitOpAssign => {
                if expr1.is_lvalue
                    && !expr1.ty.is_const()
                    && CType::bitop_type(expr1.ty, expr2.ty).is_some()
                {
                    Some(CExprInfo::new(expr1.ty, false))
                } else {
                    None
                }
            }
            CBinOpType::Comma => Some(CExprInfo::new(expr2.ty, false)),
        }
    }

    // nts expr : 5 expr ? expr : expr
    fn nts_expr_5(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        expr1: &CExprInfo<'a>,
        _s2: &PropEmpty,
        expr2: &CExprInfo<'a>,
        _s4: &PropEmpty,
        expr3: &CExprInfo<'a>,
    ) -> Option<CExprInfo<'a>> {
        if expr1.ty.is_scalar() {
            CType::condition_type(expr2.ty, expr3.ty).map(|ty| CExprInfo::new(ty, false))
        } else {
            None
        }
    }

    // nts expr : 6 ( expr )
    fn nts_expr_6(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s3: &PropEmpty,
    ) -> CExprInfo<'a> {
        expr.clone()
    }

    // nti 0 expr : 7 IDENTIFIER ( args )
    fn nti_expr_7_0(&self, _g: &PropArray<CProp>, inh: &CSymTab<'a>) -> CIdSelector<'a> {
        CIdSelector::FuncName(inh.clone())
    }

    // nti 2 expr : 7 IDENTIFIER ( args )
    fn nti_expr_7_2(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        id: &CIdSelected<'a>,
        _s2: &PropEmpty,
    ) -> CArgs<'a> {
        match id {
            CIdSelected::FuncName(func) => CArgs::new(inh.clone(), func.content),
            _ => unreachable!(),
        }
    }

    // nts expr : 7 IDENTIFIER ( args )
    fn nts_expr_7(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        id: &CIdSelected<'a>,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
    ) -> CExprInfo<'a> {
        match id {
            CIdSelected::FuncName(func) => CExprInfo::new(func.content.ret(), false),
            _ => unreachable!(),
        }
    }

    // nts expr : 8 ( typeExpr ) expr
    fn nts_expr_8(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        ty: &CTypeRef<'a>,
        _s3: &PropEmpty,
        expr: &CExprInfo<'a>,
    ) -> Option<CExprInfo<'a>> {
        if CType::can_cast_to(expr.ty, *ty) {
            Some(CExprInfo::new(*ty, false))
        } else {
            None
        }
    }

    // nts expr : 9 sizeof expr
    fn nts_expr_9(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &CExprInfo<'a>,
    ) -> CExprInfo<'a> {
        CExprInfo::new(self.env.types().type_u64(), false)
    }

    // nts expr : 10 sizeof ( typeExpr )
    fn nts_expr_10(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &CTypeRef<'a>,
        _s4: &PropEmpty,
    ) -> CExprInfo<'a> {
        CExprInfo::new(self.env.types().type_u64(), false)
    }

    // nts expr : 11 exprprefixCast expr
    fn nts_expr_11(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        op: &CExprPrefixType,
        expr: &CExprInfo<'a>,
    ) -> Option<CExprInfo<'a>> {
        match op {
            CExprPrefixType::Neg => {
                if expr.ty.is_arithmetic() {
                    Some(CExprInfo::new(expr.ty, false))
                } else {
                    None
                }
            }
            CExprPrefixType::Not => {
                if expr.ty.is_scalar() {
                    Some(CExprInfo::new(expr.ty, false))
                } else {
                    None
                }
            }
            CExprPrefixType::BitNot => {
                if expr.ty.is_integer() {
                    Some(CExprInfo::new(expr.ty, false))
                } else {
                    None
                }
            }
            CExprPrefixType::DeRef => expr
                .ty
                .remove_pointer_array()
                .map(|ty| CExprInfo::new(ty, true)),
            CExprPrefixType::Ref => {
                if expr.is_lvalue && !expr.ty.is_void() {
                    self.env
                        .types()
                        .derive_pointer(expr.ty)
                        .map(|ty| CExprInfo::new(ty, false))
                } else {
                    None
                }
            }
        }
    }

    // nts args : 0
    fn nts_args_0(&self, _g: &PropArray<CProp>, inh: &CArgs<'a>) -> Option<PropEmpty> {
        match inh.can_zero() {
            true => Some(PropEmpty),
            false => None,
        }
    }

    // nti 0 args : 1 argsOther
    fn nti_args_1_0(&self, _g: &PropArray<CProp>, inh: &CArgs<'a>) -> Option<CArgs<'a>> {
        inh.to_zero_more()
    }

    // nti 0 argsOther : 0 expr
    fn nti_argsOther_0_0(&self, _g: &PropArray<CProp>, inh: &CArgs<'a>) -> Option<CSymTab<'a>> {
        match inh.can_end_now() {
            true => Some(inh.sym_tab()),
            false => None,
        }
    }

    // nts argsOther : 0 expr
    fn nts_argsOther_0(
        &self,
        _g: &PropArray<CProp>,
        inh: &CArgs<'a>,
        expr: &CExprInfo<'a>,
    ) -> Option<PropEmpty> {
        if inh.can_take(expr.ty) {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nti 0 argsOther : 1 expr , argsOther
    fn nti_argsOther_1_0(&self, _g: &PropArray<CProp>, inh: &CArgs<'a>) -> Option<CSymTab<'a>> {
        match inh.can_produce_more() {
            true => Some(inh.sym_tab()),
            false => None,
        }
    }

    // nti 2 argsOther : 1 expr , argsOther
    fn nti_argsOther_1_2(
        &self,
        _g: &PropArray<CProp>,
        inh: &CArgs<'a>,
        expr: &CExprInfo<'a>,
        _s2: &PropEmpty,
    ) -> Option<CArgs<'a>> {
        if inh.can_take(expr.ty) {
            Some(inh.next_param())
        } else {
            None
        }
    }

    // nts literal : 0 LITERAL_INT
    fn nts_literal_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprInfo<'a> {
        // TODO fix int literal type
        CExprInfo::new(self.env.types().type_i32(), false)
    }

    // nts literal : 1 LITERAL_FLOAT
    fn nts_literal_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprInfo<'a> {
        // TODO fix float literal type
        CExprInfo::new(self.env.types().type_f64(), false)
    }

    // nts literal : 2 LITERAL_STRING
    fn nts_literal_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CExprInfo<'a> {
        CExprInfo::new(self.env.types().type_str(), false)
    }

    // nts stmtOrDecl : 0 decl
    fn nts_stmtOrDecl_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        decl: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        decl.clone()
    }

    // nts stmtOrDecl : 1 stmt
    fn nts_stmtOrDecl_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.clone()
    }

    // nti 1 stmt : 0 { stmtList }
    fn nti_stmt_0_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.new_scope()
    }

    // nti 4 stmt : 2 if ( expr ) stmt
    fn nti_stmt_2_4(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s4: &PropEmpty,
    ) -> Option<CSymTab<'a>> {
        if expr.ty.is_scalar() {
            Some(inh.clone())
        } else {
            None
        }
    }

    // nti 4 stmt : 3 if ( expr ) stmt else stmt
    fn nti_stmt_3_4(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s4: &PropEmpty,
    ) -> Option<CSymTab<'a>> {
        if expr.ty.is_scalar() {
            Some(inh.clone())
        } else {
            None
        }
    }

    // nts stmt : 4 break ;
    fn nts_stmt_4(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> Option<PropEmpty> {
        match inh.get(self.env.break_label()) {
            Some(_) => Some(PropEmpty),
            None => None,
        }
    }

    // nts stmt : 5 continue ;
    fn nts_stmt_5(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> Option<PropEmpty> {
        match inh.get(self.env.continue_label()) {
            Some(_) => Some(PropEmpty),
            None => None,
        }
    }

    // nti 1 stmt : 6 do stmt while ( expr ) ;
    fn nti_stmt_6_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.extend_multiple(
            [self.env.break_label(), self.env.continue_label()].into_iter(),
            self.void_info.clone(),
        )
    }

    // nts stmt : 6 do stmt while ( expr ) ;
    fn nts_stmt_6(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s6: &PropEmpty,
        _s7: &PropEmpty,
    ) -> Option<PropEmpty> {
        if expr.ty.is_scalar() {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nti 4 stmt : 7 while ( expr ) stmt
    fn nti_stmt_7_4(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s4: &PropEmpty,
    ) -> Option<CSymTab<'a>> {
        if expr.ty.is_scalar() {
            Some(inh.extend_multiple(
                [self.env.break_label(), self.env.continue_label()].into_iter(),
                self.void_info.clone(),
            ))
        } else {
            None
        }
    }

    // nti 2 stmt : 8 for ( for1 for2 ; for3 ) stmt
    fn nti_stmt_8_2(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.new_scope()
    }

    // nti 3 stmt : 8 for ( for1 for2 ; for3 ) stmt
    fn nti_stmt_8_3(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        for1: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        for1.clone()
    }

    // nti 5 stmt : 8 for ( for1 for2 ; for3 ) stmt
    fn nti_stmt_8_5(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        for1: &CSymTab<'a>,
        _s4: &PropEmpty,
        _s5: &PropEmpty,
    ) -> CSymTab<'a> {
        for1.clone()
    }

    // nti 7 stmt : 8 for ( for1 for2 ; for3 ) stmt
    fn nti_stmt_8_7(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        for1: &CSymTab<'a>,
        _s4: &PropEmpty,
        _s5: &PropEmpty,
        _s6: &PropEmpty,
        _s7: &PropEmpty,
    ) -> CSymTab<'a> {
        for1.extend_multiple(
            [self.env.break_label(), self.env.continue_label()].into_iter(),
            self.void_info.clone(),
        )
    }

    // nti 5 stmt : 9 switch ( expr ) { switchBlock }
    fn nti_stmt_9_5(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s4: &PropEmpty,
        _s5: &PropEmpty,
    ) -> Option<CSymTab<'a>> {
        if expr.ty.is_integer() {
            Some(
                inh.new_scope()
                    .extend(self.env.break_label(), self.void_info.clone()),
            )
        } else {
            None
        }
    }

    // nts stmt : 12 return ;
    fn nts_stmt_12(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> Option<PropEmpty> {
        if self.env.current_func().ret() == self.env.types().type_void() {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nti 1 stmt : 13 return expr ;
    fn nti_stmt_13_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> Option<CSymTab<'a>> {
        if self.env.current_func().ret() != self.env.types().type_void() {
            Some(inh.clone())
        } else {
            None
        }
    }

    // nts stmt : 13 return expr ;
    fn nts_stmt_13(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        expr: &CExprInfo<'a>,
        _s3: &PropEmpty,
    ) -> Option<PropEmpty> {
        if CType::can_cast_to(expr.ty, self.env.current_func().ret()) {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nti 1 switchBlock : 1 switchBlockContent switchBlock
    fn nti_switchBlock_1_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        block: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        block.clone()
    }

    // nts switchBlockContent : 0 case INT_LITERAL :
    fn nts_switchBlockContent_0(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.clone()
    }

    // nts switchBlockContent : 1 default :
    fn nts_switchBlockContent_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.clone()
    }

    // nts switchBlockContent : 2 decl
    fn nts_switchBlockContent_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        decl: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        decl.clone()
    }

    // nts switchBlockContent : 3 stmt
    fn nts_switchBlockContent_3(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.clone()
    }

    // nti 1 stmtList : 1 stmtOrDecl stmtList
    fn nti_stmtList_1_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        stmt: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        stmt.clone()
    }

    // nti 1 decl : 0 type declList ;
    fn nti_decl_0_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        ty: &CTypeComposed,
    ) -> CDeclInh<'a> {
        let ty = self.env.types().construct_from_composed(ty);
        CDeclInh::new(inh.clone(), ty, false)
    }

    // nts decl : 0 type declList ;
    fn nts_decl_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &CTypeComposed,
        decls: &CSymTab<'a>,
        _s3: &PropEmpty,
    ) -> CSymTab<'a> {
        decls.clone()
    }

    // nts declList : 0 declOp
    fn nts_declList_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        op: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        op.clone()
    }

    // nti 2 declList : 1 declOp , declList
    fn nti_declList_1_2(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        op: &CSymTab<'a>,
        _s2: &PropEmpty,
    ) -> CDeclInh<'a> {
        CDeclInh::new(op.clone(), inh.decl_type, false)
    }

    // nts declList : 1 declOp , declList
    fn nts_declList_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        _s1: &CSymTab<'a>,
        _s2: &PropEmpty,
        decl_list: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        decl_list.clone()
    }

    // nts declOp : 0 declId
    fn nts_declOp_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        decl: &CDeclIdSyn<'a>,
    ) -> CSymTab<'a> {
        decl.symtab.clone()
    }

    // nti 0 declOp : 1 declId = initializer
    fn nti_declOp_1_0(&self, _g: &PropArray<CProp>, inh: &CDeclInh<'a>) -> CDeclInh<'a> {
        CDeclInh::new(inh.symtab.clone(), inh.decl_type, true)
    }

    // nti 2 declOp : 1 declId = initializer
    fn nti_declOp_1_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        decl: &CDeclIdSyn<'a>,
        _s2: &PropEmpty,
    ) -> CDeclInh<'a> {
        CDeclInh::new(decl.symtab.clone(), decl.decl_type, false)
    }

    // nts declOp : 1 declId = initializer
    fn nts_declOp_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        op: &CDeclIdSyn<'a>,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
    ) -> CSymTab<'a> {
        op.symtab.clone()
    }

    // nti 0 declId : 0 IDENTIFIER
    fn nti_declId_0_0(&self, _g: &PropArray<CProp>, inh: &CDeclInh<'a>) -> Option<CIdSelector<'a>> {
        if inh.decl_type.is_complete() || (inh.has_init && inh.decl_type.is_array()) {
            Some(CIdSelector::NewIdentifier(inh.symtab.clone()))
        } else {
            None
        }
    }

    // nts declId : 0 IDENTIFIER
    fn nts_declId_0(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        id: &CIdSelected<'a>,
    ) -> CDeclIdSyn<'a> {
        match id {
            CIdSelected::NewIdentifier(id) => {
                let symtab = inh.symtab.extend(*id, VarInfo::new(inh.decl_type));
                CDeclIdSyn::new(symtab, inh.decl_type)
            }
            _ => unreachable!(),
        }
    }

    // nti 1 declId : 1 * declId
    fn nti_declId_1_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        _s1: &PropEmpty,
    ) -> Option<CDeclInh<'a>> {
        match self.env.types().derive_pointer(inh.decl_type) {
            Some(ty) => Some(CDeclInh::new(inh.symtab.clone(), ty, inh.has_init)),
            None => None,
        }
    }

    // nts declId : 1 * declId
    fn nts_declId_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        _s1: &PropEmpty,
        declId: &CDeclIdSyn<'a>,
    ) -> CDeclIdSyn<'a> {
        declId.clone()
    }

    // nti 2 declId : 3 * const declId
    fn nti_declId_3_2(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> Option<CDeclInh<'a>> {
        match self.env.types().derive_pointer(inh.decl_type) {
            Some(ty) => match self.env.types().derive_const(ty) {
                Some(ty) => Some(CDeclInh::new(inh.symtab.clone(), ty, inh.has_init)),
                None => None,
            },
            None => None,
        }
    }

    // nts declId : 3 * const declId
    fn nts_declId_3(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        decl_id: &CDeclIdSyn<'a>,
    ) -> CDeclIdSyn<'a> {
        decl_id.clone()
    }

    // nti 0 declId : 4 declId [ ]
    fn nti_declId_4_0(&self, _g: &PropArray<CProp>, inh: &CDeclInh<'a>) -> Option<CDeclInh<'a>> {
        match self.env.types().derive_imcomplete_array(inh.decl_type) {
            Some(ty) => Some(CDeclInh::new(inh.symtab.clone(), ty, inh.has_init)),
            None => None,
        }
    }

    // nts declId : 4 declId [ ]
    fn nts_declId_4(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        decl_id: &CDeclIdSyn<'a>,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
    ) -> CDeclIdSyn<'a> {
        decl_id.clone()
    }

    // nts declId : 5 ( declId )
    fn nts_declId_5(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        _s1: &PropEmpty,
        decl_id: &CDeclIdSyn<'a>,
        _s3: &PropEmpty,
    ) -> CDeclIdSyn<'a> {
        decl_id.clone()
    }

    // nti 0 declId : 2 declId [ LITERAL_INT ]
    fn nti_declId_2_0(&self, _g: &PropArray<CProp>, inh: &CDeclInh<'a>) -> Option<CDeclInh<'a>> {
        match self.env.types().derive_array(inh.decl_type) {
            Some(ty) => Some(CDeclInh::new(inh.symtab.clone(), ty, inh.has_init)),
            None => None,
        }
    }

    // nti 2 declId : 2 declId [ LITERAL_INT ]
    fn nti_declId_2_2(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        _s1: &CDeclIdSyn<'a>,
        _s2: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.symtab.clone()
    }

    // nts declId : 2 declId [ LITERAL_INT ]
    fn nts_declId_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CDeclInh<'a>,
        decl_id: &CDeclIdSyn<'a>,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
    ) -> CDeclIdSyn<'a> {
        decl_id.clone()
    }

    // nti 0 initializer : 0 expr
    fn nti_initializer_0_0(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
    ) -> Option<CSymTab<'a>> {
        if inh.decl_type.is_array() {
            None
        } else {
            Some(inh.symtab.clone())
        }
    }

    // nts initializer : 0 expr
    fn nts_initializer_0(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        expr: &CExprInfo<'a>,
    ) -> Option<PropEmpty> {
        if CType::can_cast_to(expr.ty, inh.decl_type) {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nts initializer : 1 { }
    fn nts_initializer_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> Option<PropEmpty> {
        if inh.decl_type.is_array() {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nti 1 initializer : 2 { initializerList }
    fn nti_initializer_2_1(
        &self,
        _g: &PropArray<CProp>,
        inh: &CDeclInh<'a>,
        _s1: &PropEmpty,
    ) -> CDeclInh<'a> {
        if inh.decl_type.is_array() {
            CDeclInh::new(
                inh.symtab.clone(),
                inh.decl_type.remove_pointer_array().unwrap(),
                false,
            )
        } else {
            inh.clone()
        }
    }

    // nts for1 : 0 ;
    fn nts_for1_0(&self, _g: &PropArray<CProp>, inh: &CSymTab<'a>, _s1: &PropEmpty) -> CSymTab<'a> {
        inh.clone()
    }

    // nti 0 for1 : 1 decl
    fn nti_for1_1_0(&self, _g: &PropArray<CProp>, inh: &CSymTab<'a>) -> CSymTab<'a> {
        inh.new_scope()
    }

    // nts for1 : 1 decl
    fn nts_for1_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        for1: &CSymTab<'a>,
    ) -> CSymTab<'a> {
        for1.clone()
    }

    // nts for1 : 2 expr ;
    fn nts_for1_2(
        &self,
        _g: &PropArray<CProp>,
        inh: &CSymTab<'a>,
        _s1: &CExprInfo<'a>,
        _s2: &PropEmpty,
    ) -> CSymTab<'a> {
        inh.clone()
    }

    // nts for2 : 1 expr
    fn nts_for2_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        expr: &CExprInfo<'a>,
    ) -> Option<PropEmpty> {
        if expr.ty.is_scalar() {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nts typeBasic : 0 signed
    fn nts_typeBasic_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Signed
    }

    // nts typeBasic : 1 unsigned
    fn nts_typeBasic_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Unsigned
    }

    // nts typeBasic : 2 char
    fn nts_typeBasic_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Char
    }

    // nts typeBasic : 3 short
    fn nts_typeBasic_3(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Short
    }

    // nts typeBasic : 4 int
    fn nts_typeBasic_4(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Int
    }

    // nts typeBasic : 5 long
    fn nts_typeBasic_5(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Long
    }

    // nts typeBasic : 6 float
    fn nts_typeBasic_6(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Float
    }

    // nts typeBasic : 7 double
    fn nts_typeBasic_7(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Double
    }

    // nts typeBasic : 8 const
    fn nts_typeBasic_8(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Const
    }

    // nts typeBasic : 9 void
    fn nts_typeBasic_9(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _s1: &PropEmpty,
    ) -> CTypeToken {
        CTypeToken::Void
    }

    // nts type : 0 typeBasic
    fn nts_type_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        token: &CTypeToken,
    ) -> CTypeComposed {
        CTypeComposed::from_token(*token)
    }

    // nts type : 1 typeBasic type
    fn nts_type_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        token: &CTypeToken,
        composed: &CTypeComposed,
    ) -> Option<CTypeComposed> {
        composed.append_token(*token)
    }

    // nts typeExpr : 0 type
    fn nts_typeExpr_0(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        composed: &CTypeComposed,
    ) -> CTypeRef<'a> {
        self.env.types().construct_from_composed(composed)
    }

    // nts typeExpr : 1 typeExpr *
    fn nts_typeExpr_1(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        ty: &CTypeRef<'a>,
        _s2: &PropEmpty,
    ) -> Option<CTypeRef<'a>> {
        self.env.types().derive_pointer(*ty)
    }

    // nts typeExpr : 2 typeExpr * const
    fn nts_typeExpr_2(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        ty: &CTypeRef<'a>,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
    ) -> Option<CTypeRef<'a>> {
        match self.env.types().derive_pointer(*ty) {
            Some(ty) => self.env.types().derive_const(ty),
            None => None,
        }
    }

    // nti 0 functionBody : 0 stmtList
    fn nti_functionBody_0_0(&self, _g: &PropArray<CProp>, _inh: &PropEmpty) -> CSymTab<'a> {
        CSymTab::from_map(
            self.env
                .globals()
                .iter()
                .map(|(k, v)| (*k, VarInfo::new(*v).inner_scope()))
                .chain(
                    self.env
                        .params()
                        .iter()
                        .map(|(k, v)| (*k, VarInfo::new(*v))),
                )
                .collect(),
        )
    }

    fn sts_IDENTIFIER(
        &self,
        _g: &PropArray<CProp>,
        inh: &CIdSelector<'a>,
        literal: Option<&str>,
    ) -> Vec<CIdSelected<'a>> {
        match inh {
            CIdSelector::FuncName(sym_tab) => match literal {
                Some(literal) => {
                    let literal = self.env.str_pool().get(literal).unwrap();
                    match self.env.functions().get(&literal) {
                        Some(func) => {
                            let func_valid = match sym_tab.get(literal) {
                                Some(d) => {
                                    if let Some(_) = self.env.types().get_func_content(d.ty) {
                                        true
                                    } else {
                                        false
                                    }
                                }
                                None => true,
                            };
                            if func_valid {
                                vec![CIdSelected::FuncName(func.clone())]
                            } else {
                                vec![]
                            }
                        }
                        None => {
                            if sym_tab.get(literal).is_none() {
                                match self.env.default_functions().get(&literal) {
                                    Some(func) => vec![CIdSelected::FuncName(func.clone())],
                                    None => vec![],
                                }
                            } else {
                                vec![]
                            }
                        }
                    }
                }
                None => self
                    .env
                    .functions()
                    .values()
                    .flat_map(|func| {
                        let func_valid = match sym_tab.get(func.name) {
                            Some(d) => {
                                if let Some(_) = self.env.types().get_func_content(d.ty) {
                                    true
                                } else {
                                    false
                                }
                            }
                            None => true,
                        };
                        if func_valid {
                            Some(CIdSelected::FuncName(func.clone()))
                        } else {
                            None
                        }
                    })
                    .chain(self.env.default_functions().values().flat_map(|func| {
                        match sym_tab.get(func.name) {
                            Some(_) => None,
                            None => Some(CIdSelected::FuncName(func.clone())),
                        }
                    }))
                    .collect(),
            },
            CIdSelector::Identifier(sym_tab) => match literal {
                Some(literal) => {
                    let literal = self.env.str_pool().get(literal).unwrap();
                    match sym_tab.get(literal) {
                        Some(_) => vec![CIdSelected::Identifier(literal)],
                        None => vec![],
                    }
                }
                None => sym_tab
                    .iter()
                    .flat_map(|(name, _)| {
                        if self.env.is_true_id(*name) {
                            Some(CIdSelected::Identifier(*name))
                        } else {
                            None
                        }
                    })
                    .collect(),
            },
            CIdSelector::NewIdentifier(sym_tab) => {
                if let Some(literal) = literal {
                    let literal = self.env.str_pool().get(literal).unwrap();
                    match sym_tab.get(literal) {
                        Some(s) => {
                            if s.current_scope() {
                                vec![]
                            } else {
                                vec![CIdSelected::NewIdentifier(literal)]
                            }
                        }
                        None => {
                            vec![CIdSelected::NewIdentifier(literal)]
                        }
                    }
                } else {
                    self.env
                        .identifiers()
                        .iter()
                        .flat_map(|id| match sym_tab.get(*id) {
                            Some(s) => {
                                if s.current_scope() {
                                    None
                                } else {
                                    Some(CIdSelected::NewIdentifier(*id))
                                }
                            }
                            None => Some(CIdSelected::NewIdentifier(*id)),
                        })
                        .collect()
                }
            }
        }
    }

    fn stg_IDENTIFIER(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CIdSelector<'a>,
        syn: &CIdSelected<'a>,
        _literal: Option<&str>,
    ) -> String {
        match syn {
            CIdSelected::FuncName(func) => func.name.to_string(),
            CIdSelected::Identifier(id) => id.to_string(),
            CIdSelected::NewIdentifier(id) => id.to_string(),
        }
    }

    fn sts_LITERAL_INT(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _literal: Option<&str>,
    ) -> PropEmpty {
        PropEmpty
    }

    fn stg_LITERAL_INT(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _syn: &PropEmpty,
        literal: Option<&str>,
    ) -> String {
        match literal {
            Some(literal) => literal.to_string(),
            None => "0".to_string(),
        }
    }

    fn stg_LITERAL_STRING(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _syn: &PropEmpty,
        literal: Option<&str>,
    ) -> String {
        match literal {
            Some(literal) => literal.to_string(),
            None => "\"\"".to_string(),
        }
    }

    fn stg_LITERAL_FLOAT(
        &self,
        _g: &PropArray<CProp>,
        _inh: &CSymTab<'a>,
        _syn: &PropEmpty,
        literal: Option<&str>,
    ) -> String {
        match literal {
            Some(literal) => literal.to_string(),
            None => "0.0".to_string(),
        }
    }
}
