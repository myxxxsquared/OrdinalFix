use std::{fmt::Debug, hash::Hash};

use super::UnionProp;

pub trait Prop: Debug + Clone + PartialEq + Eq + Hash {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropEmpty;
impl Prop for PropEmpty {}
impl UnionProp for PropEmpty {}
impl Default for PropEmpty {
    fn default() -> Self {
        Self
    }
}
