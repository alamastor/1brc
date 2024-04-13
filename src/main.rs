#![feature(let_chains)]
use std::collections::BTreeMap;
use std::io::Read;
use std::{fmt::Display, fs::File, path::PathBuf};

use clap::Parser;
use rayon::prelude::*;

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
    let mut file = File::open(cli.measurements_file)?;
    let file_len = file.metadata()?.len();
    let mut buf = String::with_capacity(file_len as usize);
    file.read_to_string(&mut buf)?;
    dbg!(file_len, buf.len());

    let cities = buf
        .par_lines()
        .fold(
            || BTreeMap::new(),
            |mut map: BTreeMap<&str, CityData>, line| {
                let (city, temp) = line.split_once(';').unwrap();
                let temp: f32 = temp.parse().unwrap();
                map.entry(city)
                    .and_modify(|city_data| {
                        city_data.min = city_data.min.min(temp);
                        city_data.max = city_data.max.max(temp);
                        city_data.count += 1;
                        city_data.sum += temp;
                    })
                    .or_insert(CityData {
                        min: temp,
                        max: temp,
                        count: 1,
                        sum: temp,
                    });
                map
            },
        )
        .reduce(
            || BTreeMap::new(),
            |mut l, r| {
                r.into_iter().for_each(|(r_city_name, r_city_data)| {
                    l.entry(r_city_name)
                        .and_modify(|l_city_data| {
                            l_city_data.min = l_city_data.min.min(r_city_data.min);
                            l_city_data.max = l_city_data.max.max(r_city_data.max);
                            l_city_data.count += r_city_data.count;
                            l_city_data.sum += r_city_data.sum;
                        })
                        .or_insert(r_city_data);
                });
                l
            },
        );

    print!("{{");
    print!(
        "{}",
        cities
            .into_iter()
            .map(|(city_name, city_data)| format!("{city_name}={city_data}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("}}");

    Ok(())
}
