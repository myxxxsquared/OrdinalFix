use std::collections::HashSet;

use super::{
    mjenv::{MJClsRef, MJEnv},
    mjsymtab::{MJArgs, MJDecl, MJIdSelected, MJIdSelector, MJSymTab},
    syntactic::MJProp,
};
use fixing_rs_base::{
    props::{PropArray, PropEmpty},
    tokenizer::Token,
    union_prop,
    utils::{RefArena, StringPool, StringRef},
};

pub struct MJSProcessor<'a> {
    mjenv: &'a MJEnv<'a>,
    str_pool: StringPool<'a>,
    identifiers: Vec<StringRef<'a>>,
}

impl<'a> MJSProcessor<'a> {
    pub fn new(
        mjenv: &'a MJEnv<'a>,
        arena: &'a RefArena<String>,
        tokens: &Vec<Token<'_, '_>>,
        max_new_id: usize,
    ) -> Self {
        let mut str_pool = StringPool::new(arena);

        let mut all_identifiers = HashSet::new();
        for token in tokens {
            if token.symbol.name() == "IDENTIFIER" {
                all_identifiers.insert(token.literal.to_owned());
            }
        }
        for token in mjenv.iter_names() {
            all_identifiers.insert(token.to_string());
        }
        for i in 0..max_new_id {
            all_identifiers.insert(format!("__new_id_{}", i));
        }

        let mut identifiers = Vec::new();
        for ident in all_identifiers {
            identifiers.push(str_pool.get_or_add(&ident[..]));
        }

        Self {
            str_pool,
            mjenv,
            identifiers,
        }
    }
}

union_prop!(
    MJSynProp<'a>,
    Empty,
    {
        Empty(PropEmpty),
        Type(MJClsRef<'a>),
        Decl(MJDecl<'a>),
        Str(StringRef<'a>),
        IdSelected(MJIdSelected<'a>)
    }
);

union_prop!(
    MJInhProp<'a>,
    Empty,
    {
        Empty(PropEmpty),
        SymTab(MJSymTab<'a>),
        IdSelector(MJIdSelector<'a>),
        Args(MJArgs<'a>)
    }
);

#[impl_semantic_processor(
    g_prop = "MJProp",
    si_prop = "MJInhProp<'a>",
    ss_prop = "MJSynProp<'a>",
    grammar_file = "fixing-rs-main/src/mj/middle_weight_java"
)]
#[allow(non_snake_case)]
impl<'a> MJSProcessor<'a> {
    fn rooti(&self) -> MJSymTab<'a> {
        MJSymTab::new()
    }

    //nts statements: 1 statement statements
    fn nti_statements_1_1(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJSymTab<'a>,
        left: &MJDecl<'a>,
    ) -> MJSymTab<'a> {
        inh.extend_decl(left)
    }

    //nts statement: 0 ';'
    fn nts_statement_0(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
    ) -> MJDecl<'a> {
        MJDecl::empty()
    }

    //nts statement: 1 declaration
    fn nts_statement_1(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        decl: &MJDecl<'a>,
    ) -> MJDecl<'a> {
        decl.clone()
    }

    //nts statement: 2 pExpression ';'
    fn nts_statement_2(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &MJClsRef<'a>,
        _s2: &PropEmpty,
    ) -> MJDecl<'a> {
        MJDecl::empty()
    }

    //nts statement: 4 expression '.' fieldName '=' expression ';'
    fn nts_statement_4(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &MJClsRef<'a>,
        _s2: &PropEmpty,
        left: &MJIdSelected<'a>,
        _s4: &PropEmpty,
        right: &MJClsRef<'a>,
        _s6: &PropEmpty,
    ) -> Option<MJDecl<'a>> {
        if self
            .mjenv
            .can_right_assign_to_left(left.unwrap_ty(), *right)
        {
            Some(MJDecl::empty())
        } else {
            None
        }
    }

    //nts statement: 5 identifier '=' expression ';'
    fn nts_statement_5(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        left: &MJIdSelected<'a>,
        _s2: &PropEmpty,
        right: &MJClsRef<'a>,
        _s4: &PropEmpty,
    ) -> Option<MJDecl<'a>> {
        if self
            .mjenv
            .can_right_assign_to_left(left.unwrap_ty(), *right)
        {
            Some(MJDecl::empty())
        } else {
            None
        }
    }

    //nts statement: 6 'return' expression ';'
    fn nts_statement_6(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &MJClsRef<'a>,
        _s3: &PropEmpty,
    ) -> Option<MJDecl<'a>> {
        None
    }

    //nts statement: 7 'if' '(' expression '==' expression ')' block 'else' block
    fn nts_statement_7(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
        _s3: &MJClsRef<'a>,
        _s4: &PropEmpty,
        _s5: &MJClsRef<'a>,
        _s6: &PropEmpty,
        _s7: &PropEmpty,
        _s8: &PropEmpty,
        _s9: &PropEmpty,
    ) -> MJDecl<'a> {
        MJDecl::empty()
    }

    //nts statement: 8 block
    fn nts_statement_8(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
    ) -> MJDecl<'a> {
        MJDecl::empty()
    }

    //nts statement: 9 'return' ';'
    fn nts_statement_9(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
        _s2: &PropEmpty,
    ) -> MJDecl<'a> {
        MJDecl::empty()
    }

    //nti 2 statement: 4 expression '.' fieldName '=' expression ';'
    fn nti_statement_4_2(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        expr: &MJClsRef<'a>,
        _s2: &PropEmpty,
    ) -> MJIdSelector<'a> {
        MJIdSelector::Field(expr.clone())
    }

    //nti 0 statement: 5 identifier '=' expression ';'
    fn nti_statement_5_0(&self, _g: &PropArray<MJProp>, inh: &MJSymTab<'a>) -> MJIdSelector<'a> {
        MJIdSelector::Identifier(inh.clone())
    }

    //nts declaration: 0 className newIdentifier ';'
    fn nts_declaration_0(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        ty: &MJIdSelected<'a>,
        name: &MJIdSelected<'a>,
        _s3: &PropEmpty,
    ) -> MJDecl<'a> {
        MJDecl::new(name.unwrap_newid(), ty.unwrap_ty())
    }

    //nti 0 declaration: 0 className newIdentifier ';'
    fn nti_declaration_0_0(&self, _g: &PropArray<MJProp>, _inh: &MJSymTab<'a>) -> MJIdSelector<'a> {
        MJIdSelector::Class
    }

    //nti 1 declaration: 0 className newIdentifier ';'
    fn nti_declaration_0_1(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &MJIdSelected<'a>,
    ) -> MJIdSelector<'a> {
        MJIdSelector::NewIdentifier
    }

    //nts expression: 0 identifier
    fn nts_expression_0(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        id: &MJIdSelected<'a>,
    ) -> MJClsRef<'a> {
        id.unwrap_ty()
    }

    // nts expression: 1 'null'
    fn nts_expression_1(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
    ) -> MJClsRef<'a> {
        self.mjenv.get_default_null()
    }

    // nts expression: 2 expression '.' fieldName
    fn nts_expression_2(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &MJClsRef<'a>,
        _s2: &PropEmpty,
        id: &MJIdSelected<'a>,
    ) -> MJClsRef<'a> {
        id.unwrap_ty()
    }

    // nts expression: 3 '(' className ')' expression
    fn nts_expression_3(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
        ty: &MJIdSelected<'a>,
        _s3: &PropEmpty,
        _s4: &MJClsRef<'a>,
    ) -> MJClsRef<'a> {
        ty.unwrap_ty()
    }

    // nts expression: 4 pExpression
    fn nts_expression_4(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        pexp: &MJClsRef<'a>,
    ) -> MJClsRef<'a> {
        *pexp
    }

    // nts expression: 5 '(' expression ')'
    fn nts_expression_5(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
        exp: &MJClsRef<'a>,
        _s3: &PropEmpty,
    ) -> MJClsRef<'a> {
        *exp
    }

    // nti 0 expression: 0 identifier
    fn nti_expression_0_0(&self, _g: &PropArray<MJProp>, inh: &MJSymTab<'a>) -> MJIdSelector<'a> {
        MJIdSelector::Identifier(inh.clone())
    }

    //nti 2 expression: 2 expression '.' fieldName
    fn nti_expression_2_2(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        exp: &MJClsRef<'a>,
        _s2: &PropEmpty,
    ) -> MJIdSelector<'a> {
        MJIdSelector::Field(*exp)
    }

    // nti 1 expression: 3 '(' className ')' expression
    fn nti_expression_3_1(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
    ) -> MJIdSelector<'a> {
        MJIdSelector::Class
    }

    // nts pExpression: 0 expression '.' methodName '(' argumentList ')'
    fn nts_pExpression_0(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &MJClsRef<'a>,
        _s2: &PropEmpty,
        id: &MJIdSelected<'a>,
        _s4: &PropEmpty,
        _s5: &PropEmpty,
        _s6: &PropEmpty,
    ) -> MJClsRef<'a> {
        match id.unwrap_method().ret_ty() {
            Some(ty) => *ty,
            None => self.mjenv.get_default_void(),
        }
    }

    // nts pExpression: 1 'new' className '(' argumentList ')'
    fn nts_pExpression_1(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
        id: &MJIdSelected<'a>,
        _s3: &PropEmpty,
        _s4: &PropEmpty,
        _s5: &PropEmpty,
    ) -> MJClsRef<'a> {
        id.unwrap_ty()
    }

    // nti 2 pExpression: 0 expression '.' fieldName '(' argumentList ')'
    fn nti_pExpression_0_2(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        exp: &MJClsRef<'a>,
        _s2: &PropEmpty,
    ) -> MJIdSelector<'a> {
        MJIdSelector::Method(*exp)
    }

    // nti 1 pExpression: 1 'new' className '(' argumentList ')'
    fn nti_pExpression_1_1(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
    ) -> MJIdSelector<'a> {
        MJIdSelector::Class
    }

    // nti 4 pExpression: 0 expression '.' fieldName '(' argumentList ')'
    fn nti_pExpression_0_4(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJSymTab<'a>,
        _s1: &MJClsRef<'a>,
        _s2: &PropEmpty,
        id: &MJIdSelected<'a>,
        _s4: &PropEmpty,
    ) -> MJArgs<'a> {
        MJArgs::Method(id.unwrap_method(), 0, inh.clone())
    }

    // nti 3 pExpression: 1 'new' className '(' argumentList ')'
    fn nti_pExpression_1_3(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJSymTab<'a>,
        _s1: &PropEmpty,
        id: &MJIdSelected<'a>,
        _s3: &PropEmpty,
    ) -> MJArgs<'a> {
        MJArgs::Constructor(id.unwrap_ty().constructor(), 0, inh.clone())
    }

    // nts argumentList: 0
    fn nts_argumentList_0(&self, _g: &PropArray<MJProp>, inh: &MJArgs<'a>) -> Option<PropEmpty> {
        let m = match inh {
            MJArgs::Method(m, _, _) => m.params(),
            MJArgs::Constructor(m, _, _) => m.params(),
        };
        if m.len() == 0 {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nti 0 argumentList: 1 argumentListOther
    fn nti_argumentList_1_0(&self, _g: &PropArray<MJProp>, inh: &MJArgs<'a>) -> Option<MJArgs<'a>> {
        let m = match inh {
            MJArgs::Method(m, _, _) => m.params(),
            MJArgs::Constructor(m, _, _) => m.params(),
        };
        if m.len() > 0 {
            Some(inh.clone())
        } else {
            None
        }
    }

    // nts argumentListOther: 0 expression
    fn nts_argumentListOther_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJArgs<'a>,
        exp: &MJClsRef<'a>,
    ) -> Option<PropEmpty> {
        let (m, c) = match inh {
            MJArgs::Method(m, c, _) => (m.params(), c),
            MJArgs::Constructor(m, c, _) => (m.params(), c),
        };
        if self.mjenv.can_right_assign_to_left(m[*c], *exp) {
            Some(PropEmpty)
        } else {
            None
        }
    }

    // nti 0 argumentListOther: 0 expression
    fn nti_argumentListOther_0_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJArgs<'a>,
    ) -> Option<MJSymTab<'a>> {
        let (m, c, i) = match inh {
            MJArgs::Method(m, c, i) => (m.params(), c, i),
            MJArgs::Constructor(m, c, i) => (m.params(), c, i),
        };
        if m.len() == c + 1 {
            Some(i.clone())
        } else {
            None
        }
    }

    // nti 0 argumentListOther: 1 expression ',' argumentListOther
    fn nti_argumentListOther_1_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJArgs<'a>,
    ) -> Option<MJSymTab<'a>> {
        let (m, c, i) = match inh {
            MJArgs::Method(m, c, i) => (m.params(), c, i),
            MJArgs::Constructor(m, c, i) => (m.params(), c, i),
        };
        if m.len() > c + 1 {
            Some(i.clone())
        } else {
            None
        }
    }

    // nti 2 argumentListOther: 1 expression ',' argumentListOther
    fn nti_argumentListOther_1_2(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJArgs<'a>,
        exp: &MJClsRef<'a>,
        _s2: &PropEmpty,
    ) -> Option<MJArgs<'a>> {
        let (m, c) = match inh {
            MJArgs::Method(m, c, _) => (m.params(), c),
            MJArgs::Constructor(m, c, _) => (m.params(), c),
        };
        if !self.mjenv.can_right_assign_to_left(m[*c], *exp) {
            return None;
        }
        Some(inh.next())
    }

    // nts identifier: 0 IDENTIFIER
    fn nts_identifier_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJIdSelector<'a>,
        id: &StringRef<'a>,
    ) -> MJIdSelected<'a> {
        match inh {
            MJIdSelector::Identifier(sym_tab) => {
                MJIdSelected::Identifier(sym_tab.get(*id).unwrap())
            }
            _ => panic!("not identifier"),
        }
    }

    // nts className: 0 IDENTIFIER
    fn nts_className_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJIdSelector<'a>,
        id: &StringRef<'a>,
    ) -> MJIdSelected<'a> {
        match inh {
            MJIdSelector::Class => {
                MJIdSelected::Identifier(self.mjenv.get_class(id.as_str()).unwrap())
            }
            _ => panic!("not classname"),
        }
    }

    // nts fieldName: 0 IDENTIFIER
    fn nts_fieldName_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJIdSelector<'a>,
        id: &StringRef<'a>,
    ) -> MJIdSelected<'a> {
        match inh {
            MJIdSelector::Field(c) => MJIdSelected::Identifier(
                *c.content().borrow().fields().get(id.as_str()).unwrap().ty(),
            ),
            _ => panic!("not field"),
        }
    }

    // nts methodName: 0 IDENTIFIER
    fn nts_methodName_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJIdSelector<'a>,
        id: &StringRef<'a>,
    ) -> MJIdSelected<'a> {
        match inh {
            MJIdSelector::Method(c) => {
                MJIdSelected::Method(*c.content().borrow().methods().get(id.as_str()).unwrap())
            }
            _ => panic!("not method"),
        }
    }

    // nts newIdentifier: 0 IDENTIFIER
    fn nts_newIdentifier_0(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJIdSelector<'a>,
        id: &StringRef<'a>,
    ) -> MJIdSelected<'a> {
        match inh {
            MJIdSelector::NewIdentifier => MJIdSelected::NewId(*id),
            _ => panic!("not new"),
        }
    }

    // sts IDENTIFIER
    fn sts_IDENTIFIER(
        &self,
        _g: &PropArray<MJProp>,
        inh: &MJIdSelector<'a>,
        literal: Option<&str>,
    ) -> Vec<StringRef<'a>> {
        let mut result = Vec::new();
        let literal = literal.map(|s| self.str_pool.get(s).unwrap());
        match inh {
            MJIdSelector::Identifier(sym_tab) => match literal {
                Some(literal) => {
                    if let Some(_) = sym_tab.get(literal) {
                        result.push(literal);
                    }
                }
                None => {
                    for (k, _) in sym_tab.iter() {
                        result.push(*k);
                    }
                }
            },
            MJIdSelector::Field(c) => match literal {
                Some(literal) => {
                    if let Some(_) = c.content().borrow().fields().get(literal.as_str()) {
                        result.push(literal);
                    }
                }
                None => {
                    for (k, _) in c.content().borrow().fields().iter() {
                        result.push(self.str_pool.get(k).unwrap());
                    }
                }
            },
            MJIdSelector::Method(c) => match literal {
                Some(literal) => {
                    if let Some(_) = c.content().borrow().methods().get(literal.as_str()) {
                        result.push(literal);
                    }
                }
                None => {
                    for (k, _) in c.content().borrow().methods().iter() {
                        result.push(self.str_pool.get(k).unwrap());
                    }
                }
            },
            MJIdSelector::Class => match literal {
                Some(literal) => {
                    if let Some(_) = self.mjenv.get_class(literal.as_str()) {
                        result.push(literal);
                    }
                }
                None => {
                    for c in self.mjenv.iter_class() {
                        result.push(self.str_pool.get(c.name()).unwrap());
                    }
                }
            },
            MJIdSelector::NewIdentifier => match literal {
                Some(literal) => result.push(literal),
                None => {
                    for c in self.identifiers.iter() {
                        result.push(*c);
                    }
                }
            },
        }
        result
    }

    // stg IDENTIFIER
    fn stg_IDENTIFIER(
        &self,
        _g: &PropArray<MJProp>,
        _inh: &MJIdSelector<'a>,
        syn: &StringRef<'a>,
        _literal: Option<&str>,
    ) -> String {
        syn.to_string()
    }
}
