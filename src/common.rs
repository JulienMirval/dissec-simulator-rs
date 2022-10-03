pub type Address = usize;

pub trait Incrementable {
    fn increment(&self, offset: Option<usize>) -> Address;
}

impl Incrementable for Address {
    fn increment(&self, offset: Option<usize>) -> Address {
        self + offset.unwrap_or(1)
    }
}
