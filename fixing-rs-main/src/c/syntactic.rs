use fixing_rs_base::{props::PropEmpty, union_prop};

pub enum CExprOps {
    Plus,
    Move,
    Compare,
    Equals,
    Bit,
    Bool,
    Assign,
    AssignAdd,
    AssignMove,
    AssignBit,
    Comma,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Prop)]
pub enum OperatorPrecedence {
    Primary,        // 0
    Postfix,        // 1
    Unary,          // 2 prefix, ++, --, sizeof must be unary, other cast
    Cast,           // 3
    Multiplicative, // 4
    Additive,       // 5
    Shift,          // 6
    Relational,     // 7
    Equality,       // 8
    BitAnd,         // 9
    BitXor,         // 10
    BitOr,          // 11
    LogicalAnd,     // 12
    LogicalOr,      // 13
    Conditional,    // 14, ?:
    Assignment,     // 15, left must be unary
    Comma,          // 16
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Prop)]
pub enum StatementInfo {
    NoElse,
    WithElse,
}

union_prop!(
    CProp,
    Empty,
    {
        Empty(PropEmpty),
        Expr(OperatorPrecedence),
        Statement(StatementInfo)
    }
);

pub struct CGProcessor;

#[impl_syntactic_processor(g_prop = "CProp", grammar_file = "fixing-rs-main/src/c/c_grammar")]
#[allow(non_snake_case)]
impl CGProcessor {
    // exprbinop : 0 *
    fn nt_exprbinop_0(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Multiplicative
    }

    // exprbinop : 1 /
    fn nt_exprbinop_1(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Multiplicative
    }

    // exprbinop : 2 %
    fn nt_exprbinop_2(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Multiplicative
    }

    // exprbinop : 3 +
    fn nt_exprbinop_3(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Additive
    }

    // exprbinop : 4 -
    fn nt_exprbinop_4(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Additive
    }

    // exprbinop : 5 <<
    fn nt_exprbinop_5(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Shift
    }

    // exprbinop : 6 >>
    fn nt_exprbinop_6(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Shift
    }

    // exprbinop : 7 <
    fn nt_exprbinop_7(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Relational
    }

    // exprbinop : 8 <=
    fn nt_exprbinop_8(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Relational
    }

    // exprbinop : 9 >
    fn nt_exprbinop_9(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Relational
    }

    // exprbinop : 10 >=
    fn nt_exprbinop_10(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Relational
    }

    // exprbinop : 11 ==
    fn nt_exprbinop_11(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Equality
    }

    // exprbinop : 12 !=
    fn nt_exprbinop_12(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Equality
    }

    // exprbinop : 13 &
    fn nt_exprbinop_13(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::BitAnd
    }

    // exprbinop : 14 ^
    fn nt_exprbinop_14(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::BitXor
    }

    // exprbinop : 15 |
    fn nt_exprbinop_15(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::BitOr
    }

    // exprbinop : 16 &&
    fn nt_exprbinop_16(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::LogicalAnd
    }

    // exprbinop : 17 ||
    fn nt_exprbinop_17(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::LogicalOr
    }

    // exprbinop : 18 =
    fn nt_exprbinop_18(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 19 +=
    fn nt_exprbinop_19(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 20 -=
    fn nt_exprbinop_20(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 21 *=
    fn nt_exprbinop_21(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 22 /=
    fn nt_exprbinop_22(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 23 %=
    fn nt_exprbinop_23(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 24 <<=
    fn nt_exprbinop_24(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 25 >>=
    fn nt_exprbinop_25(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 26 &=
    fn nt_exprbinop_26(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 27 ^=
    fn nt_exprbinop_27(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 28 |=
    fn nt_exprbinop_28(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Assignment
    }

    // exprbinop : 29 ,
    fn nt_exprbinop_29(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Comma
    }

    // expr : 0 IDENTIFIER
    fn nt_expr_0(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Primary
    }

    // expr : 1 literal
    fn nt_expr_1(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Primary
    }

    // expr : 2 expr exprsuffix
    fn nt_expr_2(&self, expr: &OperatorPrecedence, _s2: &PropEmpty) -> Option<OperatorPrecedence> {
        if *expr <= OperatorPrecedence::Postfix {
            Some(OperatorPrecedence::Postfix)
        } else {
            None
        }
    }

    // expr : 3 exprprefix expr
    fn nt_expr_3(&self, _s1: &PropEmpty, expr: &OperatorPrecedence) -> Option<OperatorPrecedence> {
        if *expr <= OperatorPrecedence::Unary {
            Some(OperatorPrecedence::Unary)
        } else {
            None
        }
    }

    // expr : 4 expr exprbinop expr
    fn nt_expr_4(
        &self,
        expr1: &OperatorPrecedence,
        op: &OperatorPrecedence,
        expr2: &OperatorPrecedence,
    ) -> Option<OperatorPrecedence> {
        match op {
            OperatorPrecedence::Assignment => {
                if *expr1 <= OperatorPrecedence::Unary && *expr2 <= OperatorPrecedence::Assignment {
                    Some(OperatorPrecedence::Assignment)
                } else {
                    None
                }
            }
            _ => {
                if *expr1 <= *op && *expr2 < *op {
                    Some(*op)
                } else {
                    None
                }
            }
        }
    }

    // expr : 5 expr ? expr : expr
    fn nt_expr_5(
        &self,
        expr1: &OperatorPrecedence,
        _s2: &PropEmpty,
        _expr2: &OperatorPrecedence,
        _s4: &PropEmpty,
        expr3: &OperatorPrecedence,
    ) -> Option<OperatorPrecedence> {
        if *expr1 < OperatorPrecedence::Conditional && *expr3 <= OperatorPrecedence::Conditional {
            Some(OperatorPrecedence::Conditional)
        } else {
            None
        }
    }

    // expr : 6 ( expr )
    fn nt_expr_6(
        &self,
        _s1: &PropEmpty,
        _s2: &OperatorPrecedence,
        _s3: &PropEmpty,
    ) -> OperatorPrecedence {
        OperatorPrecedence::Primary
    }

    // expr : 7 IDENTIFIER ( args )
    fn nt_expr_7(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
    ) -> OperatorPrecedence {
        OperatorPrecedence::Primary
    }

    // expr : 8 ( typeExpr ) expr
    fn nt_expr_8(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        expr: &OperatorPrecedence,
    ) -> Option<OperatorPrecedence> {
        if *expr <= OperatorPrecedence::Cast {
            Some(OperatorPrecedence::Cast)
        } else {
            None
        }
    }

    // expr : 9 sizeof expr
    fn nt_expr_9(&self, _s1: &PropEmpty, expr: &OperatorPrecedence) -> Option<OperatorPrecedence> {
        if *expr <= OperatorPrecedence::Unary {
            Some(OperatorPrecedence::Unary)
        } else {
            None
        }
    }

    // expr : 10 sizeof ( typeExpr )
    fn nt_expr_10(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
    ) -> OperatorPrecedence {
        OperatorPrecedence::Unary
    }

    // expr : 11 exprprefixCast expr
    fn nt_expr_11(&self, _s1: &PropEmpty, expr: &OperatorPrecedence) -> Option<OperatorPrecedence> {
        if *expr <= OperatorPrecedence::Cast {
            Some(OperatorPrecedence::Unary)
        } else {
            None
        }
    }

    // stmt : 0 { stmtList }
    fn nt_stmt_0(&self, _s1: &PropEmpty, _s2: &PropEmpty, _s3: &PropEmpty) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 1 expr ;
    fn nt_stmt_1(&self, _s1: &OperatorPrecedence, _s2: &PropEmpty) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 2 if ( expr ) stmt
    fn nt_stmt_2(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &OperatorPrecedence,
        _s4: &PropEmpty,
        _s5: &StatementInfo,
    ) -> StatementInfo {
        StatementInfo::NoElse
    }

    // stmt : 3 if ( expr ) stmt else stmt
    fn nt_stmt_3(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &OperatorPrecedence,
        _s4: &PropEmpty,
        stmt: &StatementInfo,
        _s6: &PropEmpty,
        _s7: &StatementInfo,
    ) -> Option<StatementInfo> {
        if *stmt == StatementInfo::WithElse {
            Some(StatementInfo::WithElse)
        } else {
            None
        }
    }

    // stmt : 4 break ;
    fn nt_stmt_4(&self, _s1: &PropEmpty, _s2: &PropEmpty) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 5 continue ;
    fn nt_stmt_5(&self, _s1: &PropEmpty, _s2: &PropEmpty) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 6 do stmt while ( expr ) ;
    fn nt_stmt_6(
        &self,
        _s1: &PropEmpty,
        _s2: &StatementInfo,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
        _s5: &OperatorPrecedence,
        _s6: &PropEmpty,
        _s7: &PropEmpty,
    ) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 7 while ( expr ) stmt
    fn nt_stmt_7(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &OperatorPrecedence,
        _s4: &PropEmpty,
        _s5: &StatementInfo,
    ) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 8 for ( for1 for2 ; for3 ) stmt
    fn nt_stmt_8(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
        _s5: &PropEmpty,
        _s6: &PropEmpty,
        _s7: &PropEmpty,
        _s8: &StatementInfo,
    ) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 9 switch ( expr ) { switchBlock }
    fn nt_stmt_9(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &OperatorPrecedence,
        _s4: &PropEmpty,
        _s5: &PropEmpty,
        _s6: &PropEmpty,
        _s7: &PropEmpty,
    ) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 12 return ;
    fn nt_stmt_12(&self, _s1: &PropEmpty, _s2: &PropEmpty) -> StatementInfo {
        StatementInfo::WithElse
    }

    // stmt : 13 return expr ;
    fn nt_stmt_13(
        &self,
        _s1: &PropEmpty,
        _s2: &OperatorPrecedence,
        _s3: &PropEmpty,
    ) -> StatementInfo {
        StatementInfo::WithElse
    }

    // nt initializer: 0 expr
    fn nt_initializer_0(&self, expr: &OperatorPrecedence) -> Option<PropEmpty> {
        if *expr < OperatorPrecedence::Comma {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nt declId : 0 IDENTIFIER
    fn nt_declId_0(&self, _s1: &PropEmpty) -> OperatorPrecedence {
        OperatorPrecedence::Primary
    }

    // nt declId : 1 * declId
    fn nt_declId_1(&self, _s1: &PropEmpty, _s2: &OperatorPrecedence) -> OperatorPrecedence {
        OperatorPrecedence::Unary
    }

    // nt declId : 3 * const declId
    fn nt_declId_3(
        &self,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &OperatorPrecedence,
    ) -> OperatorPrecedence {
        OperatorPrecedence::Unary
    }

    // nt declId : 2 declId [ LITERAL_INT ]
    fn nt_declId_2(
        &self,
        decl_id: &OperatorPrecedence,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
    ) -> Option<OperatorPrecedence> {
        if *decl_id <= OperatorPrecedence::Postfix {
            Some(OperatorPrecedence::Postfix)
        } else {
            None
        }
    }

    // nt declId : 4 declId [ ]
    fn nt_declId_4(
        &self,
        decl_id: &OperatorPrecedence,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
    ) -> Option<OperatorPrecedence> {
        if *decl_id <= OperatorPrecedence::Postfix {
            Some(OperatorPrecedence::Postfix)
        } else {
            None
        }
    }

    // nt declId : 5 ( declId )
    fn nt_declId_5(
        &self,
        _s1: &PropEmpty,
        _s2: &OperatorPrecedence,
        _s3: &PropEmpty,
    ) -> OperatorPrecedence {
        OperatorPrecedence::Primary
    }

    // nt argsOther : 0 expr
    fn nt_argsOther_0(&self, expr: &OperatorPrecedence) -> Option<PropEmpty> {
        if *expr <= OperatorPrecedence::Assignment {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nt argsOther : 1 expr , argsOther
    fn nt_argsOther_1(
        &self,
        expr: &OperatorPrecedence,
        _s2: &PropEmpty,
        _s3: &PropEmpty,
    ) -> Option<PropEmpty> {
        if *expr <= OperatorPrecedence::Assignment {
            Some(PropEmpty)
        } else {
            None
        }
    }
}
