use std::env;
use std::path::PathBuf;
use std::process::Command;

use glob::glob;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir.clone());
    
    // Assemble all asm files in src directory
    for path in glob("src/**/*.asm").expect("Failed to read asm glob pattern") {
        let asm_path = path.unwrap();
        let asm_path_str = asm_path.to_str().unwrap();
        
        let obj_path = out_path.join(asm_path.file_name().unwrap()).with_extension("obj");
        let obj_path_str = obj_path.to_str().unwrap();
        
        println!("cargo:rerun-if-changed={asm_path_str}");
        
        Command::new("nasm")
            .arg(asm_path_str)
            .arg(format!("-o {obj_path_str}").as_str())
            .arg("-f win64")
            .status().unwrap_or_else(|e| {
            println!("cargo:warning={e}");
            panic!("NASM build failed. Make sure you have nasm installed.\n\
            You can get NASM from https://nasm.us or your system's package manager.\n\nerror: {e}");
        });

        cc::Build::new().get_archiver()
            .arg(obj_path_str)
            .arg("-nologo")
            .status().unwrap_or_else(|e| {
            println!("cargo:warning={e}");
            panic!("lib command failed. Make sure you have MSVC build tools installed.\n\nerror: {e}");
        });
        
        println!("cargo:rustc-link-lib=static={}", asm_path.with_extension("").file_name().unwrap().to_str().unwrap());
    }

    println!("cargo:rustc-link-search=native={out_dir}");
}