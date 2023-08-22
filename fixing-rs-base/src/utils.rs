mod arena;
mod pointer;
mod provide_arena;
mod queue;
mod refcell;
mod strpool;
mod symtab;
mod value_enum;

pub use arena::RefArena;
pub use pointer::Pointer;
pub use provide_arena::ProvideArena;
pub use queue::{Queue, QueueItem, QueueItemIndex};
pub use refcell::RefCellFrom;
pub use strpool::{StringPool, StringRef};
pub use symtab::{SymTab, SymTabContent};
pub use value_enum::ValueEnum;

pub fn slice_check_len<T>(slice: &[T], len: usize, msg: &str) {
    if slice.len() != len {
        panic!(
            "Slice length mismatch: expected {}, got {} ({}).",
            len,
            slice.len(),
            msg
        );
    }
}
