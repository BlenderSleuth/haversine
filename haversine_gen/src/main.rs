mod rand;

use std::{env, fs};
use std::mem::size_of;

// NOTE(casey): earth_radius is generally expected to be 6372.8
fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64, earth_radius: f64) -> f64
{
    /* NOTE(casey): This is not meant to be a "good" way to calculate the Haversine distance.
       Instead, it attempts to follow, as closely as possible, the formula used in the real-world
       question on which these homework exercises are loosely based.
    */

    let lat1 = y0;
    let lat2 = y1;
    let lon1 = x0;
    let lon2 = x1;

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    earth_radius * c
}

fn print_usage() {
    use std::path::Path;

    let exe_name =
        Path::new(&env::current_exe().unwrap())
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap().to_string();

    println!("Usage: {exe_name} [uniform/cluster] [random seed] [number of coordinate pairs to generate]");
}

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<String>>();
    
    if args.len() != 4 {
        print_usage();
        return Ok(());
    }

    let mut cluster_count_left = u64::MAX;
    let max_allowed_x = 180.0;
    let max_allowed_y = 90.0;

    let mut x_center = 0.0;
    let mut y_center = 0.0;
    let mut x_radius = max_allowed_x;
    let mut y_radius = max_allowed_y;

    let distribution = args[1].as_str();
    
    if distribution == "cluster" {
        cluster_count_left = 0;
    } else if distribution != "uniform" {
        println!("WARNING: Unrecognized method name. Using 'uniform'.");
    }
    
    let random_seed = if let Ok(parsed_value) = args[2].parse::<u64>() {
        parsed_value
    } else {
        print_usage();
        return Ok(());
    };
    
    let mut random_series = rand::RandomSeries::seed(random_seed);
    
    let num_pairs = if let Ok(parsed_value) = args[3].parse::<usize>() {
        parsed_value
    } else {
        print_usage();
        return Ok(());
    };
    
    let max_pairs_exp = 34;
    let max_pairs = 1usize << max_pairs_exp;
    if num_pairs > max_pairs {
        println!("Maximum number of pairs is 2^{max_pairs_exp} ({max_pairs}).");
        return Ok(());
    }
    
    let mut sum = 0.0;
    let sum_coef = 1.0 / num_pairs as f64;
    let cluster_count_max = 1 + (num_pairs as u64 / 64);
    
    let mut data_str = String::with_capacity(15 + num_pairs*100);
    data_str += "{\"pairs\": [\n";
    let mut answers = Vec::<u8>::with_capacity((num_pairs+1) * size_of::<f64>());
    
    for i in 0..num_pairs {
        if cluster_count_left == 0 {
            cluster_count_left = cluster_count_max;
            x_center = random_series.random_in_range(-max_allowed_x, max_allowed_x);
            y_center = random_series.random_in_range(-max_allowed_y, max_allowed_y);
            x_radius = random_series.random_in_range(0.0, max_allowed_x);
            y_radius = random_series.random_in_range(0.0, max_allowed_y);
        } else {
            cluster_count_left -= 1;
        }
        
        let x0 = random_series.random_degree(x_center, x_radius, max_allowed_x);
        let y0 = random_series.random_degree(y_center, y_radius, max_allowed_y);
        let x1 = random_series.random_degree(x_center, x_radius, max_allowed_x);
        let y1 = random_series.random_degree(y_center, y_radius, max_allowed_y);

        let earth_radius = 6372.8;
        let haversine_distance = reference_haversine(x0, y0, x1, y1, earth_radius);

        sum += sum_coef * haversine_distance;

        let json_sep = if i == (num_pairs - 1) { "\n" } else { ",\n" };
        data_str += format!("    {{\"x0\":{x0:.16}, \"y0\":{y0:.16}, \"x1\":{x1:.16}, \"y1\":{y1:.16}}}{json_sep}").as_str();
        answers.extend_from_slice(&haversine_distance.to_be_bytes());
    }
    
    data_str += "]}\n";
    answers.extend_from_slice(&sum.to_be_bytes());
    
    fs::write(format!("data_{num_pairs}_flex.json"), data_str)?;
    fs::write(format!("data_{num_pairs}_haveranswer.f64"), answers)?;
    
    println!("Distribution: {distribution}");
    println!("Random seed: {random_seed}");
    println!("Pair count: {num_pairs}");
    println!("Expected sum: {sum:.16}");

    Ok(())
}
