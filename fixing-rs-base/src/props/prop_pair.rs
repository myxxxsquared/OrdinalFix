use super::{Prop, PropResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropPair<P>
where
    P: Prop,
{
    pub last_induction: P,
    pub current: P,
}

pub trait AssembleWith<P>
where
    P: Prop,
{
    type Output;
    fn assemble_with(self, p: P) -> Self::Output;
}

impl<P> AssembleWith<P> for P
where
    P: Prop,
{
    type Output = PropPair<P>;
    fn assemble_with(self, p: P) -> Self::Output {
        PropPair {
            last_induction: p,
            current: self,
        }
    }
}

impl<P> AssembleWith<P> for PropResult<P>
where
    P: Prop,
{
    type Output = PropResult<PropPair<P>>;
    fn assemble_with(self, p: P) -> Self::Output {
        self.map(|t| t.assemble_with(p.clone()))
    }
}

impl<P> Prop for PropPair<P> where P: Prop {}
