use std::any::Any;

pub fn identity_once<T>(x: T) -> T {
    x
}

pub trait FuncExt : Sized {
    fn ignore(&self);

    fn apply<U, F: FnOnce(&mut Self)>(&mut self, f: F) -> &Self;
}

impl <T : Sized> FuncExt for T {
    fn ignore(&self) { }

    fn apply<U, F: FnOnce(&mut Self)>(&mut self, f: F) -> &Self {
        f(self);
        self
    }
}

pub trait OptionExt<T> {
    fn to_result<E, F: FnOnce() -> E>(self, error: F) -> Result<T, E>;
}

impl <T> OptionExt<T> for Option<T> {
    fn to_result<E, F: FnOnce() -> E>(self, error: F) -> Result<T, E> {
        match self {
            Some(x) => Ok(x),
            None => Err(error())
        }
    }
}