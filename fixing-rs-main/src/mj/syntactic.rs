use fixing_rs_base::{props::PropEmpty, union_prop};

union_prop! {
    MJProp,
    Empty,
    {
        Empty(PropEmpty),
        ExpressionPriority(MJExpressionPriority)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub enum MJExpressionPriority {
    NoLeft,
    HaveLeft,
}

pub struct MJGProcessor;

extern crate fixing_rs_macros;
use fixing_rs_macros::impl_syntactic_processor;

#[impl_syntactic_processor(
    g_prop = "MJProp",
    grammar_file = "fixing-rs-main/src/mj/middle_weight_java"
)]
#[allow(unused, non_snake_case)]
impl MJGProcessor {
    // identifier
    fn nt_expression_0(&self, _: &PropEmpty) -> MJExpressionPriority {
        MJExpressionPriority::NoLeft
    }

    // 'null'
    fn nt_expression_1(&self, _: &PropEmpty) -> MJExpressionPriority {
        MJExpressionPriority::NoLeft
    }

    // expression '.' fieldName
    fn nt_expression_2(
        &self,
        subexp: &MJExpressionPriority,
        _: &PropEmpty,
        _: &PropEmpty,
    ) -> Option<MJExpressionPriority> {
        match subexp {
            MJExpressionPriority::HaveLeft => None,
            MJExpressionPriority::NoLeft => Some(MJExpressionPriority::NoLeft),
        }
    }

    // '(' className ')' expression
    fn nt_expression_3(
        &self,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &MJExpressionPriority,
    ) -> MJExpressionPriority {
        MJExpressionPriority::HaveLeft
    }

    // pExpression
    fn nt_expression_4(&self, subexp: &MJExpressionPriority) -> MJExpressionPriority {
        subexp.clone()
    }

    // '(' expression ')'
    fn nt_expression_5(
        &self,
        _: &PropEmpty,
        _: &MJExpressionPriority,
        _: &PropEmpty,
    ) -> MJExpressionPriority {
        MJExpressionPriority::NoLeft
    }

    // expression '.' methodName '(' argumentList ')'
    fn nt_pExpression_0(
        &self,
        subexp: &MJExpressionPriority,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
    ) -> Option<MJExpressionPriority> {
        match subexp {
            MJExpressionPriority::HaveLeft => None,
            MJExpressionPriority::NoLeft => Some(MJExpressionPriority::NoLeft),
        }
    }

    // 'new' className '(' argumentList ')'
    fn nt_pExpression_1(
        &self,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
    ) -> MJExpressionPriority {
        MJExpressionPriority::NoLeft
    }

    // expression '.' fieldName '=' expression ';'
    fn nt_statement_4(
        &self,
        subexp: &MJExpressionPriority,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &PropEmpty,
        _: &MJExpressionPriority,
        _: &PropEmpty,
    ) -> Option<PropEmpty> {
        match subexp {
            MJExpressionPriority::HaveLeft => None,
            MJExpressionPriority::NoLeft => Some(PropEmpty),
        }
    }
}
