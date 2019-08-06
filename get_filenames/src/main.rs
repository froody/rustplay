extern crate byteorder;
extern crate istring;
extern crate walkdir;

use byteorder::{LittleEndian, WriteBytesExt};
use istring::IString;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use walkdir::WalkDir;

fn main() {
    let mut strings: Vec<IString> = Vec::new();
    let mut stringsMap: HashMap<IString, u32> = HashMap::new();

    let args: Vec<String> = env::args().collect();
    let mut dirs: usize = 0;
    let mut next_string_index: u32 = 0;
    let mut file = BufWriter::new(File::create(&args[2]).unwrap());

    for entry in WalkDir::new(&args[1]) {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }

        let indices: Vec<u32> = entry
            .path()
            .components()
            .map(|component| -> u32 {
                let scomp: String = component.as_os_str().to_string_lossy().into();
                let istr: IString = scomp.into();
                stringsMap
                    .entry(istr.clone())
                    .or_insert_with(|| {
                        strings.push(istr.clone());
                        next_string_index += 1;
                        next_string_index
                    })
                    .clone()
            })
            .collect();

                let mut wtr = vec![];
        for index in indices {
            wtr.write_u32::<LittleEndian>(index).unwrap();
        }
        wtr.write_u32::<LittleEndian>(0).unwrap();
        file.write_all(&wtr);

        dirs += 1;
    }

    let mut stringfile = BufWriter::new(File::create(&args[3]).unwrap());
    for astring in strings {
        stringfile.write(astring.as_bytes());
        stringfile.write(b"\n");
    }

    println!("Hello, world! {}", dirs);
}
