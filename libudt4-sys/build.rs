#![feature(path_ext)]
#![feature(result_expect)]

extern crate gcc;

use std::env;
use std::path::PathBuf;
use std::fs::{read_dir, PathExt};
use std::io;

fn main() {

    let udt4_src = PathBuf::from("libudt4/udt4/src");
    if !udt4_src.exists() {
        panic!("Can't find UDT src dir: {:?}", udt4_src);
    }

    let mut cpp_files = Vec::new();
    // get list of .cpp files
    for dir in read_dir(&udt4_src).expect("Failed to read udt4 src dir") {
        let path = dir.expect("Failed to get path").path();
        if let Some(ext) = path.extension() {
            if ext == "cpp" {
                cpp_files.push(path.clone());
            }
        }
    }

    println!("{:?}", cpp_files);
    // g++ -fPIC -Wall -Wextra -DLINUX -finline-functions -O3 -fno-strict-aliasing -fvisibility=hidden -DAMD64 ccc.cpp -c
    let mut cfg = gcc::Config::new();
    for file in cpp_files {
        cfg.file(file);
    }
    cfg.include(&udt4_src);
    cfg.file("udt_wrap.cpp");
    cfg.define("LINUX", None)
       .define("AMD64", None)
       .debug(false)
       .cpp(true)
       .opt_level(3)
       .flag("-fPIC")
       .flag("-Wextra")
       .flag("-Wall")
       .flag("-finline-functions")
       .flag("-fno-strict-aliasing")
       .flag("-fvisibility=hidden")
       .compile("libudt4wrap.a");



}
