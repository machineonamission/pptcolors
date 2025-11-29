use dashmap::{DashMap, DashSet};
use num::{BigInt, FromPrimitive, Rational64};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::max;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::OnceLock;

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
    fn to_index(&self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }

    fn from_index(index: u32) -> ColorInt {
        ColorInt {
            r: ((index >> 16) & 0xFF) as u8,
            g: ((index >> 8) & 0xFF) as u8,
            b: (index & 0xFF) as u8,
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct ColorConstruction {
    color1: u32,
    color2: u32,
    transparency: u8,
    steps: usize,
}

#[derive(Eq, Hash, PartialEq, Clone)]
enum ColorMix {
    Base(u32),
    Mixed(ColorConstruction),
}

fn hex_to_int(hex: &str) -> u8 {
    u8::from_str_radix(hex, 16).unwrap()
}

fn average_colors(color1: &ColorInt, color2: &ColorInt, transparency: u8) -> Option<ColorInt> {
    let r_100 =
        (color1.r as u16 * (100 - transparency) as u16 + color2.r as u16 * transparency as u16);
    if r_100 % 100 != 0 {
        return None;
    }
    let g_100 =
        (color1.g as u16 * (100 - transparency) as u16 + color2.g as u16 * transparency as u16);
    if g_100 % 100 != 0 {
        return None;
    }
    let b_100 =
        (color1.b as u16 * (100 - transparency) as u16 + color2.b as u16 * transparency as u16);
    if b_100 % 100 != 0 {
        return None;
    }
    Some(ColorInt {
        r: (r_100 / 100) as u8,
        g: (g_100 / 100) as u8,
        b: (b_100 / 100) as u8,
    })
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

    // let color_fractions: Vec<ColorFractions> = colors
    //     .iter()
    //     .clone()
    //     .map(|hex| ColorFractions {
    //         r: Rational64::from_u8(hex.r).unwrap(),
    //         g: Rational64::from_u8(hex.g).unwrap(),
    //         b: Rational64::from_u8(hex.b).unwrap(),
    //     })
    //     .collect();

    // let transparencies_ratios: [(u8, Rational64, Rational64); 6] = TRANSPARENCIES
    //     .iter()
    //     .map(|&t| {
    //         let base = 100i64;
    //         let top = t as i64;
    //         (
    //             t.clone(),
    //             Rational64::new(top.clone(), base.clone()),
    //             Rational64::new(base.clone() - top.clone(), base.clone()),
    //         )
    //     })
    //     .collect::<Vec<(u8, Rational64, Rational64)>>()
    //     .try_into()
    //     .unwrap();

    // let combination_mixed: Vec<Vec<AtomicBool>> = (0..TOTAL_COLORS)
    //     .map(|_| (0..TOTAL_COLORS).map(|_| AtomicBool::new(false)).collect())
    //     .collect();

    let constructions: Vec<OnceLock<ColorMix>> = std::iter::repeat_with(OnceLock::new)
        .take(TOTAL_COLORS)
        .collect();

    // let mut int_constructions: FxHashSet<ColorFractions> = FxHashSet::default();

    for color in &colors {
        constructions[color.to_index() as usize].set(ColorMix::Base(color.to_index()));
        // int_constructions.insert(color.clone());
    }
    let mut total = AtomicU32::new(0);
    let mut iteration = 1;
    loop {
        println!("iteration: {}", iteration);
        // println!("Combinations tried: {}", combination_mixed.len());
        println!("Constructions found: {}", total.get_mut());
        println!();
        let mut keys: Vec<u32> = vec!();
        for i in 0..TOTAL_COLORS {
            if constructions[i].get().is_some() {
                keys.push(i as u32);
            }
        }
        keys.par_iter().for_each(|color| {
            for other_color in &keys {
                // if combination_mixed[*color as usize][*other_color as usize].load(Ordering::Relaxed) {
                //     continue;
                // }
                // combination_mixed[*color as usize][*other_color as usize].store(true, Ordering::Relaxed);
                for transparency in &TRANSPARENCIES {
                    if let Some(mixed_color) = average_colors(&ColorInt::from_index(*color), &ColorInt::from_index(*other_color), *transparency) {
                        let mixed_index = mixed_color.to_index() as usize;
                        if !constructions[mixed_index].get().is_some() {
                            // let const1 = constructions.get(color).unwrap();
                            // let const2 = constructions.get(other_color).unwrap();
                            let construction = ColorConstruction {
                                color1: color.clone(),
                                color2: other_color.clone(),
                                transparency: transparency.clone(),
                                steps: 0, //max(const1_steps, const2_steps) + 1,
                            };
                            constructions[mixed_index].set(ColorMix::Mixed(construction));
                            total.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
        });
        iteration += 1;
    }
}
