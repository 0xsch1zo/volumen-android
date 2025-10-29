pub enum TakenElements<T> {
    NotEnough,
    Exact(T),
    TooMuch(T),
}

impl<T> TakenElements<T> {
    pub fn exact_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Self::Exact(t) => Ok(t),
            _ => Err(err),
        }
    }

    pub fn enough_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Self::NotEnough => Err(err),
            Self::Exact(t) => Ok(t),
            Self::TooMuch(t) => Ok(t),
        }
    }

    pub fn inspect_surplus(self, f: impl FnOnce(&T) -> ()) -> Self {
        match &self {
            Self::TooMuch(t) => f(t),
            _ => {}
        }
        self
    }
}

pub trait TakeExactlyExt: Iterator {
    fn take_exactly<T: FromIterator<Self::Item>>(self, n: usize) -> TakenElements<T>;
}

impl<I: Iterator> TakeExactlyExt for I {
    fn take_exactly<T: FromIterator<Self::Item>>(mut self, n: usize) -> TakenElements<T> {
        let Some(collection) = (0..n)
            .into_iter()
            .map(|_| self.next())
            .collect::<Option<T>>()
        else {
            return TakenElements::NotEnough;
        };

        if self.next().is_some() {
            TakenElements::TooMuch(collection)
        } else {
            TakenElements::Exact(collection)
        }
    }
}
