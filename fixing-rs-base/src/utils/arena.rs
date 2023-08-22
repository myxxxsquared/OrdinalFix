use super::Pointer;
use typed_arena::Arena;

pub struct RefArena<T> {
    arena: Arena<T>,
}

impl<T> RefArena<T> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    pub fn alloc<'a>(&'a self, value: T) -> Pointer<'a, T> {
        Pointer::new(self.arena.alloc(value))
    }

    pub fn alloc_extend<I>(&self, iterable: I) -> &[T]
    where
        I: IntoIterator<Item = T>,
    {
        self.arena.alloc_extend(iterable)
    }
}
