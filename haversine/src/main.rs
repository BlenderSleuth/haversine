use std::{env, fs};

use anyhow::Result;

fn print_usage() {
    use std::path::Path;
    
    let exe_name =
        Path::new(&env::current_exe().unwrap())
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap().to_string();

    println!("Usage: {exe_name} [haversine_input.json]");
    println!("       {exe_name} [haversine_input.json] [haversine_answer.f64]");
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();

    let mut input_file = String::new();
    let mut answer_file: Option<String> = None;
    let mut invalid_args = 2 > args.len() || args.len() > 3;

    // Read in filenames
    for i in 1..args.len() {
        if invalid_args {
            break;
        }
        match i {
            1 => {
                input_file = args[i].clone();
            }
            2 => {
                answer_file = Some(args[i].clone());
            }
            _ => {
                invalid_args = true;
            }
        }
    }

    if invalid_args {
        print_usage();
        return Ok(());
    }
    
    let input = fs::read_to_string(input_file).expect("Unable to read input file.");
    let _answers = if let Some(answer_file) = answer_file {
        Some(fs::read(answer_file).expect("Unable to read answer file."))
    } else {
        None
    };

    println!("{input}");
    
    Ok(())
}
