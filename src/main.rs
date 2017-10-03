extern crate image;
extern crate threadpool;

mod stgr;

use std::path::Path;
use std::env::args;
use std::fs::File;
use std::io::{stdin, stdout, Write};

fn get_filename() -> String {
    let mut arguments = args();
    let _ = arguments.next();
    return arguments.next().unwrap();
}

fn main() {
    let filename = get_filename();
    println!("Reading {}...", filename);
    let filepath = Path::new(&filename);
    match image::open(&filepath) {
        Ok(mut img) => {
            match stgr::read_message(&img) {
                Ok(ustr) => println!("Current message: {}", ustr),
                Err(_) => println!("Error while processing unicode string"),
            }
            print!("Input new message: ");
            let _ = stdout().flush();
            let reader = stdin();
            let mut input = String::new();
            match reader.read_line(&mut input) {
                Ok(_) => {
                    let trim = input.trim();
                    if trim.len() == 0 {
                        println!("Nothing to write. Good bye!");
                        return;
                    }
                    stgr::write_message(&mut img, String::from(trim))
                },
                Err(_) => println!("Error while reading input"),
            }
            println!("Writing {}...", filename);
            let mut fout = File::create(&filepath).unwrap();
            let _ = img.save(&mut fout, image::BMP).unwrap();
            println!("Writing complete!");
        },
        Err(_) => println!("Error while reading file"),
    }
}
