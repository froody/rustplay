use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

extern crate byteorder;
extern crate istring;
extern crate itertools;

use byteorder::{LittleEndian, ReadBytesExt};
use istring::IString;

#[derive(Default)]
struct Files {
    strings: Vec<IString>,
    strings_map: HashMap<IString, u32>,
    files: Vec<Vec<u32>>,
    files_rev: Vec<Vec<u32>>,
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

    let files_rev = Vec::new();
    return Ok(Files {
        strings,
        strings_map,
        files,
        files_rev,
    });
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

    let filebuf = BufReader::new(File::open(filenames_path).unwrap());
    for line in filebuf.lines() {
        strings.push(line?.into());
    }

    let mut filebuf = BufReader::new(File::open(files_path).unwrap());

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
    let files_rev = Vec::new();
    return Ok(Files {
        strings,
        strings_map,
        files,
        files_rev
    });
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
    let mut strings;
    if args.len() == 1 {
        strings = load_file_list_from_text(&args[0])?;
    } else {
        strings = load_file_list_from_binary(&args[0], &args[1])?;
    }

    //strings.files.iter().map(|
    strings.files_rev  = strings.files.iter().map(|f| { let mut ret = f.clone(); ret.reverse(); ret }).collect(); 

    println!("reversed!");
    for list in &mut [&mut strings.files, &mut strings.files_rev] {
        list.sort_by(|a: &Vec<u32>, b: &Vec<u32>| {
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
        });
    }
    println!("sorted!");

    Ok(strings)
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
    groups2: HashMap<(IString, &'a [u32], &'a [u32]), HashSet<String>>,
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
                let key = (strings.strings[this[commonlen - 1] as usize].clone(),
                        &this[commonlen..],
                        &next[commonlen..]);
                let entry = stats
                    .groups
                    .entry(key.clone())
                    .or_insert(0);
                *entry += 1;
                let entry = stats
                    .groups2
                    .entry(key)
                    .or_insert(HashSet::new());
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
    for (val, key) in ordered_groups {
        let mut group_names: Vec<&String> = stats.groups2[&key].iter().collect();
        group_names.sort();
        let (name, parent1, parent2) = key;
        println!("groups is {:?}, {:?}", (&name, val, decode(parent1.iter(), &strings.strings), decode(parent2.iter(), &strings.strings)), group_names);
    }
}
