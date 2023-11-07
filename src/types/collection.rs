use std::ops::{Add, AddAssign};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Collection<T> {
    #[default]
    None,
    One(T),
    Multiple(Vec<T>),
}

impl<T> Collection<T> {
    pub fn push(self, t: T) -> Self {
        match self {
            Collection::None => Collection::One(t),
            Collection::One(t2) => Collection::Multiple(vec![t, t2]),
            Collection::Multiple(mut arr) => {
                arr.push(t);
                Collection::Multiple(arr)
            }
        }
    }
}

impl<T> Add for Collection<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Collection::None, rhs) => rhs,
            (lhs, Collection::None) => lhs,
            (Collection::One(elem), rhs) => rhs.push(elem),
            (lhs, Collection::One(elem)) => lhs.push(elem),
            (Collection::Multiple(mut lhs), Collection::Multiple(mut rhs)) => {
                lhs.append(&mut rhs);
                Collection::Multiple(lhs)
            }
        }
    }
}

impl<T> AddAssign for Collection<T> {
    fn add_assign(&mut self, other: Self) {
        let this = std::mem::take(self);
        *self = this + other;
    }
}

impl<T> From<Vec<T>> for Collection<T> {
    fn from(arr: Vec<T>) -> Self {
        match arr.len() {
            0 => Collection::None,
            1 => Collection::One(arr.into_iter().next().unwrap()),
            _ => Collection::Multiple(arr),
        }
    }
}

impl<T> From<Option<Vec<T>>> for Collection<T> {
    fn from(opt: Option<Vec<T>>) -> Self {
        match opt {
            None => Collection::None,
            Some(arr) => match arr.len() {
                0 => Collection::None,
                1 => Collection::One(arr.into_iter().next().unwrap()),
                _ => Collection::Multiple(arr),
            },
        }
    }
}

impl<T> From<Collection<T>> for Vec<T> {
    fn from(coll: Collection<T>) -> Vec<T> {
        match coll {
            Collection::None => Vec::with_capacity(0),
            Collection::One(t) => vec![t],
            Collection::Multiple(arr) => arr,
        }
    }
}
