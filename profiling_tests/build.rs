use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/*.asm");

    let out_dir = env::var("OUT_DIR").unwrap();
    
    Command::new("nasm")
        .arg("src/nop_loop.asm")
        .arg(format!("-o {out_dir}/nop_loop.obj").as_str())
        .arg("-f win64")
        .status().unwrap_or_else(|e| {
        println!("cargo:warning={e}");
        panic!("NASM build failed. Make sure you have nasm installed.\n\
        You can get NASM from https://nasm.us or your system's package manager.\n\nerror: {e}");
    });
    
    cc::Build::new().get_archiver()
        .arg(format!("{out_dir}/nop_loop.obj").as_str())
        .arg("-nologo")
        .status().unwrap_or_else(|e| {
        println!("cargo:warning={e}");
        panic!("lib command failed. Make sure you have msvc build tools installed.\n\nerror: {e}");
    });

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=nop_loop");
}