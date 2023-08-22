use super::Prop;
use std::{fmt::Debug, vec::IntoIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropResult<T>
where
    T: Prop,
{
    Empty,
    One(T),
    Many(Box<[T]>),
}

impl<T> PropResult<T>
where
    T: Prop,
{
    pub fn for_each_prop(&self, mut f: impl FnMut(&T)) {
        match self {
            PropResult::Empty => {}
            PropResult::One(p) => f(p),
            PropResult::Many(ps) => {
                for p in ps.iter() {
                    f(p)
                }
            }
        }
    }

    pub fn consume(self, mut f: impl FnMut(T)) {
        match self {
            PropResult::Empty => {}
            PropResult::One(p) => f(p),
            PropResult::Many(ps) => {
                for p in ps.into_vec().into_iter() {
                    f(p)
                }
            }
        }
    }
    pub fn map<O: Prop>(self, mut f: impl FnMut(T) -> O) -> PropResult<O> {
        match self {
            PropResult::Empty => PropResult::Empty,
            PropResult::One(p) => PropResult::One(f(p)),
            PropResult::Many(ps) => PropResult::Many(ps.into_vec().into_iter().map(f).collect()),
        }
    }
}

pub trait IntoPropResult<T>
where
    T: Prop,
{
    fn into_prop_result(self) -> PropResult<T>;
}

impl<T> IntoPropResult<T> for T
where
    T: Prop,
{
    fn into_prop_result(self) -> PropResult<T> {
        PropResult::One(self)
    }
}

impl<T> IntoPropResult<T> for Option<T>
where
    T: Prop,
{
    fn into_prop_result(self) -> PropResult<T> {
        match self {
            Some(t) => PropResult::One(t),
            None => PropResult::Empty,
        }
    }
}

impl<T> IntoPropResult<T> for Vec<T>
where
    T: Prop,
{
    fn into_prop_result(self) -> PropResult<T> {
        match self.len() {
            0 => PropResult::Empty,
            1 => PropResult::One(self.into_iter().next().unwrap()),
            _ => PropResult::Many(self.into_boxed_slice()),
        }
    }
}

pub enum PropResultIter<T>
where
    T: Prop,
{
    Empty,
    One(Option<T>),
    Many(IntoIter<T>),
}

impl<T> IntoIterator for PropResult<T>
where
    T: Prop,
{
    type Item = T;
    type IntoIter = PropResultIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            PropResult::Empty => PropResultIter::Empty,
            PropResult::One(p) => PropResultIter::One(Some(p)),
            PropResult::Many(ps) => PropResultIter::Many(ps.into_vec().into_iter()),
        }
    }
}

impl<T> Iterator for PropResultIter<T>
where
    T: Prop,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PropResultIter::Empty => None,
            PropResultIter::One(p) => p.take(),
            PropResultIter::Many(ps) => ps.next(),
        }
    }
}
