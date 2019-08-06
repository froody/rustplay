use std::cmp::{min, Ordering};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};
mod sorted_iterator;

extern crate byteorder;
extern crate istring;
extern crate itertools;
extern crate rayon;

use byteorder::{LittleEndian, ReadBytesExt};
use istring::IString;
use rayon::prelude::*;
use sorted_iterator::Ext;

type FileList = Vec<Vec<u32>>;

#[derive(Default)]
struct Files {
    strings: Vec<IString>,
    strings_map: HashMap<IString, u32>,
    files: FileList,
    files_rev: FileList,
}

fn compare_file_entry_iters<'a>(
    mut a: impl Iterator<Item = &'a u32>,
    mut b: impl Iterator<Item = &'a u32>,
) -> Ordering {
    let mut mismatch = a.by_ref().zip(b.by_ref()).skip_while(|(x, y)| x == y);

    match mismatch.next() {
        Some((x, y)) => x.cmp(&y),
        None => a.count().cmp(&b.count()),
    }
}

fn compare_file_entry(a: &[u32], b: &[u32]) -> Ordering {
    compare_file_entry_iters(a.iter(), b.iter())
}

impl Files {
    fn new(
        strings: Vec<IString>,
        strings_map: HashMap<IString, u32>,
        files: FileList,
        files_rev: FileList,
    ) -> Files {
        eprintln!("loaded!");

        let mut files_rev = files_rev;
        let before = Instant::now();

        files_rev
            .par_chunks_mut(100)
            .for_each(|c: &mut [Vec<u32>]| {
                c.iter_mut().for_each(|f: &mut Vec<u32>| {
                    f.reverse();
                });
            });

        eprintln!("reversed! {}", before.elapsed().as_millis());
        let before = Instant::now();

        let mut files = files;

        files.par_sort_by(|a, b| compare_file_entry(&a[..], &b[..]));
        files_rev.par_sort_by(|a, b| compare_file_entry(&a[..], &b[..]));

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
    let mut files_rev: Vec<Vec<u32>> = vec![];
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
        files.push(file_bits.clone());
        files_rev.push(file_bits);
        if i % 1000000 == 0 {
            //let joined : String = itertools::join(tmpiter2, "!");
            //println!("path: {}, {:?}, {:?}", i, str::from_utf8(&buf).unwrap(), joined);
            println!("path: {}", i);
        }
    }

    return Ok(Files::new(strings, strings_map, files, files_rev));
}

fn load_file_list_from_binary<'a>(
    filenames_path: &String,
    files_path: &String,
) -> Result<Files, io::Error> {
    let before = Instant::now();
    let mut strings: Vec<IString> = vec![];
    let mut files: Vec<Vec<u32>> = vec![];
    let mut files_rev: Vec<Vec<u32>> = vec![];
    let strings_map = HashMap::new();

    // Indices start from 1, so add dummy item
    strings.push("".into());

    let filebuf = BufReader::with_capacity(1 << 27, File::open(filenames_path)?);
    for line in filebuf.lines() {
        strings.push(line?.into());
    }

    let mut filebuf = BufReader::new(File::open(files_path)?);

    let mut file_buf: Vec<u32> = vec![];
    loop {
        match filebuf.read_u32::<LittleEndian>() {
            Ok(0) => {
                files.push(file_buf.clone());
                files_rev.push(file_buf);
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
    eprintln!("loaded! {}", before.elapsed().as_millis());
    return Ok(Files::new(strings, strings_map, files, files_rev));
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

fn compare_adjacent<'a>(
    this: &'a Vec<u32>,
    next: &'a Vec<u32>,
    strings: &Files,
    stats: &mut Stats<'a>,
) {
    let mut last: String = strings.strings[this[0] as usize].clone().into();
    last = last.to_lowercase();
    if !(last.ends_with(".jpg") || last.ends_with(".jpeg")) {
        //continue;
    }
    let commonlen = common_prefix(this, next);
    if commonlen == 0 {
        return;
    }
    let entry = stats.counts.entry(commonlen).or_insert(0);
    *entry += 1;
    if commonlen > 1 {
        let key: (IString, &'a [u32], &'a [u32]) = (
            strings.strings[this[commonlen - 1] as usize].clone(),
            &this[commonlen - 1..],
            &next[commonlen - 1..],
        );
        let entry = stats.groups.entry(key).or_insert(0);
        *entry += 1;
    }
}
fn find_overlap(parent1: &[u32], parent2: &[u32], strings: &Files) -> (usize, usize, usize) {
    let mut parent1 = parent1.to_vec();
    parent1.reverse();
    let mut parent2 = parent2.to_vec();
    parent2.reverse();

    fn bounds_of_prefix(prefix: &Vec<u32>, haystack: &Vec<Vec<u32>>) -> (usize, usize) {
        let bottom = haystack
            .iter()
            .lower_bound_by(|val| compare_file_entry(&val, &prefix));
        let top = bottom
            + haystack.iter().skip(bottom).upper_bound_by(|val| {
                compare_file_entry(&val[0..min(val.len(), prefix.len())], &prefix)
            });
        (bottom, top)
    }
    //let before = Instant::now();
    let (left, left_top) = bounds_of_prefix(&parent1, &strings.files);
    let (right, right_top) = bounds_of_prefix(&parent2, &strings.files);
    //println!(
    //    "found after {}, {:?}",
    //    before.elapsed().as_nanos(),
    //    (left, left_top, right, right_top, strings.files.len())
    //);
    //println!(
    //    "parents {:?}",
    //    (
    //        decode(parent1.iter(), &strings.strings),
    //        decode(parent2.iter(), &strings.strings)
    //    )
    //);
    //println!(
    //    "found {}, {:#}, {:#}, {:#}, {:#}, {:#}, {:#}, {:#}",
    //    left,
    //    decode(strings.files[left].iter(), &strings.strings),
    //    decode(strings.files[left - 1].iter(), &strings.strings),
    //    decode(strings.files[left + 1].iter(), &strings.strings),
    //    decode(strings.files[left_top].iter(), &strings.strings),
    //    decode(strings.files[left_top - 1].iter(), &strings.strings),
    //    decode(strings.files[left_top - 2].iter(), &strings.strings),
    //    decode(
    //        strings.files[min(left_top + 1, strings.files.len())].iter(),
    //        &strings.strings
    //    )
    //);

    let mut i_l = strings.files[left..left_top].iter();
    let mut i_r = strings.files[right..right_top].iter();
    let mut common = 0;
    let mut left_only = 0;
    let mut right_only = 0;

    i_l.set_differences_by(
        &mut i_r,
        |a, b| compare_file_entry_iters(a.iter().skip(parent1.len()), b.iter().skip(parent2.len())),
        |_| left_only += 1,
        |_| right_only += 1,
        |_, _| common += 1,
    );
    (common, left_only, right_only)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let strings: Files = load_file(&args[1..]).unwrap();

    if cfg!(debug_assertions) {
        print_debug_info(&strings);
    }

    let before = Instant::now();
    let file_pairs: Vec<(&Vec<u32>, &Vec<u32>)> = strings
        .files_rev
        .iter()
        .zip(&strings.files_rev[1..])
        .collect();

    let stats_chunks: Vec<Stats> = file_pairs
        .par_chunks(100)
        .map(|chunk| -> Stats {
            let mut local_stats: Stats = Default::default();
            for (this, next) in chunk {
                compare_adjacent(this, next, &strings, &mut local_stats);
            }
            local_stats
        })
        .collect();
    eprintln!("mapped! {}", before.elapsed().as_millis());
    let before = Instant::now();
    let mut stats: Stats = Default::default();
    for chunk in stats_chunks {
        for (key, value) in chunk.counts {
            *stats.counts.entry(key).or_insert(0) += value;
        }
        for (key, value) in chunk.groups {
            *stats.groups.entry(key).or_insert(0) += value;
        }
    }
    eprintln!("reduced! {}", before.elapsed().as_millis());
    let before = Instant::now();

    for item in stats.counts.iter() {
        println!("hist is {:?}", item);
    }
    let mut ordered_groups: Vec<(usize, (IString, &[u32], &[u32]))> = stats
        .groups
        .iter()
        .map(|(key, val)| (*val, key.clone()))
        .collect();
    ordered_groups.par_sort();
    eprintln!("sorted groups! {}", before.elapsed().as_millis());
    let summaries = ordered_groups.iter().map(|(val, key)| {
        let (name, parent1, parent2) = key;
        let (common, left, right) = find_overlap(parent1, parent2, &strings);
        format!(
            "groups is {:?}",
            (
                &name,
                common,
                common as f64 / (common + left + right) as f64,
                decode(parent1.iter(), &strings.strings),
                decode(parent2.iter(), &strings.strings)
            ),
        )
    });
    let thing = itertools::join(summaries, "\n");
    eprintln!("made summaries! {}", before.elapsed().as_millis());

    io::stdout()
        .write_all(&thing.into_bytes())
        .expect("failed to write summaries");

    eprintln!("printed summaries! {}", before.elapsed().as_millis());
}
