use super::{IntoUnionProp, Prop, PropResult, UnionProp};

pub trait AssemblyPropResult<T>
where
    T: Prop,
{
    fn assembly_prop_result(self) -> PropResult<T>;
}

impl<T1, T2> AssemblyPropResult<T2> for T1
where
    T1: Prop + IntoUnionProp<T2>,
    T2: UnionProp,
{
    fn assembly_prop_result(self) -> PropResult<T2> {
        PropResult::One(self.into_union_prop())
    }
}

impl<T1, T2> AssemblyPropResult<T2> for Option<T1>
where
    T1: Prop + IntoUnionProp<T2>,
    T2: UnionProp,
{
    fn assembly_prop_result(self) -> PropResult<T2> {
        match self {
            Some(t) => PropResult::One(t.into_union_prop()),
            None => PropResult::Empty,
        }
    }
}

impl<T1, T2> AssemblyPropResult<T2> for Vec<T1>
where
    T1: Prop + IntoUnionProp<T2>,
    T2: UnionProp,
{
    fn assembly_prop_result(self) -> PropResult<T2> {
        match self.len() {
            0 => PropResult::Empty,
            1 => PropResult::One(self.into_iter().next().unwrap().into_union_prop()),
            _ => PropResult::Many(
                self.into_iter()
                    .map(|t| t.into_union_prop())
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        }
    }
}
