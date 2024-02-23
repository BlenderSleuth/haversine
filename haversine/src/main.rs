mod parser;

use std::mem::size_of;
use std::path::Path;
use std::{env, fs};

use parser::parse_pairs;
use haversine::profile::{print_time_records, time_block};
use haversine::timer::{estimate_cpu_frequency, read_cpu_timer};

#[derive(Default, Copy, Clone)]
struct Pair {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

fn main() -> std::io::Result<()> {
    let prof_begin = read_cpu_timer();

    let args = env::args().collect::<Vec<String>>();

    // 2 or 3 arguments required
    if !(2..=3).contains(&args.len()) {
        let exe_name = Path::new(&env::current_exe().unwrap())
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .to_string();

        println!("Usage: {exe_name} [haversine_input.json]");
        println!("       {exe_name} [haversine_input.json] [haversine_answer.f64]");

        return Ok(());
    }

    let input_file = args[1].as_str();
    let answer_file = args.get(2).map(|x| x.as_str());

    let input= {
        time_block!("Read", 0);
        fs::read_to_string(input_file)?
    };

    let pairs = parse_pairs(&input).expect("ERROR: Malformed input JSON.");

    let mut distance_sum = 0.0;
    {
        time_block!("Sum", 2);
        let sum_coef = 1.0 / pairs.len() as f64;
        for Pair { x0, y0, x1, y1 } in pairs.iter().cloned() {
            time_block!("SingleSum", 3);
            let earth_radius = 6372.8;
            let distance = haversine::reference_haversine(x0, y0, x1, y1, earth_radius);
            distance_sum += distance * sum_coef;
        }
    }
    
    {
        time_block!("MiscOutput", 4);

        println!("Input size: {}", input.len());
        println!("Pair count: {}", pairs.len());
        println!("Haversine sum: {distance_sum:.16}");

        let answers = if let Some(file) = answer_file {
            Some(fs::read(file)?)
        } else {
            None
        };

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
    }
    
    let prof_end = read_cpu_timer();
    let freq = estimate_cpu_frequency(1000);

    let program_time = prof_end - prof_begin;
    let program_time_ms = program_time as f64 * 1000.0 / freq as f64;
    println!("Total time: {program_time_ms:.4}ms (CPU freq {freq}Hz)");

    print_time_records(program_time);

    println!();

    Ok(())
}
