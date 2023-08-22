use std::{
    borrow::{Borrow, BorrowMut},
    cell::UnsafeCell,
    marker::PhantomData,
};

pub struct RefCellFrom<T, F> {
    value: UnsafeCell<T>,
    _p: PhantomData<F>,
}

impl<T, F> RefCellFrom<T, F> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            _p: PhantomData,
        }
    }

    pub fn borrow<'a, 'b, 'c, B>(&'a self, b: &'b B) -> &'c T
    where
        B: Borrow<F>,
        'a: 'c,
        'b: 'c,
    {
        let _: &F = b.borrow();
        unsafe { &*self.value.get() }
    }

    pub fn borrow_mut<'a, 'b, 'c, B>(&'a self, b: &'b mut B) -> &'c mut T
    where
        B: BorrowMut<F>,
        'a: 'c,
        'b: 'c,
    {
        let _: &mut F = b.borrow_mut();
        unsafe { &mut *self.value.get() }
    }
}
