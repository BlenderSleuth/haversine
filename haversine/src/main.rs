mod parser;

use std::mem::size_of;
use std::path::Path;
use std::{env, fs};

use parser::parse_pairs;

#[derive(Default, Copy, Clone)]
struct Pair {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

fn main() -> std::io::Result<()> {
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
        let exe_name = Path::new(&env::current_exe().unwrap())
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .to_string();

        println!("Usage: {exe_name} [haversine_input.json]");
        println!("       {exe_name} [haversine_input.json] [haversine_answer.f64]");

        return Ok(());
    }

    let input = fs::read_to_string(input_file)?;
    let answers = if let Some(file) = answer_file {
        Some(fs::read(file)?)
    } else {
        None
    };

    let pairs = parse_pairs(&input).expect("ERROR: Malformed input JSON.");

    let mut distance_sum = 0.0;
    let sum_coef = 1.0 / pairs.len() as f64;
    for Pair { x0, y0, x1, y1 } in pairs.iter().cloned() {
        let earth_radius = 6372.8;
        let distance = haversine::reference_haversine(x0, y0, x1, y1, earth_radius);
        distance_sum += distance * sum_coef;
    }

    println!("Input size: {}", input.len());
    println!("Pair count: {}", pairs.len());
    println!("Haversine sum: {distance_sum:.16}");

    if let Some(answers) = answers {
        println!();
        println!("Validation:");

        let ref_sum_idx = answers.len() - size_of::<f64>();
        let num_answers = (ref_sum_idx) / size_of::<f64>();

        if num_answers != pairs.len() {
            println!("FAILED - pair count doesn't match {num_answers}.");
        }
        let reference_sum = f64::from_be_bytes(answers[ref_sum_idx..].try_into().unwrap());

        println!("Reference sum: {reference_sum:.16}");
        println!("Difference: {:.16}", distance_sum - reference_sum);
        println!();
    }

    Ok(())
}
