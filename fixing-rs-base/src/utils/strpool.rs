use super::{Pointer, RefArena};
use std::collections::HashMap;

pub struct StringPool<'a> {
    pool: HashMap<&'a str, StringRef<'a>>,
    arena: &'a RefArena<String>,
}

impl<'a> StringPool<'a> {
    pub fn new(arena: &'a RefArena<String>) -> Self {
        Self {
            pool: HashMap::new(),
            arena,
        }
    }

    pub fn get_or_add(&mut self, s: &str) -> StringRef<'a> {
        if let Some(r) = self.pool.get(s) {
            *r
        } else {
            let r = self.arena.alloc(s.to_string());
            self.pool.insert(r.ptr(), r);
            r
        }
    }

    pub fn get(&self, s: &str) -> Option<StringRef<'a>> {
        match self.pool.get(s) {
            Some(r) => Some(*r),
            None => {
                panic!("String not found: {}", s)
            }
        }
    }
}

pub type StringRef<'a> = Pointer<'a, String>;
