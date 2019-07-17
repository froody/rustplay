use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

extern crate istring;
extern crate itertools;

use istring::IString;

struct Files {
    strings: HashMap<IString, u32>,
    files: Vec<Vec<u32>>,
}

fn load_file_list(filename: &String) -> Result<Files, io::Error> {
    let file = File::open(filename)?;
    let mut filebuf = BufReader::new(file);
    let mut buf: Vec<u8> = Vec::new();
    let mut i = 0;
    let mut strings = HashMap::new();
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
                strings
                    .entry(component.clone())
                    .or_insert_with(|| {
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

    return Ok(Files { strings, files });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let strings = load_file_list(&args[1]).unwrap();

    println!(
        "Hello, world! {:?}, {}, {}",
        args[1],
        strings.strings.len(),
        strings.files.len(),
    );
    for item in strings.strings.iter().take(10) {
        println!("item is {:?}", item);
    }
    for item in strings.files.iter().take(10) {
        println!("item is {:?}", item);
    }
}
