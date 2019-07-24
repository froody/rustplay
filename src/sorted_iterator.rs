use std::cmp::Ordering::{self, Equal, Greater, Less};
use std::collections::VecDeque;

pub trait Ext {
    type Item;

    fn lower_bound_by<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&Self::Item) -> Ordering;

    fn upper_bound_by<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&Self::Item) -> Ordering;

    fn binary_search_by<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&Self::Item) -> Ordering;

    fn set_differences_by<F, I1, I2, C>(&mut self, other: &mut Self, f: F, d1: I1, d2: I2, c: C)
    where
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
        I1: FnMut(Self::Item),
        I2: FnMut(Self::Item),
        C: FnMut(Self::Item, Self::Item);
}

impl<T, Thing> Ext for Thing
where
    Thing: Iterator<Item = T> + Clone,
{
    type Item = T;
    fn lower_bound_by<Func>(&mut self, mut f: Func) -> usize
    where
        Func: FnMut(&Self::Item) -> Ordering,
    {
        let mut len: usize = self.clone().count();
        let mut offset: usize = 0;

        while len != 0 {
            let half: usize = len / 2;
            let value = self.clone().skip(offset + half).take(1);
            value.for_each(|uvalue| match f(&uvalue) {
                Less => {
                    let delta = half + 1;
                    offset += delta;
                    len -= delta;
                }
                _ => {
                    len = half;
                }
            });
        }
        offset
    }

    fn upper_bound_by<Func>(&mut self, mut f: Func) -> usize
    where
        Func: FnMut(&Self::Item) -> Ordering,
    {
        let mut len: usize = self.clone().count();
        let mut offset: usize = 0;

        while len != 0 {
            let half: usize = len / 2;
            let value = self.clone().skip(offset + half).take(1);
            value.for_each(|uvalue| match f(&uvalue) {
                Greater => {
                    len = half;
                }
                _ => {
                    let delta = half + 1;
                    offset += delta;
                    len -= delta;
                }
            });
        }
        offset
    }

    fn binary_search_by<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&Self::Item) -> Ordering,
    {
        0
    }

    fn set_differences_by<F, I1, I2, C>(
        &mut self,
        other: &mut Self,
        mut compare: F,
        mut found_a: I1,
        mut found_b: I2,
        mut found_ab: C,
    ) where
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
        I1: FnMut(Self::Item),
        I2: FnMut(Self::Item),
        C: FnMut(Self::Item, Self::Item),
    {
        let mut ss = self.peekable();
        let mut oo = other.peekable();
        loop {
            match (ss.peek(), oo.peek()) {
                (None, None) => break,
                (Some(_), None) => {
                    ss.for_each(|xx| found_a(xx));
                    break;
                }
                (None, Some(_)) => {
                    oo.for_each(|yy| found_b(yy));
                    break;
                }
                (Some(x), Some(y)) => match compare(x, y) {
                    Less => {
                        ss.by_ref().take(1).for_each(|x| found_a(x));
                    }
                    Greater => {
                        oo.by_ref().take(1).for_each(|x| found_b(x));
                    }
                    Equal => {
                        ss.by_ref()
                            .take(1)
                            .zip(oo.by_ref().take(1))
                            .for_each(|(x, y)| found_ab(x, y));
                    }
                },
            }
        }
    }
}

#[test]
fn lower_bound() {
    let array = vec![1, 2, 3, 3, 3, 4, 5, 6];
    let positions = vec![
        (-1, 0),
        (0, 0),
        (1, 0),
        (2, 1),
        (3, 2),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 8),
        (8, 8),
        (9, 8),
    ];

    for (value, position) in positions {
        assert_eq!(array.iter().lower_bound_by(|x| (*x).cmp(&value)), position);
        assert_eq!(
            array
                .clone()
                .into_iter()
                .lower_bound_by(|x| (x).cmp(&value)),
            position
        );
    }
}

#[test]
fn upper_bound() {
    let array = vec![1, 2, 3, 3, 3, 4, 5, 6];
    let positions = vec![
        (-1, 0),
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 5),
        (4, 6),
        (5, 7),
        (6, 8),
        (7, 8),
        (8, 8),
        (9, 8),
    ];

    for (value, position) in positions {
        assert_eq!(array.iter().upper_bound_by(|x| (*x).cmp(&value)), position);
        assert_eq!(
            array
                .clone()
                .into_iter()
                .upper_bound_by(|x| (x).cmp(&value)),
            position
        );
    }
}

#[test]
fn set_differences() {
    let array: Vec<u32> = vec![0, 1, 2, 3, 3, 3, 4, 5, 6];
    let array2: Vec<u32> = vec![1, 2, 3, 4, 4, 4, 5, 6, 7];

    let mut common: Vec<u32> = Vec::new();
    let mut array1only: Vec<u32> = Vec::new();
    let mut array2only: Vec<u32> = Vec::new();

    array.iter().set_differences_by(
        &mut array2.iter(),
        |a: &&u32, b: &&u32| (*a).cmp(*b),
        |x: &u32| array1only.push(*x),
        |x: &u32| array2only.push(*x),
        |x: &u32, y: &u32| {
            assert_eq!(*x, *y);
            common.push(*x);
        },
    );
    assert_eq!(common, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(array1only, vec![0, 3, 3]);
    assert_eq!(array2only, vec![4, 4, 7]);

    common.clear();
    array1only.clear();
    array2only.clear();

    array.into_iter().set_differences_by(
        &mut array2.into_iter(),
        |a: &u32, b: &u32| a.cmp(b),
        |x: u32| array1only.push(x),
        |x: u32| array2only.push(x),
        |x: u32, y: u32| {
            assert_eq!(x, y);
            common.push(x);
        },
    );

    assert_eq!(common, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(array1only, vec![0, 3, 3]);
    assert_eq!(array2only, vec![4, 4, 7]);
}
