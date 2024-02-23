use std::str::Chars;
use haversine::time_function;

use crate::Pair;

enum ParseState {
    X0,
    Y0,
    X1,
    Y1,
}

impl ParseState {
    pub fn next(&mut self) {
        *self = match self {
            ParseState::X0 => ParseState::Y0,
            ParseState::Y0 => ParseState::X1,
            ParseState::X1 => ParseState::Y1,
            ParseState::Y1 => ParseState::X0,
        }
    }
}

fn get_digit(digit: char) -> Option<u8> {
    (digit as u8).checked_sub(b'0')
}

pub fn parse_num(input: &mut Chars) -> Option<f64> {
    // Parse number
    let mut valid_num = false;
    let mut num = 0.0;
    let mut num_negative = false;
    let mut frac_part = false;
    let mut frac_div = 0.1;
    while let Some(digit) = input.next() {
        if digit.is_whitespace() {
            continue;
        } else if digit == '-' {
            num_negative = true;
        } else if digit == '.' {
            frac_part = true;
        } else if let Some(digit) = get_digit(digit) {
            valid_num = true;
            if frac_part {
                num += (digit as f64) * frac_div;
                frac_div *= 0.1;
            } else {
                num = num * 10.0 + digit as f64;
            }
        } else {
            break;
        }
    }
    if !valid_num {
        return None;
    }

    if num_negative {
        num = -num;
    }

    Some(num)
}

pub fn parse_pairs(input: &str) -> Option<Vec<Pair>> {
    time_function!(1);
    
    let minimum_json_pair_encoding = 24 * 4; // Minimum 24 bytes per number encoding
    let max_pair_count = input.len() / minimum_json_pair_encoding;

    let mut pairs = vec![Pair::default(); max_pair_count];
    let mut pair_idx = 0;

    let mut input = input.chars();
    let mut start_parsing = false;
    let mut state = ParseState::X0;
    while let Some(char) = input.next() {
        // Wait until [ to start parsing
        if char == '[' {
            start_parsing = true;
            continue;
        }

        if !start_parsing {
            continue;
        }

        // Use : to delimit the start of a number
        if char == ':' {
            // Parse number
            let num = parse_num(&mut input)?;

            match state {
                ParseState::X0 => {
                    pairs[pair_idx].x0 = num;
                }
                ParseState::Y0 => {
                    pairs[pair_idx].y0 = num;
                }
                ParseState::X1 => {
                    pairs[pair_idx].x1 = num;
                }
                ParseState::Y1 => {
                    pairs[pair_idx].y1 = num;
                    pair_idx += 1;
                }
            }

            state.next();
        }
    }

    pairs.truncate(pair_idx);
    Some(pairs)
}