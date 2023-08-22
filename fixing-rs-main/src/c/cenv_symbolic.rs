use getset::CopyGetters;

#[derive(CopyGetters)]
pub struct CEnvSymbolic {
    #[get_copy = "pub"]
    num_functions: usize,

    #[get_copy = "pub"]
    num_identifiers: usize,

    #[get_copy = "pub"]
    max_number_args: usize,
}
