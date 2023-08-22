use super::mjenv::{MJClsRef, MJConstructorRef, MJMethodRef};
use fixing_rs_base::{containers::Map, utils::StringRef};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Debug, Clone, Prop)]
pub struct MJSymTab<'a> {
    pub decled_vars: Rc<Map<StringRef<'a>, MJClsRef<'a>>>,
    hash: u64,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Prop)]
pub struct MJDecl<'a> {
    pub decl: Option<(StringRef<'a>, MJClsRef<'a>)>,
}

impl<'a> MJDecl<'a> {
    pub fn new(name: StringRef<'a>, ty: MJClsRef<'a>) -> Self {
        Self {
            decl: Some((name, ty)),
        }
    }
    pub fn empty() -> Self {
        Self { decl: None }
    }
}

impl<'a> MJSymTab<'a> {
    pub fn new() -> Self {
        Self::from_map(Map::new())
    }

    fn from_map(map: Map<StringRef<'a>, MJClsRef<'a>>) -> Self {
        let hash = Self::compute_hash(&map);
        Self {
            decled_vars: Rc::new(map),
            hash,
        }
    }

    fn compute_hash(m: &Map<StringRef<'a>, MJClsRef<'a>>) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut keys = m.keys().collect::<Vec<_>>();
        keys.sort();
        for key in keys {
            let val = m.get(key).unwrap();
            key.hash(&mut hasher);
            val.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn extend(&self, name: StringRef<'a>, ty: MJClsRef<'a>) -> Self {
        let mut new_map = self.decled_vars.as_ref().clone();
        new_map.insert(name, ty);
        Self::from_map(new_map)
    }

    pub fn extend_decl(&self, decl: &MJDecl<'a>) -> Self {
        if let Some((name, ty)) = decl.decl {
            self.extend(name, ty)
        } else {
            self.clone()
        }
    }

    pub fn get(&self, name: StringRef<'a>) -> Option<MJClsRef<'a>> {
        self.decled_vars.get(&name).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&StringRef<'a>, &MJClsRef<'a>)> {
        self.decled_vars.iter()
    }
}

impl Default for MJSymTab<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for MJSymTab<'_> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.decled_vars, &other.decled_vars)
            || (self.hash == other.hash
                && self.decled_vars.len() == other.decled_vars.len()
                && self
                    .decled_vars
                    .iter()
                    .all(|(k, v)| other.decled_vars.get(k).map_or(false, |v2| v == v2)))
    }
}

impl Eq for MJSymTab<'_> {}

impl Hash for MJSymTab<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub enum MJIdSelector<'a> {
    Identifier(MJSymTab<'a>),
    Field(MJClsRef<'a>),
    Method(MJClsRef<'a>),
    Class,
    NewIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub enum MJIdSelected<'a> {
    Identifier(MJClsRef<'a>),
    NewId(StringRef<'a>),
    Method(MJMethodRef<'a>),
}

impl<'a> MJIdSelected<'a> {
    pub fn unwrap_ty(&self) -> MJClsRef<'a> {
        match self {
            MJIdSelected::Identifier(ref ty) => *ty,
            _ => panic!("not an identifier"),
        }
    }
    pub fn unwrap_newid(&self) -> StringRef<'a> {
        match self {
            MJIdSelected::NewId(ref id) => *id,
            _ => panic!("not a new identifier"),
        }
    }
    pub fn unwrap_method(&self) -> MJMethodRef<'a> {
        match self {
            MJIdSelected::Method(ref ty) => *ty,
            _ => panic!("not a method"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
pub enum MJArgs<'a> {
    Method(MJMethodRef<'a>, usize, MJSymTab<'a>),
    Constructor(MJConstructorRef<'a>, usize, MJSymTab<'a>),
}

impl<'a> MJArgs<'a> {
    pub fn next(&self) -> Self {
        match self {
            MJArgs::Method(ref m, ref c, ref i) => MJArgs::Method(*m, c + 1, i.clone()),
            MJArgs::Constructor(ref m, ref c, ref i) => MJArgs::Constructor(*m, c + 1, i.clone()),
        }
    }
}
