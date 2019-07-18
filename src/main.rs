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

struct Files {
    strings: Vec<IString>,
    strings_map: HashMap<IString, u32>,
    files: Vec<Vec<u32>>,
}

fn load_file_list_from_text(filename: &String) -> Result<Files, io::Error> {
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

    return Ok(Files {
        strings,
        strings_map,
        files,
    });
}

fn load_file_list_from_binary(
    filenames_path: &String,
    files_path: &String,
) -> Result<Files, io::Error> {
    let mut strings: Vec<IString> = vec![];
    let mut files: Vec<Vec<u32>> = vec![];
    let strings_map = HashMap::new();

    strings.push("".into());
    let mut filebuf = BufReader::new(File::open(filenames_path).unwrap());
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

    for string in strings.iter().take(10) {
        println!("item is {:?}", string);
    }
    return Ok(Files {
        strings,
        strings_map,
        files,
    });
}

fn common(a: &Vec<u32>, b: &Vec<u32>) -> usize {
    let mut total: usize = 0;
    let alen = a.len();
    let blen = b.len();
    for i in 0..min(alen, blen) {
        if a[i] == b[i] {
            total += 1;
        }
    }
    return total;
}

fn decode<'a,I: Iterator<Item=&'a u32>>(item: I, strings: &Vec<IString>) -> String {
    itertools::join(item.map(|i| -> String { strings[*i as usize].clone().into() }), "/")
        
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut strings;
    if args.len() == 2 {
        strings = load_file_list_from_text(&args[1]).unwrap();
    } else {
        strings = load_file_list_from_binary(&args[1], &args[2]).unwrap();
    }

    let names: Vec<&String> = vec![];

    //strings.files.iter().map(|
    for file in &mut strings.files {
        file.reverse();
    }
    println!("reversed!");
    strings.files.sort_by(|a: &Vec<u32>, b: &Vec<u32>| {
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
    println!("reversed!");

    let mut groups: HashMap<IString, usize> = HashMap::new();
    let mut groups2: HashMap<IString, HashSet<String>> = HashMap::new();

    println!(
        "Hello, world! {:?}, {}, {}",
        args[1],
        strings.strings.len(),
        strings.files.len(),
    );
    for item in strings.strings_map.iter().take(10) {
        println!("item is {:?}", item);
    }
    for item in strings.files.iter().take(10) {
        let thing: Vec<IString> = item
            .iter()
            .map(|i| -> IString { strings.strings[*i as usize].clone() })
            .collect();
        println!("item is {:?}, {:?}", item, thing);
    }

    let mut fiter = strings.files.iter().peekable();
    let mut counts: HashMap<usize, u32> = HashMap::new();
    while fiter.peek() != None {
        let this = fiter.next();
        let next = fiter.peek();
        if this != None && next != None {
            let this = this.unwrap();
            let next = next.unwrap();
            let mut last: String = strings.strings[this[0] as usize].clone().into();
            last = last.to_lowercase();
            if !(last.ends_with(".jpg") || last.ends_with(".jpeg")) {
                continue;
            }
            let commonlen = common(this, next);
            if commonlen == 0 {
                continue;
            }
            let entry = counts.entry(commonlen).or_insert(0);
            *entry += 1;
            if commonlen > 1 {
                let entry = groups
                    .entry(strings.strings[this[commonlen - 1] as usize].clone())
                    .or_insert(0);
                *entry += 1;
                let entry = groups2
                    .entry(strings.strings[this[commonlen - 1] as usize].clone())
                    .or_insert(HashSet::new());
                entry.insert(decode(this[commonlen..].iter(), &strings.strings));
                entry.insert(decode(next[commonlen..].iter(), &strings.strings));
            }

            if commonlen >= 23 {
                println!("longA {:?}", decode(this.iter(), &strings.strings));
                println!("longB {:?}", decode(next.iter(), &strings.strings));
            }
        }
    }
    for item in counts.iter() {
        println!("hist is {:?}", item);
    }
    let mut ordered_groups: Vec<(usize, IString)> = groups
        .iter()
        .map(|(key, val)| (*val, key.clone()))
        .collect();
    ordered_groups.sort();
    for (val, key) in ordered_groups {
        println!("groups is {:?}, {:?}", (&key, val), groups2[&key]);
    }
}
