use dashmap::{DashMap, DashSet};
use num::{BigInt, FromPrimitive, Rational64};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::max;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

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

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
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

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
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

fn average_colors(c1: &ColorInt, c2: &ColorInt, t: u8) -> Option<ColorInt> {
    // The numbers go inside ::< ... > because they are compile-time constants.
    // 't' stays in the ( ... ) because it is used for the final calculation.
    match t {
        50 => check_channels::<2>(c1, c2, t),
        80 => check_channels::<5>(c1, c2, t),
        30 => check_channels::<10>(c1, c2, t),
        15 | 65 | 95 => check_channels::<20>(c1, c2, t),
        _ => None,
    }
}

// 1. 'D' is defined here as a const generic parameter
// 2. We take 3 runtime arguments: c1, c2, t
#[inline(always)]
fn check_channels<const D: i16>(c1: &ColorInt, c2: &ColorInt, t: u8) -> Option<ColorInt> {
    let dr = c2.r as i16 - c1.r as i16;

    // The compiler sees "dr % 2" (or 5, 10, etc.) and optimizes it.
    if dr % D != 0 {
        return None;
    }

    let dg = c2.g as i16 - c1.g as i16;
    if dg % D != 0 {
        return None;
    }

    let db = c2.b as i16 - c1.b as i16;
    if db % D != 0 {
        return None;
    }

    Some(ColorInt {
        // We still need 't' here to calculate the actual blended color
        r: (c1.r as i16 + (dr * t as i16) / 100) as u8,
        g: (c1.g as i16 + (dg * t as i16) / 100) as u8,
        b: (c1.b as i16 + (db * t as i16) / 100) as u8,
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

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct ColorDetailed {
    color1: Box<MixDetailed>,
    color2: Box<MixDetailed>,
    transparency: u8,
    steps: usize,
}
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum MixDetailed {
    Base(ColorInt),
    Mixed(ColorDetailed),
}
fn trace_color(mix: &ColorMix, constructions: &Vec<OnceLock<ColorMix>>) -> MixDetailed {
    match mix {
        ColorMix::Base(a) => MixDetailed::Base(ColorInt::from_index(*a)),
        ColorMix::Mixed(construction) => MixDetailed::Mixed(ColorDetailed {
            color1: Box::from(trace_color(
                &constructions[construction.color1 as usize].get().unwrap(),
                constructions,
            )),
            color2: Box::from(trace_color(
                &constructions[construction.color2 as usize].get().unwrap(),
                constructions,
            )),
            transparency: construction.transparency,
            steps: construction.steps,
        }),
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
        .filter(|c| {
            c.r == 255 || c.r == 0 && c.g == 255 || c.g == 0 && c.b == 255 || c.b == 0
        })
        .collect();

    // let interesting_values = [0u8, 1u8, 255u8];
    // let mut interesting_colors: Vec<ColorInt> = vec![];
    // for r in &interesting_values {
    //     for g in &interesting_values {
    //         for b in &interesting_values {
    //             let color = ColorInt {
    //                 r: *r,
    //                 g: *g,
    //                 b: *b,
    //             };
    //             interesting_colors.push(color);
    //         }
    //     }
    // }
    let critical_values: Vec<u8> = [
        1, 2, 4, 7, 8, 11, 13, 14, 16, 19, 22, 23, 26, 28, 29, 31, 32, 37, 38, 41, 43, 44, 46, 47,
        49, 52, 53, 56, 58, 59, 61, 62, 64, 67, 71, 73, 74, 76, 77, 79, 82, 83, 86, 88, 89, 91, 92,
        94, 97, 98, 101, 103, 104, 106, 107, 109, 112, 113, 116, 118, 121, 122, 124, 127, 128, 131,
        133, 134, 137, 139, 142, 143, 146, 148, 149, 151, 152, 154, 157, 158, 161, 163, 164, 166,
        167, 169, 172, 173, 176, 178, 179, 181, 182, 184, 188, 191, 193, 194, 196, 197, 199, 202,
        203, 206, 208, 209, 211, 212, 214, 217, 218, 223, 224, 226, 227, 229, 232, 233, 236, 239,
        241, 242, 244, 247, 248, 251, 253, 254,
    ]
    .try_into()
    .unwrap();
    let mut critical_array: [bool; 256] = [false; 256];
    for v in critical_values.clone() {
        critical_array[v as usize] = true;
    }
    // let interesting_indices: Vec<u32> = interesting_colors.iter().map(|c| c.to_index()).collect();
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
        let mut keys: Vec<u32> = vec![];
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
                    if let Some(mixed_color) = average_colors(
                        &ColorInt::from_index(*color),
                        &ColorInt::from_index(*other_color),
                        *transparency,
                    ) {
                        let mixed_index = mixed_color.to_index() as usize;
                        if constructions[mixed_index].get().is_none() {
                            // let const1 = constructions.get(color).unwrap();
                            // let const2 = constructions.get(other_color).unwrap();
                            let construction = ColorConstruction {
                                color1: color.clone(),
                                color2: other_color.clone(),
                                transparency: transparency.clone(),
                                steps: 0, //max(const1_steps, const2_steps) + 1,
                            };

                            if constructions[mixed_index]
                                .set(ColorMix::Mixed(construction.clone()))
                                .is_ok()
                            {

                                let t = total.fetch_add(1, Ordering::Relaxed);
                                // if t & (2<<20)-1 == 0 {
                                //     std::process::exit(0);
                                // }
                                if t & (2 << 16) - 1 == 0 {
                                    println!(
                                        "Constructions found: {} ({}%)",
                                        t,
                                        t as f64 / (TOTAL_COLORS as f64) * 100.0
                                    );

                                    for c_r in critical_values.clone() {
                                        for c_g in critical_values.clone() {
                                            'outer: for c_b in critical_values.clone() {
                                                for r in [0,c_r,255] {
                                                    for g in [0,c_g,255] {
                                                        for b in [0,c_b,255] {
                                                            let col = ColorInt {
                                                                r,g,b
                                                            };
                                                            if constructions[col.to_index() as usize].get().is_none() {
                                                                continue 'outer;
                                                            }
                                                        }
                                                    }
                                                }
                                                println!("HOOOOOLY FUCKING SHIT");
                                                println!("{} {} {}", c_r, c_g, c_b);
                                            }
                                        }
                                    }

                                }
                                if t == TOTAL_COLORS as u32 {
                                    println!("All colors constructed!");
                                    std::process::exit(0);
                                }
                            }
                        }
                    }
                }
            }
        });
        iteration += 1;
    }
}
