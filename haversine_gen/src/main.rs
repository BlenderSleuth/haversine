use std::{env, fs};
use std::fmt::Display;
use std::mem::size_of;
use std::path::Path;

use anyhow::Result;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use serde::Serialize;
use serde_json::json;

#[derive(Copy, Clone)]
enum HaversineDistribution {
    Uniform,
    Cluster,
}

impl Display for HaversineDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HaversineDistribution::Uniform => write!(f, "uniform"),
            HaversineDistribution::Cluster => write!(f, "cluster"),
        }
    }
}

#[derive(Serialize)]
struct Pair {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

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

fn random_pair(x_dist: Uniform<f64>, y_dist: Uniform<f64>, rng: &mut StdRng) -> (Pair, f64) {
    let pair = Pair {
        x0: x_dist.sample(rng),
        y0: y_dist.sample(rng),
        x1: x_dist.sample(rng),
        y1: y_dist.sample(rng),
    };
    let answer = reference_haversine(pair.x0, pair.y0, pair.x1, pair.y1, 6372.8);
    (pair, answer)
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();

    let mut distribution = HaversineDistribution::Uniform;
    let mut random_seed = 23890454589;
    let mut num_pairs: usize = 10;
    let mut invalid_args = args.len() != 4;

    for i in 1..args.len() {
        if invalid_args {
            break;
        }
        match i {
            1 => {
                match args[i].as_str() {
                    "uniform" => distribution = HaversineDistribution::Uniform,
                    "cluster" => distribution = HaversineDistribution::Cluster,
                    _ => {
                        invalid_args = true;
                    }
                }
            }
            2 => {
                if let Ok(parsed_value) = args[i].parse::<u64>() {
                    random_seed = parsed_value;
                } else {
                    invalid_args = true;
                }
            }
            3 => {
                if let Ok(parsed_value) = args[i].parse::<usize>() {
                    num_pairs = parsed_value;
                } else {
                    invalid_args = true;
                }
            }
            _ => {
                invalid_args = true;
            }
        }
    }

    if invalid_args {
        // Print usage
        let exe_name =
            Path::new(&env::current_exe().unwrap())
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap().to_string();

        println!("Usage: {exe_name} [uniform/cluster] [random seed] [number of coordinate pairs to generate]", );
        return Ok(());
    }

    println!("Distribution: {distribution}");
    println!("Random seed: {random_seed}");
    println!("Pair count: {num_pairs}");

    let mut expected_sum = 0.0;
    let mut rng: StdRng = SeedableRng::seed_from_u64(random_seed);


    let mut pairs = Vec::<Pair>::with_capacity(num_pairs);
    let mut answers = Vec::<u8>::with_capacity(num_pairs * size_of::<f64>());

    // sample clusters on a sphere
    match distribution {
        HaversineDistribution::Uniform => {
            let x_dist = Uniform::<f64>::new_inclusive(-180.0, 180.0);
            let y_dist = Uniform::<f64>::new_inclusive(-90.0, 90.0);

            for _ in 0..num_pairs {
                let (pair, answer) = random_pair(x_dist, y_dist, &mut rng);
                expected_sum += answer;

                pairs.push(pair);
                answers.extend_from_slice(&answer.to_be_bytes());
            }
        }
        HaversineDistribution::Cluster => {
            let num_groups = 64;
            let group_size = num_pairs.div_ceil(num_groups);

            let num_x_divisions = 16;
            let num_y_divisions = num_x_divisions / 2;
            let division = 360.0 / num_x_divisions as f64;

            let x_division = Uniform::<i32>::new(-num_x_divisions / 2, num_x_divisions / 2);
            let y_division = Uniform::<i32>::new(-num_y_divisions / 2, num_y_divisions / 2);

            'group_iter: for _ in 0..num_groups {
                let x_group = x_division.sample(&mut rng) as f64 * division;
                let y_group = y_division.sample(&mut rng) as f64 * division;

                let x_dist = Uniform::<f64>::new_inclusive(x_group, x_group + division);
                let y_dist = Uniform::<f64>::new_inclusive(y_group, y_group + division);

                for _ in 0..group_size {
                    if pairs.capacity() == pairs.len() {
                        break 'group_iter; // Generate no more points (last group is truncated)
                    }

                    let (pair, answer) = random_pair(x_dist, y_dist, &mut rng);
                    expected_sum += answer;

                    pairs.push(pair);
                    answers.extend_from_slice(&answer.to_be_bytes());
                }
            }
        }
    }

    expected_sum /= num_pairs as f64;
    answers.extend_from_slice(&expected_sum.to_be_bytes());
    println!("Expected sum: {expected_sum}");

    let j = json!({"pairs": pairs});

    fs::write(format!("data_{num_pairs}_flex.json"), j.to_string())?;
    fs::write(format!("data_{num_pairs}_haveranswer.f64"), answers)?;

    Ok(())
}
