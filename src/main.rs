#![feature(let_chains)]
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;

#[derive(Debug)]
struct CityData {
    count: usize,
    min: f32,
    max: f32,
    sum: f32,
}

impl CityData {
    fn mean(&self) -> f32 {
        ((10.0 * self.sum) / self.count as f32).round() / 10.0
    }
}

impl Display for CityData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}/{:.1}/{:.1}", self.min, self.mean(), self.max)
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long, help = "Number of rows to process")]
    limit: Option<usize>,
    #[arg(
        short,
        long,
        help = "Path to measurements file",
        default_value = "measurements.txt"
    )]
    measurements_file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let file = File::open(cli.measurements_file)?;
    let mut buf_reader = BufReader::new(file);
    let mut cities: HashMap<String, CityData> = HashMap::new();
    let mut buf = String::with_capacity(1024);
    let mut i = 0;
    loop {
        buf.clear();
        buf_reader.read_line(&mut buf)?;
        if buf.is_empty() {
            break;
        }
        let (city, temp) = buf.split_once(';').unwrap();
        let temp: f32 = temp[..temp.len() - 1].parse().unwrap();
        match cities.get_mut(city) {
            Some(d) => {
                d.count += 1;
                d.min = f32::min(d.min, temp);
                d.max = f32::max(d.max, temp);
                d.sum += temp;
            }
            None => {
                cities.insert(
                    city.to_string(),
                    CityData {
                        count: 1,
                        min: temp,
                        max: temp,
                        sum: temp,
                    },
                );
            }
        }
        i += 1;
        if let Some(limit) = cli.limit && limit == i {
            break;
        }
    }

    print!("{{");
    let mut sorted_cities = cities.keys().collect::<Vec<_>>();
    sorted_cities.sort();
    print!(
        "{}",
        sorted_cities
            .into_iter()
            .map(|city| format!("{city}={}", cities.get(city).unwrap()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("}}");

    Ok(())
}
