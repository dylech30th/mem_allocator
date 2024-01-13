use std::any::Any;

pub fn identity_once<T>(x: T) -> T {
    x
}

pub trait FuncExt : Sized {
    fn ignore(&self);

    fn apply<F: FnOnce(&mut Self)>(&mut self, f: F) -> &Self;
}

impl <T : Sized> FuncExt for T {
    fn ignore(&self) { }

    fn apply<F: FnOnce(&mut Self)>(&mut self, f: F) -> &Self {
        f(self);
        self
    }
}

pub trait OptionExt<T> {
    fn to_result<E, F: FnOnce() -> E>(self, error: F) -> Result<T, E>;

    fn combine<U>(self, rhs: Option<U>) -> Option<(T, U)>;

    fn flat_map_none<F: FnOnce() -> Option<T>>(self, f: F) -> Option<T>;
}

impl <T> OptionExt<T> for Option<T> {
    fn to_result<E, F: FnOnce() -> E>(self, error: F) -> Result<T, E> {
        match self {
            Some(x) => Ok(x),
            None => Err(error())
        }
    }

    fn combine<U>(self, rhs: Option<U>) -> Option<(T, U)> {
        self.and_then(|x| rhs.map(|y| (x, y)))
    }

    fn flat_map_none<F: FnOnce() -> Option<T>>(self, f: F) -> Option<T> {
        match self {
            Some(_) => self,
            None => f()
        }
    }
}