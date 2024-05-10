#![feature(let_chains)]
#![feature(slice_split_once)]
#![feature(unchecked_math)]
#![feature(const_inherent_unchecked_arith)]
#![feature(hasher_prefixfree_extras)]
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasherDefault, Hasher};
use std::{fmt::Display, fs::File, path::PathBuf};

use clap::Parser;
use memmap2::Mmap;
use rayon::prelude::*;

static HASH_TABLE_SIZE: u64 = 1 << 17;

#[derive(Debug)]
struct City<'a> {
    name: &'a str,
    count: usize,
    min: f32,
    max: f32,
    sum: f32,
}

impl<'a> City<'a> {
    fn mean(&self) -> f32 {
        ((10.0 * self.sum) / self.count as f32).round() / 10.0
    }
}

impl<'a> Display for City<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={:.1}/{:.1}/{:.1}",
            self.name,
            self.min,
            self.mean(),
            self.max
        )
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
    let file_len = file.metadata()?.len();
    let buf = unsafe { Mmap::map(&file)? };

    dbg!(file_len, buf.len());

    let cities = buf
        .par_split(|i| *i == b'\n')
        .fold(
            || {
                HashMap::with_capacity_and_hasher(
                    HASH_TABLE_SIZE as usize,
                    BuildNameHasher::default(),
                )
            },
            |mut map: HashMap<&[u8], City, BuildNameHasher>, line| {
                if let Some((name, temp)) = line.split_once(|i| *i == b';') {
                    let temp: f32 = unsafe { std::str::from_utf8_unchecked(temp).parse().unwrap() };
                    map.entry(name)
                        .and_modify(|city_data| {
                            city_data.min = city_data.min.min(temp);
                            city_data.max = city_data.max.max(temp);
                            city_data.count += 1;
                            city_data.sum += temp;
                        })
                        .or_insert(City {
                            name: unsafe { std::str::from_utf8_unchecked(name) },
                            min: temp,
                            max: temp,
                            count: 1,
                            sum: temp,
                        });
                }
                map
            },
        )
        .fold(
            || BTreeMap::new(),
            |mut l, r| {
                r.into_iter().for_each(|(r_city_name, r_city_data)| {
                    l.entry(r_city_name)
                        .and_modify(|l_city_data: &mut City| {
                            l_city_data.min = l_city_data.min.min(r_city_data.min);
                            l_city_data.max = l_city_data.max.max(r_city_data.max);
                            l_city_data.count += r_city_data.count;
                            l_city_data.sum += r_city_data.sum;
                        })
                        .or_insert(r_city_data);
                });
                l
            },
        )
        .reduce(
            || BTreeMap::new(),
            |mut l, r| {
                r.into_iter().for_each(|(r_city_name, r_city_data)| {
                    l.entry(r_city_name)
                        .and_modify(|l_city_data: &mut City| {
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
            .into_values()
            .map(|city_data| format!("{city_data}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("}}");

    Ok(())
}

const MASKS: [u64; 8] = {
    let mut result = [0; 8];
    let mut i = 0;
    while i < 7 {
        result[i] = !(u64::MAX << (i + 1) * 8);
        i += 1;
    }
    result[7] = u64::MAX;
    result
};

#[derive(Default)]
struct NameHasher {
    hash: Option<u64>,
}

impl Hasher for NameHasher {
    fn write(&mut self, bytes: &[u8]) {
        match self.hash {
            Some(_) => panic!("Name hasher: second write attempted"),
            None => {
                self.hash = Some({
                    let name_p1 =
                        u64::from_le_bytes(unsafe { *(bytes.as_ptr() as *const [u8; 8]) });
                    let name_p2 = u64::from_le_bytes(unsafe {
                        *((bytes.as_ptr() as usize + 8) as *const [u8; 8])
                    });
                    if bytes.len() < 17 {
                        let mask_1 = MASKS[(bytes.len() - 1).min(7)];
                        let masked_p1 = name_p1 & mask_1;
                        let mask_2 = MASKS[(bytes.len()).max(9) - 9]
                            & if bytes.len() > 8 { u64::MAX } else { 0 };
                        let masked_p2 = name_p2 & mask_2;
                        masked_p1 ^ masked_p2
                    } else {
                        let mut hash = name_p1 ^ name_p2;
                        for i in (16..bytes.len()).step_by(8) {
                            let name_chunk = u64::from_le_bytes(unsafe {
                                *((bytes.as_ptr() as usize + i) as *const [u8; 8])
                            });
                            hash ^= name_chunk & MASKS[(bytes.len() - i - 1).min(7)];
                        }
                        hash
                    }
                })
            }
        }
    }

    // No-op; prevent multiple calls to `write`
    fn write_length_prefix(&mut self, _len: usize) {}

    fn finish(&self) -> u64 {
        self.hash.unwrap()
    }
}

type BuildNameHasher = BuildHasherDefault<NameHasher>;

#[cfg(test)]
fn hash_name(name: &[u8]) -> u64 {
    let mut hasher = NameHasher::default();
    hasher.write(name);
    hasher.finish()
}

#[test]
fn hash_name_1() {
    assert_eq!(hash_name(b"a"), 0x61);
}

#[test]
fn hash_name_lt8() {
    assert_eq!(hash_name(b"abcdefgh"), 0x6867666564636261);
}

#[test]
fn hash_name_lt16() {
    assert_eq!(hash_name(b"abcdefghij"), 0x6867666564636261 ^ 0x6a69);
}

#[test]
fn hash_name_gte16() {
    assert_eq!(
        hash_name(b"abcdefghijklmnopqr"),
        0x6867666564636261 ^ 0x706f6e6d6c6b6a69 ^ 0x7271
    );
}

#[test]
fn hash_name_gte24() {
    assert_eq!(
        hash_name(b"abcdefghijklmnopqrstuvwxyz{"),
        0x6867666564636261 ^ 0x706f6e6d6c6b6a69 ^ 0x7877767574737271 ^ 0x7b7a79
    );
}
