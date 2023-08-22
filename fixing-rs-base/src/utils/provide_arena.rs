use super::RefArena;

pub trait ProvideArena<T> {
    fn provide_arena(&self) -> &RefArena<T>;
}

pub trait GetArena {
    fn get_arena<T>(&self) -> &RefArena<T>
    where
        Self: ProvideArena<T>;
}

impl<A> GetArena for A {
    fn get_arena<T>(&self) -> &RefArena<T>
    where
        Self: ProvideArena<T>,
    {
        self.provide_arena()
    }
}
