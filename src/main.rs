use std::env;
use std::io::{self, BufReader, BufRead};
use std::fs::File;
use std::str;
extern crate itertools;
fn main() {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1]).unwrap();
    let mut filebuf = BufReader::new(file);
    let mut buf : Vec<u8> = Vec::new();
    let mut i = 0;
    loop {
        buf.clear();
        let _size = filebuf.read_until(b'\0', &mut buf).unwrap();
        if _size == 0 {
            break;
        }
        buf.pop();
        let cursor = io::Cursor::new(&buf);
        let splits = cursor.split(b'/');
        i+= 1;
        if i % 100000 == 0 {
            let tmpiter : Vec<Vec<u8>> = splits.into_iter().map(|s| s.unwrap()).collect();
            let tmpiter2 = tmpiter.iter().map(|s| str::from_utf8(&s).unwrap() );
            let joined : String = itertools::join(tmpiter2, "!");
            println!("path: {}, {:?}, {:?}", i, str::from_utf8(&buf).unwrap(), joined);
        }
    }

    println!("Hello, world! {:?}, {}", args[1], i);
}    
