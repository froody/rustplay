use std::cmp::Ordering::{self, Less, Equal, Greater};

pub trait Ext {
    type Item;

    fn lower_bound_by<F>(&mut self, f: F) -> Self::Item
        where F : FnMut(&Self::Item) -> Ordering;

}

fn safe_count<I, J>(arg: &I) -> usize
    where I : Iterator<Item=J> + Clone
{
    let temp : I = (*arg).clone();
    temp.count()
}

impl<T, Thing> Ext for Thing
    where Thing: Iterator<Item=T> + Clone {
    type Item = T;
    fn lower_bound_by<Func>(&mut self, mut f: Func) -> Self::Item
        where Func: FnMut(&Self::Item) -> Ordering
    {
        let mut len: usize = self.clone().count();
        if len > 0 {
            len -= 1;
        }
        let mut offset : usize = 0;

        while len != 0 {
            let half : usize = len / 2;
            eprintln!("{}, {}, {}", offset, len, half);
            let value = self.clone().skip(offset + half).next();
            let uvalue =value.unwrap();
            match f(&uvalue) {
                Less => { eprintln!("Less!");
                    offset += half + 1; len -= half + 1; },
                _ => {eprintln!("Other!"); len = half;}
            }
            
        }
        self.clone().skip(offset).next().unwrap()
    }
}

#[test]
fn lower_bound() {
    let array = vec![1, 2, 3, 4, 5, 6];

    assert_eq!(3.cmp(&4), Less);
    assert_eq!(3.cmp(&3), Equal);
    assert_eq!(3.cmp(&2), Greater);

eprintln!("test1");
    assert_eq!(*array.iter().lower_bound_by(|x| x.cmp(&&3)), 3);
eprintln!("test2");
    assert_eq!(*array.iter().lower_bound_by(|x| {eprintln!("cmp {}", x); x.cmp(&&4)}), 4);
eprintln!("test3");
    assert_eq!(*array.iter().lower_bound_by(|x| x.cmp(&&5)), 5);
}