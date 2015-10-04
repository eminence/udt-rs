extern crate gcc;

use std::env;
use std::path::PathBuf;
use std::fs::{read_dir, PathExt};
use std::io;


fn main() {

    let udt4_src = PathBuf::from("libudt4/udt4/src");
    if let Err(_) = std::fs::metadata(&udt4_src) {
        panic!("Can't find UDT src dir: {:?}", udt4_src);
    }

    let mut cpp_files = Vec::new();
    // get list of .cpp files
    for dir in read_dir(&udt4_src).unwrap() {
        let path = dir.unwrap().path();
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
    if cfg!(target_os = "macox") {
        cfg.define("osx", None);
    }
    if cfg!(target_os = "unix") {
        cfg.define("LINUX", None)
           .define("AMD64", None);
    }
    cfg.debug(false)
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
