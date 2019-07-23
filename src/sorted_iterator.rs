use std::cmp::Ordering::{self, Equal, Greater, Less};

pub trait Ext {
    type Item;

    fn lower_bound_by<F>(&mut self, f: F) -> usize
    where
        F: FnMut(&Self::Item) -> Ordering;

    fn upper_bound_by<F>(&mut self, f: F) -> usize
    where
        F: FnMut(Self::Item) -> Ordering;

    fn set_differences_by<F, I1, I2,C>(&mut self, other: &mut Self, f: F, d1: I1, d2: I2, c: C)
    where
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
        I1: FnMut(&Self::Item),
        I2: FnMut(&Self::Item),
        C: FnMut(&Self::Item, &Self::Item);
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
            let value = self.clone().skip(offset + half).next();
            let uvalue = value.unwrap();
            match f(&uvalue) {
                Less => {
                    let delta = half + 1;
                    offset += delta;
                    len -= delta;
                }
                _ => {
                    len = half;
                }
            }
        }
        offset
    }

    fn upper_bound_by<Func>(&mut self, mut f: Func) -> usize
    where
        Func: FnMut(Self::Item) -> Ordering,
    {
        let mut len: usize = self.clone().count();
        let mut offset: usize = 0;

        while len != 0 {
            let half: usize = len / 2;
            let value = self.clone().skip(offset + half).next();
            let uvalue = value.unwrap();
            match f(uvalue) {
                Greater => {
                    len = half;
                }
                _ => {
                    let delta = half + 1;
                    offset += delta;
                    len -= delta;
                }
            }
        }
        offset
    }

    fn set_differences_by<F, I1, I2, C>(
        &mut self,
        other: &mut Self,
        mut f: F,
        mut d1: I1,
        mut d2: I2,
        mut c: C,
    ) where
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
        I1: FnMut(&Self::Item),
        I2: FnMut(&Self::Item),
        C: FnMut(&Self::Item, &Self::Item),
    {
        let mut l: Option<Self::Item> = self.next();
        let mut r: Option<Self::Item> = other.next();
        loop {
            if l.is_none() && r.is_none() {
                break;
            }
            if l.is_none() {
                while r.is_some() {
                    d2(&r.unwrap());
                    r = other.next();
                }
                break;
            }
            if r.is_none() {
                while l.is_some() {
                    d1(&l.unwrap());
                    l = self.next();
                }
                break;
            }
            let ll : &Self::Item = &*(&l).as_ref().unwrap();
            let rr : &Self::Item = &*(&r).as_ref().unwrap();
            match f(ll, rr) {
                Ordering::Less => {
                    d1(ll);
                    l = self.next();
                }
                Ordering::Greater => {
                    d2(rr);
                    r = other.next();
                }
                Ordering::Equal => {
                    c(ll, rr);
                    l = self.next();
                    r = other.next();
                }
            }
        }
    }
}

#[test]
fn lower_bound() {
    let array = vec![1, 2, 3, 3, 3, 4, 5, 6];

    assert_eq!(3.cmp(&4), Less);
    assert_eq!(3.cmp(&3), Equal);
    assert_eq!(3.cmp(&2), Greater);

    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&-1)), 0);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&0)), 0);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&1)), 0);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&2)), 1);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&3)), 2);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&4)), 5);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&5)), 6);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&6)), 7);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&7)), 8);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&8)), 8);
    assert_eq!(array.iter().lower_bound_by(|x| x.cmp(&&9)), 8);
}

#[test]
fn upper_bound() {
    let array = vec![1, 2, 3, 3, 3, 4, 5, 6];

    assert_eq!(3.cmp(&4), Less);
    assert_eq!(3.cmp(&3), Equal);
    assert_eq!(3.cmp(&2), Greater);

    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&-1)), 0);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&0)), 0);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&1)), 1);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&2)), 2);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&3)), 5);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&4)), 6);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&5)), 7);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&6)), 8);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&7)), 8);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&8)), 8);
    assert_eq!(array.iter().upper_bound_by(|x| x.cmp(&9)), 8);
    assert_eq!(array.iter().skip(5).next(), Some(&4));
}

#[test]
fn set_differences() {
    let array : Vec<u32> = vec![0, 1, 2, 3, 3, 3, 4, 5, 6];
    let array2 : Vec<u32> = vec![1, 2, 3, 4, 4, 4, 5, 6, 7];

    let mut common : Vec<u32> = Vec::new();
    let mut array1only : Vec<u32> = Vec::new();
    let mut array2only : Vec<u32> = Vec::new();

    array.iter().set_differences_by(
        &mut array2.iter(),
        |a: &&u32, b: &&u32| (**a).cmp(*b),
        |x: &&u32| array1only.push(**x),
        |x: &&u32| array2only.push(**x),
        |x: &&u32, y: &&u32| {
            assert_eq!(**x, **y);
            common.push(**x);
        },
    );
    assert_eq!(common, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(array1only, vec![0, 3, 3]);
    assert_eq!(array2only, vec![4, 4, 7]);
}
