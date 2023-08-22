use super::Prop;

pub trait IntoUnionProp<T> {
    fn into_union_prop(self) -> T;
}

pub trait IntoSingleProp<T> {
    fn into_single_prop(&self, err_msg: &str) -> &T;
}

impl<T1, T2> IntoUnionProp<Option<T2>> for Option<T1>
where
    T1: IntoUnionProp<T2>,
{
    fn into_union_prop(self) -> Option<T2> {
        self.map(|t| t.into_union_prop())
    }
}

impl<T1, T2> IntoUnionProp<Vec<T2>> for Vec<T1>
where
    T1: IntoUnionProp<T2>,
{
    fn into_union_prop(self) -> Vec<T2> {
        self.into_iter().map(|t| t.into_union_prop()).collect()
    }
}

impl<T1, T2> IntoUnionProp<Box<[T2]>> for Box<[T1]>
where
    T1: IntoUnionProp<T2>,
{
    fn into_union_prop(self) -> Box<[T2]> {
        self.into_vec()
            .into_iter()
            .map(|t| t.into_union_prop())
            .collect()
    }
}

pub trait UnionProp: Prop + Default {}

#[macro_export]
macro_rules! union_prop {
    {$name:ident, $def:ident, { $( $item:ident($type:ident) ),* }} => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Prop)]
        pub enum $name {
            $( $item ( $type ) ),*
        }
        $(
            impl $crate::props::IntoUnionProp < $name > for $type {
                fn into_union_prop(self) -> $name {
                    $name :: $item ( self )
                }
            }
            impl $crate::props::IntoSingleProp < $type > for $name {
                fn into_single_prop(&self, err_msg: &str) -> & $type {
                    match self {
                        $name :: $item (ref p) => p,
                        _ => panic!("Prop excepted: {}, found: {}, msg: {}", stringify!($type), self.prop_name(), err_msg),
                    }
                }
            }
        )*
        impl Default for $name {
            fn default() -> Self {
                Self :: $def ( Default::default() )
            }
        }
        impl $crate::props::UnionProp for $name {}
        impl $name {
            pub fn prop_name(&self) -> &'static str {
                match self {
                    $( $name :: $item (ref _val) => stringify!($type) ),*
                }
            }
        }
    };
    {$name:ident < $l:lifetime >, $def:ident, { $( $item:ident($type:ty) ),* }} => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name < $l >{
            $( $item ( $type ) ),*
        }
        $(
            impl< $l > $crate::props::IntoUnionProp < $name< $l > > for $type {
                fn into_union_prop(self) -> $name< $l > {
                    $name :: $item ( self )
                }
            }
            impl< $l > $crate::props::IntoSingleProp < $type > for $name< $l > {
                fn into_single_prop(&self, err_msg: &str) -> & $type {
                    match self {
                        $name :: $item (ref p) => p,
                        _ => panic!("Prop excepted: {}, found: {}, msg: {}.", stringify!($type), self.prop_name(), err_msg),
                    }
                }
            }
        )*
        impl< $l > Default for $name< $l > {
            fn default() -> Self {
                Self :: $def ( Default::default() )
            }
        }
        impl< $l > $crate::props::Prop for $name< $l > {}
        impl< $l > $crate::props::UnionProp for $name< $l > {}
        impl $name <'_> {
            pub fn prop_name(&self) -> &'static str {
                match self {
                    $( $name :: $item (ref _val) => stringify!($type) ),*
                }
            }
        }
    };
}
