use crate::props::Prop;
use std::{borrow::Borrow, fmt::Debug, hash::Hash, ops::Deref};

pub struct Pointer<'a, T: ?Sized> {
    ptr: &'a T,
}

impl<T: ?Sized + Debug> Debug for Pointer<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <T as Debug>::fmt(self.ptr, f)
    }
}

impl<T: ?Sized + Debug> Prop for Pointer<'_, T> {}

impl<'a, T: ?Sized> Pointer<'a, T> {
    pub fn ptr(&self) -> &'a T {
        self.ptr
    }
}

impl<'a, T: ?Sized> Clone for Pointer<'a, T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<'a, T: ?Sized> Copy for Pointer<'a, T> {}

impl<'a, T: ?Sized> Pointer<'a, T> {
    pub(super) fn new(ptr: &'a T) -> Self {
        Self { ptr }
    }

    pub fn from_ptr(ptr: &'a T) -> Self {
        Self { ptr }
    }
}

impl<'a, T: ?Sized> Hash for Pointer<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.ptr, state);
    }
}

impl<'a, T: ?Sized> PartialEq for Pointer<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr, other.ptr)
    }
}

impl<'a, T: ?Sized> Eq for Pointer<'a, T> {}

impl<'a, T: ?Sized> Deref for Pointer<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ptr
    }
}

impl<'a, T: ?Sized> Borrow<T> for Pointer<'a, T> {
    fn borrow(&self) -> &T {
        self.ptr
    }
}

fn cmp_ptr<T: ?Sized>(a: *const T, b: *const T) -> std::cmp::Ordering {
    if a < b {
        std::cmp::Ordering::Less
    } else if a > b {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

impl<'a, T: ?Sized> PartialOrd for Pointer<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T: ?Sized> Ord for Pointer<'a, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        cmp_ptr(self.ptr, other.ptr)
    }
}
