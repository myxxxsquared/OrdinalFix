#![cfg_attr(nightly, feature(proc_macro_diagnostic))]

use proc_macro::TokenStream;

mod prop;
mod semantic;
mod semantic_symbolic;
mod syntactic;
mod union_entity;
mod utils;
mod value_enum;

#[proc_macro_derive(Prop)]
pub fn derive_prop(input: TokenStream) -> TokenStream {
    prop::derive_prop_inner(input)
}

#[proc_macro_derive(ValueEnum)]
pub fn derive_value_enum(input: TokenStream) -> TokenStream {
    match value_enum::derive_value_enum_inner(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn impl_syntactic_processor(args: TokenStream, item: TokenStream) -> TokenStream {
    match syntactic::impl_syntactic_processor_inner(args, item) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn impl_semantic_processor(args: TokenStream, item: TokenStream) -> TokenStream {
    match semantic::impl_semantic_processor_inner(args, item) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn impl_semantic_symbolic_processor(args: TokenStream, item: TokenStream) -> TokenStream {
    match semantic_symbolic::impl_semantic_symbolic_processor_inner(args, item) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn create_union_entity(args: TokenStream, item: TokenStream) -> TokenStream {
    match union_entity::union_entity_inner(args, item) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
