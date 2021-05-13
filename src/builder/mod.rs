use std::fmt::Debug;

pub trait Buildable {
    type Builder: Builder<Target = Self>;

    fn builder() -> Self::Builder;
}

pub trait Builder {
    type Target: Debug;

    fn build(&mut self) -> Self::Target;
}
