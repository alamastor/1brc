use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};

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

fn main() -> anyhow::Result<()> {
    let file = File::open("measurements.txt")?;
    let buf_reader = BufReader::new(file);
    let mut cities: HashMap<String, CityData> = HashMap::new();
    for (i, line) in buf_reader.lines().enumerate() {
        let line = line?;
        let (city, temp) = line.split_once(';').unwrap();
        let temp: f32 = temp.parse().unwrap();
        let city = city.to_string();
        cities
            .entry(city)
            .and_modify(|d| {
                d.count += 1;
                d.min = f32::min(d.min, temp);
                d.max = f32::max(d.max, temp);
                d.sum += temp;
            })
            .or_insert_with(|| CityData {
                count: 1,
                min: temp,
                max: temp,
                sum: temp,
            });
        if i == 1_000_000_000 {
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
