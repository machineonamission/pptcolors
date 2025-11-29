use rayon::iter::ParallelIterator;
use num::{BigInt, Rational64, FromPrimitive};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::max;
use dashmap::{DashMap, DashSet};
use rayon::iter::IntoParallelRefIterator;

const INPUT: [&str; 143] = [
    "000000", "000066", "000099", "0000CC", "0000FF", "003300", "003366", "003399", "0033CC",
    "006600", "006666", "006699", "0066CC", "0066FF", "008000", "008080", "009900", "009999",
    "0099CC", "0099FF", "00CC00", "00CC66", "00CC99", "00CCFF", "00FF00", "00FF99", "00FFCC",
    "00FFFF", "080808", "111111", "1C1C1C", "292929", "333300", "333333", "333399", "3333CC",
    "3333FF", "336600", "336699", "3366CC", "3366FF", "339933", "339966", "3399FF", "33CC33",
    "33CCCC", "33CCFF", "4D4D4D", "5F5F5F", "660033", "660066", "6600CC", "6600FF", "663300",
    "666633", "666699", "6666FF", "669900", "6699FF", "66CCFF", "66FF33", "66FF66", "66FF99",
    "66FFCC", "66FFFF", "777777", "800000", "800080", "808000", "808080", "969696", "990000",
    "990033", "990099", "9900CC", "9900FF", "993300", "993366", "9933FF", "996600", "996633",
    "9966FF", "9999FF", "99CC00", "99CCFF", "99FF33", "99FF66", "99FF99", "99FFCC", "A50021",
    "B2B2B2", "C0C0C0", "CC0000", "CC0066", "CC0099", "CC00CC", "CC00FF", "CC3300", "CC3399",
    "CC6600", "CC66FF", "CC9900", "CC99FF", "CCCC00", "CCCCFF", "CCECFF", "CCFF33", "CCFF66",
    "CCFF99", "CCFFCC", "CCFFFF", "D60093", "DDDDDD", "EAEAEA", "F8F8F8", "FF0000", "FF0066",
    "FF00FF", "FF3300", "FF3399", "FF33CC", "FF5050", "FF6600", "FF6699", "FF66CC", "FF66FF",
    "FF7C80", "FF9900", "FF9933", "FF9966", "FF9999", "FF99CC", "FF99FF", "FFCC00", "FFCC66",
    "FFCC99", "FFCCCC", "FFCCFF", "FFFF00", "FFFF66", "FFFF99", "FFFFCC", "FFFFFF",
];

#[derive(Eq, Hash, PartialEq, Clone)]
struct ColorInt {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct ColorFractions {
    r: Rational64,
    g: Rational64,
    b: Rational64,
}

impl ColorInt {
    fn to_index(&self) -> usize {
        (self.r as usize) << 16 | (self.g as usize) << 8 | (self.b as usize)
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct ColorConstruction {
    color1: ColorFractions,
    color2: ColorFractions,
    transparency: u8,
    steps: usize,
}

#[derive(Eq, Hash, PartialEq, Clone)]
enum ColorMix {
    Base(ColorFractions),
    Mixed(ColorConstruction),
}

fn hex_to_int(hex: &str) -> u8 {
    u8::from_str_radix(hex, 16).unwrap()
}

fn average_colors(color1: &ColorInt, color2: &ColorInt, transparency: u8) -> ColorInt {
    ColorInt {
        r: ((color1.r as u16 * (100 - transparency) as u16 + color2.r as u16 * transparency as u16)
            / 100) as u8,
        g: ((color1.g as u16 * (100 - transparency) as u16 + color2.g as u16 * transparency as u16)
            / 100) as u8,
        b: ((color1.b as u16 * (100 - transparency) as u16 + color2.b as u16 * transparency as u16)
            / 100) as u8,
    }
}

fn average_colors_fractions(
    color1: &ColorFractions,
    color2: &ColorFractions,
    ratio: &Rational64,
    inv_ratio: &Rational64,
) -> ColorFractions {
    ColorFractions {
        r: &color1.r * ratio + &color2.r * inv_ratio,
        g: &color1.g * ratio + &color2.g * inv_ratio,
        b: &color1.b * ratio + &color2.b * inv_ratio,
    }
}

const TRANSPARENCIES: [u8; 6] = [15, 30, 50, 65, 80, 95];

const TOTAL_COLORS: usize = 2usize.pow(8).pow(3);

fn main() {
    let colors: Vec<ColorInt> = INPUT
        .iter()
        .map(|&hex| ColorInt {
            r: hex_to_int(&hex[0..2]),
            g: hex_to_int(&hex[2..4]),
            b: hex_to_int(&hex[4..6]),
        })
        .collect();

    let color_fractions: Vec<ColorFractions> = colors
        .iter()
        .clone()
        .map(|hex| ColorFractions {
            r: Rational64::from_u8(hex.r).unwrap(),
            g: Rational64::from_u8(hex.g).unwrap(),
            b: Rational64::from_u8(hex.b).unwrap(),
        })
        .collect();

    let transparencies_ratios: [(u8, Rational64, Rational64); 6] = TRANSPARENCIES
        .iter()
        .map(|&t| {
            let base = 100i64;
            let top = t as i64;
            (
                t.clone(),
                Rational64::new(top.clone(), base.clone()),
                Rational64::new(base.clone() - top.clone(), base.clone()),
            )
        })
        .collect::<Vec<(u8, Rational64, Rational64)>>()
        .try_into()
        .unwrap();

    let mut combination_mixed: DashSet<(ColorFractions, ColorFractions)> = DashSet::default();

    let mut constructions: DashMap<ColorFractions, ColorMix> = DashMap::default();

    // let mut int_constructions: FxHashSet<ColorFractions> = FxHashSet::default();

    for color in &color_fractions {
        constructions.insert(color.clone(), ColorMix::Base(color.clone()));
        // int_constructions.insert(color.clone());
    }
    let mut tries = 0;
    let mut iteration = 1;
    loop {
        println!("iteration: {}", iteration);
        println!("Combinations tried: {}", combination_mixed.len());
        println!("Constructions found: {}", constructions.len());
        println!();
        let keys: Vec<ColorFractions> = constructions.iter().map(|ref_multi| ref_multi.key().clone()).collect();
        keys.par_iter().for_each(|color| {
            let mut tries = 0;
            for other_color in &keys {
                tries += 1;
                if tries % 10_000 == 0 {
                    println!("iteration: {}", iteration);
                    println!("Combinations tried: {}", combination_mixed.len());
                    println!("Constructions found: {}", constructions.len());
                    println!();
                }
                if combination_mixed.contains(&(color.clone(), other_color.clone())) {
                    continue;
                }
                combination_mixed.insert((color.clone(), other_color.clone()));
                for (transparency, ratio, inv_ratio) in transparencies_ratios.iter() {
                    let mixed_color =
                        average_colors_fractions(color, other_color, ratio, inv_ratio);
                    if mixed_color.r.is_integer()
                        && mixed_color.g.is_integer()
                        && mixed_color.b.is_integer()
                    {
                        if !constructions.contains_key(&mixed_color) {
                            // let const1 = constructions.get(color).unwrap();
                            // let const2 = constructions.get(other_color).unwrap();
                            let construction = ColorConstruction {
                                color1: color.clone(),
                                color2: other_color.clone(),
                                transparency: transparency.clone(),
                                steps: 0//max(const1_steps, const2_steps) + 1,
                            };
                            constructions.insert(mixed_color.clone(), ColorMix::Mixed(construction));
                        }
                    }

                }
            }
        });
        iteration += 1;
    }
}
