use std::fmt::Debug;

pub trait Buildable {
    type Builder: Builder;

    fn builder() -> Self::Builder;
}

pub trait Builder {
    type Target: Buildable + Debug;

    fn build(&mut self) -> Self::Target;
}
