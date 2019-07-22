use std::cmp::{min, Ordering};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};
mod sorted_iterator;

extern crate byteorder;
extern crate istring;
extern crate itertools;
extern crate superslice;

use byteorder::{LittleEndian, ReadBytesExt};
use istring::IString;
use superslice::Ext;

type FileList = Vec<Vec<u32>>;

#[derive(Default)]
struct Files {
    strings: Vec<IString>,
    strings_map: HashMap<IString, u32>,
    files: FileList,
    files_rev: FileList,
}

fn compare_file_entry_iters<'a, I, J>(mut a: I, mut b: J) -> Ordering
where
    I: Iterator<Item = &'a u32>,
    J: Iterator<Item = &'a u32>,
{
    let mut mismatch = a.by_ref().zip(b.by_ref()).skip_while(|(x, y)| x == y);
    match mismatch.next() {
        Some((x, y)) => x.cmp(&y),
        None => a.count().cmp(&b.count()),
    }
}

fn compare_file_entry(a: &Vec<u32>, b: &Vec<u32>) -> Ordering {
    let alen = a.len();
    let blen = b.len();
    for i in 0..min(alen, blen) {
        if a[i] == b[i] {
            continue;
        } else {
            return a[i].cmp(&b[i]);
        }
    }

    return alen.cmp(&blen);
}

fn sort_vec32(list: &mut FileList) {
    list.sort_by(compare_file_entry);
}

impl Files {
    fn new(strings: Vec<IString>, strings_map: HashMap<IString, u32>, files: FileList) -> Files {
        eprintln!("loaded!");

        for i in 1..10 {
            let mut skiplen = files.len();
            if i < 10 {
                skiplen /= 10;
                skiplen *= i;
            }
            let before = Instant::now();
            let mut val = files.iter().skip(skiplen);
            let after = before.elapsed().as_nanos();
            eprintln!("counted! {}, {:?}, {}", skiplen, val.next(), after);
            let before = Instant::now();
            let mut val = files.iter().rev().skip(skiplen);
            let after = before.elapsed().as_nanos();
            eprintln!("rcounted! {}, {:?}, {}", skiplen, val.next(), after);
        }

        let files_rev = files
            .iter()
            .map(|f| {
                let mut ret = f.clone();
                ret.reverse();
                ret
            })
            .collect();

        eprintln!("reversed!");
        let before = Instant::now();

        let slice = vec![files, files_rev];
        type JH = JoinHandle<FileList>;

        let threads: Vec<JH> = slice
            .into_iter()
            .map(move |mut arg| -> JH {
                spawn(move || -> FileList {
                    let tbef = Instant::now();
                    sort_vec32(&mut arg);
                    eprintln!("sorted a {}", tbef.elapsed().as_millis());
                    arg
                })
            })
            .collect();

        let mut result: Vec<FileList> = threads
            .into_iter()
            .map(move |t| t.join().unwrap())
            .collect();
        let files_rev = result.pop().unwrap();
        let files = result.pop().unwrap();
        // eprintln!("nothreads!");
        // let mut files = files;
        // let mut files_rev = files_rev;
        // sort_vec32(&mut files);
        // sort_vec32(&mut files_rev);
        //let files = th1.join().unwrap();
        //let files_rev = th2.join().unwrap();

        eprintln!("sorted! {}", before.elapsed().as_millis());

        Files {
            strings,
            strings_map,
            files,
            files_rev,
        }
    }
}

fn load_file_list_from_text<'a>(filename: &String) -> Result<Files, io::Error> {
    let file = File::open(filename)?;
    let mut filebuf = BufReader::new(file);
    let mut buf: Vec<u8> = Vec::new();
    let mut i = 0;
    let mut strings_map = HashMap::new();
    let mut strings: Vec<IString> = vec![];
    let mut files: Vec<Vec<u32>> = vec![];
    let mut next_string_index: u32 = 0;
    loop {
        buf.clear();
        if filebuf.read_until(b'\0', &mut buf)? == 0 {
            break;
        }
        buf.pop();
        let cursor = io::Cursor::new(&buf);
        let splits = cursor
            .split(b'/')
            .map(|s| IString::from_utf8(s.unwrap()).unwrap());
        i += 1;
        let file_bits: Vec<u32> = splits
            .map(|component| -> u32 {
                strings_map
                    .entry(component.clone())
                    .or_insert_with(|| {
                        strings.push(component.clone());
                        let tmp = next_string_index;
                        next_string_index += 1;
                        tmp
                    })
                    .clone()
            })
            .collect();
        files.push(file_bits);
        if i % 1000000 == 0 {
            //let joined : String = itertools::join(tmpiter2, "!");
            //println!("path: {}, {:?}, {:?}", i, str::from_utf8(&buf).unwrap(), joined);
            println!("path: {}", i);
        }
    }

    return Ok(Files::new(strings, strings_map, files));
}

fn load_file_list_from_binary<'a>(
    filenames_path: &String,
    files_path: &String,
) -> Result<Files, io::Error> {
    let mut strings: Vec<IString> = vec![];
    let mut files: Vec<Vec<u32>> = vec![];
    let strings_map = HashMap::new();

    // Indices start from 1, so add dummy item
    strings.push("".into());

    let filebuf = BufReader::new(File::open(filenames_path)?);
    for line in filebuf.lines() {
        strings.push(line?.into());
    }

    let mut filebuf = BufReader::new(File::open(files_path)?);

    let mut file_buf: Vec<u32> = vec![];
    loop {
        match filebuf.read_u32::<LittleEndian>() {
            Ok(0) => {
                files.push(file_buf);
                file_buf = vec![]
            }
            Ok(value) => {
                file_buf.push(value);
            }
            Err(_) => break,
        }
    }

    if cfg!(debug_assertions) {
        for string in strings.iter().take(10) {
            println!("item is {:?}", string);
        }
    }
    return Ok(Files::new(strings, strings_map, files));
}

fn common_prefix(a: &Vec<u32>, b: &Vec<u32>) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

fn decode<'a, I: Iterator<Item = &'a u32>>(item: I, strings: &Vec<IString>) -> String {
    itertools::join(
        item.map(|i| -> String { strings[*i as usize].clone().into() }),
        "/",
    )
}

fn load_file<'a>(args: &[String]) -> Result<Files, io::Error> {
    if args.len() == 1 {
        return load_file_list_from_text(&args[0]);
    } else {
        return load_file_list_from_binary(&args[0], &args[1]);
    }
}

fn print_debug_info(files: &Files) {
    for item in files.strings_map.iter().take(10) {
        println!("item is {:?}", item);
    }
    for item in files.files.iter().take(10) {
        let thing: Vec<IString> = item
            .iter()
            .map(|i| -> IString { files.strings[*i as usize].clone() })
            .collect();
        println!("item is {:?}, {:?}", item, thing);
    }

    println!(
        "Hello, world! {}, {}",
        files.strings.len(),
        files.files.len(),
    );
}

#[derive(Default)]
struct Stats<'a> {
    counts: HashMap<usize, u32>,
    groups: HashMap<(IString, &'a [u32], &'a [u32]), usize>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut stats: Stats = Default::default();

    let strings: Files = load_file(&args[1..]).unwrap();

    if cfg!(debug_assertions) {
        print_debug_info(&strings);
    }

    for (this, next) in strings.files_rev.iter().zip(&strings.files_rev[1..]) {
        let mut last: String = strings.strings[this[0] as usize].clone().into();
        last = last.to_lowercase();
        if !(last.ends_with(".jpg") || last.ends_with(".jpeg")) {
            //continue;
        }
        let commonlen = common_prefix(this, next);
        if commonlen == 0 {
            continue;
        }
        let entry = stats.counts.entry(commonlen).or_insert(0);
        *entry += 1;
        if commonlen > 1 {
            let key = (
                strings.strings[this[commonlen - 1] as usize].clone(),
                &this[commonlen-1..],
                &next[commonlen-1..],
            );
            let entry = stats.groups.entry(key.clone()).or_insert(0);
            *entry += 1;
        }
    }
    for item in stats.counts.iter() {
        println!("hist is {:?}", item);
    }
    let mut ordered_groups: Vec<(usize, (IString, &[u32], &[u32]))> = stats
        .groups
        .iter()
        .map(|(key, val)| (*val, key.clone()))
        .collect();
    ordered_groups.sort();
    for (val, key) in &ordered_groups {
        let (name, parent1, parent2) = key;
        println!(
            "groups is {:?}",
            (
                &name,
                val,
                decode(parent1.iter(), &strings.strings),
                decode(parent2.iter(), &strings.strings)
            ),
        );
    }

    let last = ordered_groups.last().unwrap();
    let (_, (name, parent1, parent2)) = last;
    let mut parent1 = parent1.to_vec();
    parent1.reverse();
    let mut parent2 = parent2.to_vec();
    parent2.reverse();

    let left = strings.files[..].lower_bound_by(|val| compare_file_entry(&val, &parent1));
    let right = strings.files[..].lower_bound_by(|val| compare_file_entry(&val, &parent2));
    let left_top = strings.files[..].upper_bound_by(|val| { let prefix = &val[0..min(val.len(), parent1.len())]; compare_file_entry_iters(prefix.iter(), parent1.iter()) } );
    let right_top = strings.files[..].upper_bound_by(|val| { let prefix = &val[0..min(val.len(), parent2.len())]; compare_file_entry_iters(prefix.iter(), parent2.iter()) } );
    println!("found {:?}", (left, left_top, right, right_top, strings.files.len()));
    println!("parents {:?}", ( decode(parent1.iter(), &strings.strings), decode(parent2.iter(), &strings.strings)));
    println!(
        "found {}, {:#}, {:#}, {:#}, {:#}, {:#}, {:#}, {:#}",
        left,
        decode(strings.files[left].iter(), &strings.strings),
        decode(strings.files[left - 1].iter(), &strings.strings),
        decode(strings.files[left + 1].iter(), &strings.strings),

        decode(strings.files[left_top].iter(), &strings.strings),
        decode(strings.files[left_top - 1].iter(), &strings.strings),
        decode(strings.files[left_top - 2].iter(), &strings.strings),
        decode(strings.files[min(left_top + 1, strings.files.len())].iter(), &strings.strings)
    );

    let mut i_l = strings.files[left..left_top].iter();
    let mut i_r = strings.files[right..right_top].iter();

    let mut common: u32 = 0;
    let mut left_only: u32 = 0;
    let mut right_only: u32 = 0;
    let mut l = i_l.next();
    let mut r = i_r.next();
    loop {
        if l.is_none() && r.is_none() {
            break;
        }
        if l.is_none() {
            while r.is_some() {
                right_only += 1;
                r = i_r.next();
            }
            break;
        }
        if r.is_none() {
            while l.is_some() {
                left_only += 1;
                l = i_l.next();
            }
            break;
        }
        let ll = l.unwrap();
        let rr = r.unwrap();
        match compare_file_entry_iters(ll.iter().skip(parent1.len()), rr.iter().skip(parent2.len())) {
            Ordering::Less => {
                left_only += 1;
                l = i_l.next();
            }
            Ordering::Greater => {
                right_only += 1;
                r = i_r.next();
            }
            Ordering::Equal => {
                common += 1;
                l = i_l.next();
                r = i_r.next();
            }
        }
    }
    println!("c: {} l: {} r: {}", common, left_only, right_only);
}
