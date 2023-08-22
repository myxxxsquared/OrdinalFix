pub trait ValueEnum: Sized {
    const N: usize;
    fn value(&self) -> usize;
    fn from_value(value: usize) -> Option<Self>;
}
