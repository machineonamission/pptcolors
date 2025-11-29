use num::Rational64;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rustc_hash::FxHashSet;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

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
    result: u32,
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

fn invert_channel<const t: u8>(avg: u8, val: u8) -> Option<u8> {
    let top = 100 * avg as i16 - (val as i16 * t as i16);
    let bottom = 100 - t as i16;
    if top % bottom == 0 {
        let res = (top / bottom);
        if 0 <= res && res <= 255 {
            Some(res as u8)
        } else {
            None
        }
    } else {
        None
    }
}

fn inverse_channel_general(average: u8, color: u8, t: u8) -> Option<u8> {
    // The numbers go inside ::< ... > because they are compile-time constants.
    // 't' stays in the ( ... ) because it is used for the final calculation.
    match t {
        15 => invert_channel::<15>(average, color),
        30 => invert_channel::<30>(average, color),
        50 => invert_channel::<50>(average, color),
        65 => invert_channel::<65>(average, color),
        80 => invert_channel::<80>(average, color),
        95 => invert_channel::<95>(average, color),
        _ => panic!("uh"),
    }
}

fn inverse_average<const t: u8>(average: &ColorInt, color: &ColorInt) -> Option<ColorInt> {
    Some(ColorInt {
        r: invert_channel::<t>(average.r, color.r)?,
        g: invert_channel::<t>(average.g, color.g)?,
        b: invert_channel::<t>(average.b, color.b)?,
    })
}

fn inverse_general(average: &ColorInt, color: &ColorInt, t: u8) -> Option<ColorInt> {
    // The numbers go inside ::< ... > because they are compile-time constants.
    // 't' stays in the ( ... ) because it is used for the final calculation.
    match t {
        15 => inverse_average::<15>(average, color),
        30 => inverse_average::<30>(average, color),
        50 => inverse_average::<50>(average, color),
        65 => inverse_average::<65>(average, color),
        80 => inverse_average::<80>(average, color),
        95 => inverse_average::<95>(average, color),
        _ => panic!("uh"),
    }
}

// #[derive(Eq, Hash, PartialEq, Clone, Debug)]
// struct ColorDetailed {
//     color1: Box<MixDetailed>,
//     color2: Box<MixDetailed>,
//     transparency: u8,
//     steps: usize,
// }
// #[derive(Eq, Hash, PartialEq, Clone, Debug)]
// enum MixDetailed {
//     Base(ColorInt),
//     Mixed(ColorDetailed),
// }
// fn trace_color(mix: &ColorMix, constructions: &Vec<OnceLock<ColorMix>>) -> MixDetailed {
//     match mix {
//         ColorMix::Base(a) => {
//             MixDetailed::Base(ColorInt::from_index(*a))
//         }
//         ColorMix::Mixed(construction) => {
//             MixDetailed::Mixed(
//                 ColorDetailed {
//                     color1: Box::from(trace_color(&constructions[construction.color1 as usize].get().unwrap(), constructions)),
//                     color2: Box::from(trace_color(&constructions[construction.color2 as usize].get().unwrap(), constructions)),
//                     transparency: construction.transparency,
//                     steps: construction.steps,
//                 }
//             )
//         }
//     }
// }

const TRANSPARENCIES: [u8; 6] = [15, 30, 50, 65, 80, 95];

const TOTAL_COLORS: usize = 2usize.pow(8).pow(3);

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum ConstructionStatus {
    IsBase,
    FromBase(ColorConstruction),        // constructed from base colors
    Incomplete(Vec<ColorConstruction>), // the constructions that USE this color, its CHILDREN
    Unconstructable,
}

fn cascade_base(
    children: &Vec<ColorConstruction>,
    index: usize,
    all_constructions: &mut Vec<ConstructionStatus>,
) {
    let mut base_construction: Option<ColorConstruction> = None;
    for construction in children.clone() {
        if let ConstructionStatus::IsBase | ConstructionStatus::FromBase(_) =
            all_constructions[construction.color1 as usize]
            && let ConstructionStatus::IsBase | ConstructionStatus::FromBase(_) =
                all_constructions[construction.color2 as usize]
        {
            base_construction = Some(construction.clone());
            break;
        }
    }
    if let Some(base_construction) = base_construction {
        all_constructions[index] = ConstructionStatus::FromBase(base_construction.clone());
        for construction in children.clone() {
            if let ConstructionStatus::Incomplete(new_children) =
                all_constructions[construction.result as usize].clone()
            {
                cascade_base(
                    &new_children,
                    construction.result as usize,
                    all_constructions,
                );
            }
        }
    }
}

fn attempt_store_construction(
    construction: ColorConstruction,
    constructions: &mut Vec<ConstructionStatus>,
    try_next: &mut Vec<bool>,
) {
    let result_index = construction.result as usize;
    // if this new construction already has a known base, ignore
    if let ConstructionStatus::IsBase | ConstructionStatus::FromBase(_) =
        &constructions[result_index]
    {
        return;
    }

    // if new construction comes from bases, store it and cascade
    if let ConstructionStatus::IsBase | ConstructionStatus::FromBase(_) =
        &constructions[construction.color1 as usize]
        && let ConstructionStatus::IsBase | ConstructionStatus::FromBase(_) =
            &constructions[construction.color2 as usize]
    {
        constructions[construction.result as usize] =
            ConstructionStatus::FromBase(construction.clone());
        cascade_base(&vec![construction.clone()], result_index, constructions);
    }

    for i in [&construction.color1, &construction.color2] {
        match &mut constructions[*i as usize] {
            ConstructionStatus::Incomplete(existing_constructions) => {
                existing_constructions.push(construction.clone());
                try_next[*i as usize] = true;
            }
            ConstructionStatus::Unconstructable => {
                let mut new_vec = Vec::new();
                new_vec.push(construction.clone());
                constructions[*i as usize] =
                    ConstructionStatus::Incomplete(new_vec);
                try_next[*i as usize] = true;
            }
            _ => {}
        }
    }
}

fn store_inverses(
    color: &ColorInt,
    constructions: &mut Vec<ConstructionStatus>,
    try_next: &mut Vec<bool>,
) {
    for transparency in &TRANSPARENCIES {
        let mut r_inverses: Vec<(u8, u8)> = vec![];
        for r in 0u8..=255u8 {
            if let Some(inverse_color) = inverse_channel_general(color.r, r, *transparency) {
                r_inverses.push((r, inverse_color));
            }
        }
        let mut g_inverses: Vec<(u8, u8)> = vec![];
        for g in 0u8..=255u8 {
            if let Some(inverse_color) = inverse_channel_general(color.g, g, *transparency) {
                g_inverses.push((g, inverse_color));
            }
        }
        let mut b_inverses: Vec<(u8, u8)> = vec![];
        for b in 0u8..=255u8 {
            if let Some(inverse_color) = inverse_channel_general(color.b, b, *transparency) {
                b_inverses.push((b, inverse_color));
            }
        }
        for r_inverse in &r_inverses {
            for g_inverse in &g_inverses {
                for b_inverse in &b_inverses {
                    let color1 = ColorInt {
                        r: r_inverse.0,
                        g: g_inverse.0,
                        b: b_inverse.0,
                    }
                    .to_index();
                    let color2 = ColorInt {
                        r: r_inverse.1,
                        g: g_inverse.1,
                        b: b_inverse.1,
                    }
                    .to_index();
                    if color1 != color2 {
                        let construction = ColorConstruction {
                            color1,
                            color2,
                            transparency: transparency.clone(),
                            result: color.to_index(),
                        };
                        attempt_store_construction(construction, constructions, try_next);
                    }
                }
            }
        }
    }
}

fn main() {
    let base_colors: Vec<ColorInt> = INPUT
        .iter()
        .map(|&hex| ColorInt {
            r: hex_to_int(&hex[0..2]),
            g: hex_to_int(&hex[2..4]),
            b: hex_to_int(&hex[4..6]),
        })
        .collect();

    let base_indices: Vec<u32> = base_colors.iter().map(|c| c.to_index()).collect();

    let base_map: FxHashSet<u32> = FxHashSet::from_iter(base_indices.clone());

    // let interesting_values = [0u8, 1u8, 255u8];
    let mut interesting_colors: Vec<ColorInt> = vec![];
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
    for (r, g, b) in [
        (0, 255, 1),
        (1, 0, 255),
        (1, 1, 255),
        (1, 255, 0),
        (1, 255, 1),
        (1, 255, 255),
        (255, 0, 1),
        (255, 255, 1),
    ] {
        interesting_colors.push(ColorInt { r, g, b });
    }
    let interesting_indices: Vec<u32> = interesting_colors.iter().map(|c| c.to_index()).collect();
    println!("{:?}", interesting_indices);
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

    let mut constructions: Vec<ConstructionStatus> = (0..TOTAL_COLORS)
        .map(|_| ConstructionStatus::Unconstructable)
        .collect();
    for base in &base_indices {
        constructions[*base as usize] = ConstructionStatus::IsBase;
    }

    // let mut int_constructions: FxHashSet<ColorFractions> = FxHashSet::default();
    let mut tries = 0;
    let mut iteration = 1;
    let mut try_inverses = vec![false; TOTAL_COLORS];
    for base in &interesting_indices {
        try_inverses[*base as usize] = true;
    }
    loop {
        println!("iteration: {}", iteration);
        // println!("Combinations tried: {}", combination_mixed.len());
        println!();
        let iteration_indices = try_inverses.clone();
        try_inverses = vec![false; TOTAL_COLORS];
        iteration_indices
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .for_each(|(index, _)| {
                tries += 1;
                // if tries > 100000{
                //     std::process::exit(0);
                // }
                // println!("{index}");
                if tries % 10_000 == 0 {
                    // std::process::exit(0);
                    let cools = interesting_indices
                        .iter()
                        .map(|i| constructions[*i as usize].clone())
                        .filter(|c| matches!(c, ConstructionStatus::FromBase(_)))
                        .collect::<Vec<_>>();
                    println!("{:?}\nCOUNT: {}", cools, cools.len());
                }
                let color = ColorInt::from_index(index as u32);
                store_inverses(&color, &mut constructions, &mut try_inverses);
            });
        println!("{}", try_inverses.iter().filter(|v| **v).count());
        // for base in &base_indices {
        //     print!("{:?} ", constructions[*base as usize]);
        // }
        iteration += 1;
        if iteration > 100 {
            return;
        }
    }
}
