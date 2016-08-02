extern crate gcc;

use std::path::PathBuf;
use std::fs::{read_dir};


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
    if cfg!(target_os = "windows") {
	// These flags were discovered by opening the UDT vcproject file and viewing the 'Command Line' section of the Release Configuration
        cfg.flag("/GS")
            .flag("/analyze-")
            .flag("/Zc:wchar_t")
            .flag("/Zi")
            .flag("/Zc:inline")
            .flag("/fp:precise")
            .define("WIN32", None)
            .define("NDEBUG", None)
            .define("_CONSOLE", None)
            .define("UDT_EXPORTS", None)
            .define("_WINDLL", None)
            .flag("/errorReport:prompt")
            //.flag("/WX")
            .flag("/Zc:forScope")
            .flag("/Gd")
            .flag("/O2")  // optimize for speed
            .flag("/Ot")  // favor fast code
            .flag("/Ob2") // inline any suitable function
            .flag("/Oy")  // omit frame pointers
            .flag("/nologo") // suppress startup banner
            .flag("/W4")  // warning level
            .flag("/Gm-") // disable minimal rebuild
            .flag("/EHsc")// enable c++ exceptions
            .flag("/MD"); // multi-threaded DLL runtime
    } else {	
        cfg.flag("-fPIC")
           .opt_level(3)
           .flag("-Wextra")
           .flag("-Wall")
           .flag("-finline-functions")
           .flag("-fno-strict-aliasing")
           .flag("-fvisibility=hidden");
    }

    cfg.debug(false)
       .cpp(true)
       .compile("libudt4wrap.a");



}
