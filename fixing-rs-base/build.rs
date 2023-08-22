extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .process_dir("grammars/")
        .unwrap();
}
