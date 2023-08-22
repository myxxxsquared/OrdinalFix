use super::{Prop, PropResult};

#[derive(Debug, Clone, Eq, Hash)]
pub enum PropArray<P>
where
    P: Prop,
{
    Single(P),
    Multiple(Box<[P]>),
}

impl<P> Prop for PropArray<P> where P: Prop {}

impl<P> PropArray<P>
where
    P: Prop,
{
    pub fn new_single(p: P) -> Self {
        Self::Single(p)
    }
    pub fn new_multiple(p: Vec<P>) -> Self {
        Self::Multiple(p.into())
    }
    pub fn new_zero() -> Self {
        Self::Multiple(Box::new([]))
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Single(ref _p) => 1,
            Self::Multiple(ref p) => p.len(),
        }
    }
    pub fn append(&self, p: P) -> Self {
        let mut result = Vec::with_capacity(self.len() + 1);
        match self {
            Self::Single(ref p) => {
                result.push(p.clone());
            }
            Self::Multiple(ref p) => {
                for sub in p.iter() {
                    result.push(sub.clone());
                }
            }
        }
        result.push(p);
        Self::Multiple(result.into())
    }
    pub fn unwrap_multiple(&self) -> &[P] {
        match self {
            Self::Multiple(p) => p,
            _ => panic!("Not a box of Multiple"),
        }
    }
    pub fn unwrap_single(&self) -> &P {
        match self {
            Self::Single(p) => p,
            _ => panic!("Not a box of Single"),
        }
    }
}

impl<P> PartialEq for PropArray<P>
where
    P: Prop,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Single(l0), Self::Single(r0)) => l0 == r0,
            (Self::Multiple(l0), Self::Multiple(r0)) => l0 == r0,
            _ => panic!(
                "Different element should not be compared! {:?} vs. {:?}",
                self, other
            ),
        }
    }
}

impl<P> Into<PropResult<PropArray<P>>> for PropResult<P>
where
    P: Prop,
{
    fn into(self) -> PropResult<PropArray<P>> {
        match self {
            PropResult::Empty => PropResult::Empty,
            PropResult::One(p) => PropResult::One(PropArray::Single(p)),
            PropResult::Many(ps) => PropResult::Many(
                ps.into_vec()
                    .into_iter()
                    .map(|p| PropArray::Single(p))
                    .collect(),
            ),
        }
    }
}
