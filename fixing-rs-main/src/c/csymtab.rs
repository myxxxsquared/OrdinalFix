use std::fmt::Debug;

use super::types::{CFuncContentRef, CTypeRef};
use fixing_rs_base::utils::{StringRef, SymTab};
use getset::CopyGetters;

#[derive(Clone, Hash, PartialEq, Eq, CopyGetters)]
pub struct VarInfo<'a> {
    #[get_copy = "pub"]
    pub ty: CTypeRef<'a>,
    #[get_copy = "pub"]
    pub current_scope: bool,
}

impl Debug for VarInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {}",
            self.ty,
            if self.current_scope {
                "current"
            } else {
                "outer"
            }
        )
    }
}

impl<'a> VarInfo<'a> {
    pub fn new(ty: CTypeRef<'a>) -> Self {
        Self {
            ty,
            current_scope: true,
        }
    }
    pub fn inner_scope(&self) -> Self {
        Self {
            ty: self.ty,
            current_scope: false,
        }
    }
}

pub type CSymTab<'a> = SymTab<StringRef<'a>, VarInfo<'a>>;

pub trait CSymTabExt {
    fn new_scope(&self) -> Self;
}

impl<'a> CSymTabExt for CSymTab<'a> {
    fn new_scope(&self) -> Self {
        let m = self
            .iter()
            .map(|(k, v)| (k.clone(), v.inner_scope()))
            .collect();
        Self::from_map(m)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub enum CIdSelector<'a> {
    FuncName(CSymTab<'a>),
    Identifier(CSymTab<'a>),
    NewIdentifier(CSymTab<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub enum CIdSelected<'a> {
    FuncName(CDeclaredFunc<'a>),
    Identifier(StringRef<'a>),
    NewIdentifier(StringRef<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub struct CDeclaredFunc<'a> {
    pub name: StringRef<'a>,
    pub content: CFuncContentRef<'a>,
}

impl<'a> CDeclaredFunc<'a> {
    pub fn new(name: StringRef<'a>, content: CFuncContentRef<'a>) -> Self {
        Self { name, content }
    }
}
