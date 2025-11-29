use dashmap::{DashMap, DashSet};
use num::{BigInt, FromPrimitive, Rational64};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::max;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;

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


// A simple alias for our color tuple (R, G, B)
type Color = (u8, u8, u8);

/// Holds the final result of the optimization
#[derive(Debug)]
struct SubSetResult {
    r_values: Vec<u8>,
    g_values: Vec<u8>,
    b_values: Vec<u8>,
    volume: usize,
}

fn solve_max_subset(hex_list: &[&str]) -> SubSetResult {
    let mut points: HashSet<Color> = HashSet::new();

    // 1. Parse Hex Strings to (u8, u8, u8)
    for hex in hex_list {
        if hex.len() != 6 { continue; }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        points.insert((r, g, b));
    }

    if points.is_empty() {
        return SubSetResult { r_values: vec![], g_values: vec![], b_values: vec![], volume: 0 };
    }

    // 2. Build Adjacency Maps for O(1) lookups during optimization
    // r_map[x] contains all points where R = x
    let mut r_map: HashMap<u8, Vec<Color>> = HashMap::new();
    let mut g_map: HashMap<u8, Vec<Color>> = HashMap::new();
    let mut b_map: HashMap<u8, Vec<Color>> = HashMap::new();

    for &p in &points {
        r_map.entry(p.0).or_default().push(p);
        g_map.entry(p.1).or_default().push(p);
        b_map.entry(p.2).or_default().push(p);
    }

    let mut best_volume = 0;
    let mut best_r: Vec<u8> = Vec::new();
    let mut best_g: Vec<u8> = Vec::new();
    let mut best_b: Vec<u8> = Vec::new();

    // 3. Iterate through every unique point as a "seed"
    // We treat each point as the start of a potential block and try to grow it.
    let sorted_points: Vec<Color> = points.iter().cloned().collect();

    for seed in sorted_points {
        let mut curr_r: HashSet<u8> = HashSet::from([seed.0]);
        let mut curr_g: HashSet<u8> = HashSet::from([seed.1]);
        let mut curr_b: HashSet<u8> = HashSet::from([seed.2]);

        let mut changed = true;

        while changed {
            changed = false;

            // --- Optimize R axis ---
            // Keep G and B constant. Find all R's that satisfy ALL current G's and B's.

            // Optimization: Candidate R's must exist with the first G and first B in our current sets.
            // This drastically reduces the search space compared to checking 0..255
            let sample_g = *curr_g.iter().next().unwrap();

            // Get all points that have our sample G
            let potential_points = g_map.get(&sample_g);

            if let Some(p_list) = potential_points {
                let mut candidate_rs = HashSet::new();
                for p in p_list {
                    // Filter: Only consider R's where the point's B is also in our current B set
                    if curr_b.contains(&p.2) {
                        candidate_rs.insert(p.0);
                    }
                }

                let mut new_r_set = HashSet::new();
                for r in candidate_rs {
                    // The Heavy Check: Does this r work with EVERY g and EVERY b currently selected?
                    let mut valid = true;
                    'g_loop: for g in &curr_g {
                        for b in &curr_b {
                            if !points.contains(&(r, *g, *b)) {
                                valid = false;
                                break 'g_loop;
                            }
                        }
                    }
                    if valid {
                        new_r_set.insert(r);
                    }
                }

                if new_r_set != curr_r {
                    curr_r = new_r_set;
                    changed = true;
                }
            }

            // --- Optimize G axis ---
            let sample_r = *curr_r.iter().next().unwrap();
            if let Some(p_list) = r_map.get(&sample_r) {
                let mut candidate_gs = HashSet::new();
                for p in p_list {
                    if curr_b.contains(&p.2) { candidate_gs.insert(p.1); }
                }

                let mut new_g_set = HashSet::new();
                for g in candidate_gs {
                    let mut valid = true;
                    'r_loop: for r in &curr_r {
                        for b in &curr_b {
                            if !points.contains(&(*r, g, *b)) {
                                valid = false;
                                break 'r_loop;
                            }
                        }
                    }
                    if valid { new_g_set.insert(g); }
                }

                if new_g_set != curr_g {
                    curr_g = new_g_set;
                    changed = true;
                }
            }

            // --- Optimize B axis ---
            let sample_r = *curr_r.iter().next().unwrap();
            if let Some(p_list) = r_map.get(&sample_r) {
                let mut candidate_bs = HashSet::new();
                for p in p_list {
                    if curr_g.contains(&p.1) { candidate_bs.insert(p.2); }
                }

                let mut new_b_set = HashSet::new();
                for b in candidate_bs {
                    let mut valid = true;
                    'r_loop2: for r in &curr_r {
                        for g in &curr_g {
                            if !points.contains(&(*r, *g, b)) {
                                valid = false;
                                break 'r_loop2;
                            }
                        }
                    }
                    if valid { new_b_set.insert(b); }
                }

                if new_b_set != curr_b {
                    curr_b = new_b_set;
                    changed = true;
                }
            }
        } // End convergence loop

        let volume = curr_r.len() * curr_g.len() * curr_b.len();
        if volume > best_volume {
            best_volume = volume;
            best_r = curr_r.into_iter().collect();
            best_g = curr_g.into_iter().collect();
            best_b = curr_b.into_iter().collect();
        }
    }

    SubSetResult {
        r_values: best_r,
        g_values: best_g,
        b_values: best_b,
        volume: best_volume,
    }
}

// Just a helper to prove the logic works
fn verify_result(res: &SubSetResult, input_hex: &[&str]) {
    let mut lookup = HashSet::new();
    for hex in input_hex {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            lookup.insert((r, g, b));
        }
    }

    for r in &res.r_values {
        for g in &res.g_values {
            for b in &res.b_values {
                if !lookup.contains(&(*r, *g, *b)) {
                    println!("ERROR: Generated combination {:02X}{:02X}{:02X} NOT found in source!", r, g, b);
                    return;
                }
            }
        }
    }
    println!("Verification Success: All {} combinations exist in source.", res.volume);
}

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

    fn to_hex(&self) -> String {
        format!("#{:06X}", self.to_index())
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

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct ChannelConstruction {
    color1: u8,
    color2: u8,
    transparency: u8,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum ChannelMix {
    Base(u8),
    Mixed(ChannelConstruction),
    Unknown,
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

fn average_channel<const D: i16>(c1: u8, c2: u8, t: u8) -> Option<u8> {
    let dr = c2 as i16 - c1 as i16;

    if dr % D != 0 {
        return None;
    }

    Some((c1 as i16 + (dr * t as i16) / 100) as u8)
}

fn average_channel_generic(c1: u8, c2: u8, t: u8) -> Option<u8> {
    // The numbers go inside ::< ... > because they are compile-time constants.
    // 't' stays in the ( ... ) because it is used for the final calculation.
    match t {
        50 => average_channel::<2>(c1, c2, t),
        80 => average_channel::<5>(c1, c2, t),
        30 => average_channel::<10>(c1, c2, t),
        15 | 65 | 95 => average_channel::<20>(c1, c2, t),
        _ => None,
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

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct ColorDetailed {
    color1: Box<MixDetailed>,
    color2: Box<MixDetailed>,
    transparency: u8,
    steps: usize,
    result: ColorInt,
}
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum MixDetailed {
    Base(ColorInt),
    Mixed(ColorDetailed),
}

impl MixDetailed {
    // fn color(&self) -> ColorInt {
    //     match self {
    //         MixDetailed::Base(c) => *c,
    //         MixDetailed::Mixed(c) => {
    //
    //         }
    //     }
    // }
}
// fn trace_color(mix: &ColorMix, constructions: &Vec<OnceLock<ColorMix>>) -> MixDetailed {
//     match mix {
//         ColorMix::Base(a) => MixDetailed::Base(ColorInt::from_index(*a)),
//         ColorMix::Mixed(construction) => MixDetailed::Mixed(ColorDetailed {
//             color1: Box::from(trace_color(
//                 &constructions[construction.color1 as usize].get().unwrap(),
//                 constructions,
//             )),
//             color2: Box::from(trace_color(
//                 &constructions[construction.color2 as usize].get().unwrap(),
//                 constructions,
//             )),
//             transparency: construction.transparency,
//             steps: construction.steps,
//         }),
//
//     }
// }

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct ChannelDetailed {
    color1: Box<ChannelMixDetailed>,
    color2: Box<ChannelMixDetailed>,
    transparency: u8,
    result: u8,
    depth: u8
}
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum ChannelMixDetailed {
    Base(u8),
    Mixed(ChannelDetailed),
}

impl ChannelMixDetailed {
    fn depth(&self) -> u8 {
        if let ChannelMixDetailed::Mixed(m) = &self {
            m.depth
        } else {
            1
        }
    }
}

fn trace_channel(channel:u8, constructions: &Vec<ChannelMix>) -> ChannelMixDetailed {
    match &constructions[channel as usize] {
        ChannelMix::Base(a) => ChannelMixDetailed::Base(a.clone()),
        ChannelMix::Mixed(construction) => {
            let col1 = trace_channel(
                construction.color1,
                constructions,
            );
            let col2 = trace_channel(
                construction.color2,
                constructions,
            );
            let depth = max(col1.depth(), col2.depth()) + 1;
            ChannelMixDetailed::Mixed(ChannelDetailed {
                color1: Box::from(col1),
                color2: Box::from(col2),
                transparency: construction.transparency,
                result: channel,
                depth,
            })
        },
        _ => {panic!("ermmm")}
    }
}

fn expand_depth(tree: &ChannelMixDetailed, depth:usize) -> ChannelMixDetailed {
    if depth == 1 {
        return tree.clone();
    }
    match tree {
        ChannelMixDetailed::Base(a) => {
            let col1 = expand_depth(&ChannelMixDetailed::Base(*a), depth-1);
            let col2 = expand_depth(&ChannelMixDetailed::Base(*a), depth-1);
            ChannelMixDetailed::Mixed(ChannelDetailed {
                color1: Box::new(col1),
                color2: Box::new(col2),
                transparency: 50,
                result: *a,
                depth: depth as u8 - 1,
            })
        },
        ChannelMixDetailed::Mixed(construction) => {
            let col1 = expand_depth(&construction.color1, depth-1);
            let col2 = expand_depth(&construction.color2, depth-1);
            ChannelMixDetailed::Mixed(ChannelDetailed {
                color1: Box::new(col1),
                color2: Box::new(col2),
                transparency: 50,
                result: construction.result,
                depth: depth as u8 - 1,
            })
        }
    }
}

fn merge_trees(r_tree: &ChannelMixDetailed, g_tree: &ChannelMixDetailed, b_tree: &ChannelMixDetailed) -> MixDetailed {
    match (r_tree, g_tree, b_tree) {
        (ChannelMixDetailed::Base(r), ChannelMixDetailed::Base(g), ChannelMixDetailed::Base(b)) => {
            MixDetailed::Base(ColorInt {
                r: *r,
                g: *g,
                b: *b
            })
        },
        (ChannelMixDetailed::Mixed(r), ChannelMixDetailed::Mixed(g), ChannelMixDetailed::Mixed(b)) => {
            let m1 = merge_trees(&r.color1, &g.color1, &b.color1);
            let m2 = merge_trees(&r.color2, &g.color2, &b.color2);
            if let MixDetailed::Base(c1) = &m1 {
                if let MixDetailed::Base(c2) = &m2 {
                    if c1 == c2 {
                        return MixDetailed::Base(ColorInt {
                            r: c1.r,
                            g: c1.g,
                            b: c1.b
                        })
                    }
                }
            }
            MixDetailed::Mixed(ColorDetailed {
                color1: Box::new(m1),
                color2: Box::new(m2),
                transparency: 50,
                steps: 0,
                result: ColorInt {
                    r: r.result,
                    g: g.result,
                    b: b.result
                }
            })
        },
        _ => {
            panic!("expanding failed it would seem");
        }
    }
}

// fn verify_tree(tree: &MixDetailed) -> bool{
//     match tree {
//         MixDetailed::Base(a) => true,
//         MixDetailed::Mixed(construction) => {
//             verify_tree(&*construction.color1)
//             && verify_tree(&*construction.color2)
//             && average_colors()
//         }
//         _ => {panic!("ermmm")}
//     }
// }

fn tree_readable(tree: &MixDetailed) -> String {
    match tree {
        MixDetailed::Base(a) => a.to_hex(),
        MixDetailed::Mixed(construction) => {
            format!("({} + {})", tree_readable(
                &*construction.color1,
            ),tree_readable(
                &*construction.color2,
            ))
        }
        _ => {panic!("ermmm")}
    }
}

fn trace_channel_readable(channel:u8, constructions: &Vec<ChannelMix>) -> String {
    match &constructions[channel as usize] {
        ChannelMix::Base(a) => a.to_string(),
        ChannelMix::Mixed(construction) => {
            format!("({} + {})", trace_channel_readable(
                construction.color1,
                constructions,
            ),trace_channel_readable(
                construction.color2,
                constructions,
            ))
        }
        _ => {panic!("ermmm")}
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

    // MAIN ISSUE:
    // this algorithm assumes ALL possible combinations of R, G, and B are valid colors.
    // i cannot find any canonical ones where there is 0, 255, and another color for R, G, and B.
    // if there are more colors I can input, then this may change.
    let mut Rset: FxHashSet<u8> = FxHashSet::default();
    let mut Gset: FxHashSet<u8> = FxHashSet::default();
    let mut Bset: FxHashSet<u8> = FxHashSet::default();
    for color in colors.clone() {
        Rset.insert(color.r);
        Gset.insert(color.g);
        Bset.insert(color.b);
    }

    // let mut Rs: Vec<u8> = Rset.drain().collect();
    // let mut Gs: Vec<u8> = Gset.drain().collect();
    // let mut Bs: Vec<u8> = Bset.drain().collect();
    let mut Rs: Vec<u8> = vec![0,255,8];
    let mut Gs: Vec<u8> = vec![0,255,8];
    let mut Bs: Vec<u8> = vec![0,255,8];

    // for r in &Rs {
    //     for g in &Gs {
    //         for b in &Bs {
    //             let col = ColorInt{r:*r,g:*g,b:*b};
    //             if !colors.contains(&col) {
    //                 // continue 'outer;
    //                 println!("Invalid color: {}", col.to_hex());
    //                 std::process::exit(1);
    //             }
    //         }
    //     }
    // }
    // println!("{} {} {}", thirdr, thirdb, thirdg);

    // std::process::exit(0);

    let mut trees: Vec<Vec<ChannelMixDetailed>> = vec!(vec!();3);
    for t in TRANSPARENCIES {
        let mut possibles: FxHashSet<u8> = FxHashSet::default();
        for third in 1..255 {
            let channel = vec![0,third,255];
            let mut constructions: Vec<ChannelMix> = vec![ChannelMix::Unknown; 256];
            for val in channel {
                constructions[val as usize] = ChannelMix::Base(val);
            }
            let mut iterations = 0;
            loop {
                if iterations > 100 {
                    break
                }
                let mut found_count = 0;
                for x in 0..=255 {
                    match constructions[x] {
                        ChannelMix::Unknown => {}
                        _ => {found_count += 1;}
                    }
                }
                // println!("{}", found_count);
                if found_count >= 256 {
                    // println!("third {} with transparency {} can reach all", third, t);
                    possibles.insert(third);
                    // println!("{:#?}", constructions);
                    // for x in 0..=255 {
                    //     println!("{x}={}", trace_channel_readable(x, &constructions));
                    //     trees[i].push(trace_channel(x, &constructions));
                    // }
                    break;
                }
                iterations += 1;
                // println!("iteration {}", iterations);
                for x in 0..=255 {
                    match constructions[x] {
                        ChannelMix::Unknown => {}
                        _ => {
                            for y in 0..=255 {
                                match constructions[y] {
                                    ChannelMix::Unknown => {}
                                    _ => {
                                        if let Some(avg) = average_channel_generic(x as u8, y as u8, t) {
                                            if let ChannelMix::Unknown = constructions[avg as usize] {
                                                constructions[avg as usize] = ChannelMix::Mixed(ChannelConstruction {
                                                    color1: x as u8,
                                                    color2: y as u8,
                                                    transparency: t,
                                                })
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                }
                // println!("{:?}", constructions)
            }
        }
        let mut p = possibles.iter().collect::<Vec<_>>();
        p.sort();
        println!("{t}: {:?}",p);
    }

    for r in 0..=255u8 {
        for g in 0..=255u8 {
            for b in 0..=255u8 {
                let tree_r = trees[0][r as usize].clone();
                let tree_g = trees[1][g as usize].clone();
                let tree_b = trees[2][b as usize].clone();
                let max_depth = *[tree_r.depth(), tree_g.depth(), tree_b.depth()].iter().max().unwrap();
                let [e_tree_r, e_tree_g, e_tree_b] = [tree_r, tree_g, tree_b].map(|t| expand_depth(&t, max_depth as usize));
                let merged_tree= merge_trees(&e_tree_r, &e_tree_g, &e_tree_b);

                println!("{} = {}",ColorInt{r,g,b}.to_hex(), tree_readable(&merged_tree));
            }
        }
    }

    // let interesting_values = [0u8,1u8,255u8];
    // let mut interesting_colors: Vec<ColorInt> = vec!();
    // for r in &interesting_values {
    //     for g in &interesting_values {
    //         for b in &interesting_values {
    //             let color = ColorInt { r: *r, g: *g, b: *b };
    //             interesting_colors.push(color);
    //         }
    //     }
    // }
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

    // let constructions: Vec<OnceLock<ColorMix>> = std::iter::repeat_with(OnceLock::new)
    //     .take(TOTAL_COLORS)
    //     .collect();
    //
    // // let mut int_constructions: FxHashSet<ColorFractions> = FxHashSet::default();
    // let mut interesting_total = AtomicU32::new(0);
    //
    // for color in &colors {
    //     if interesting_indices.contains(&(color.to_index())) {
    //         interesting_total.fetch_add(1, Ordering::Relaxed);
    //     }
    //     constructions[color.to_index() as usize].set(ColorMix::Base(color.to_index()));
    //     // int_constructions.insert(color.clone());
    // }
    // let mut total = AtomicU32::new(0);
    // let mut iteration = 1;
    // loop {
    //     println!("iteration: {}", iteration);
    //     // println!("Combinations tried: {}", combination_mixed.len());
    //     println!("Constructions found: {}", total.get_mut());
    //     println!();
    //     let mut keys: Vec<u32> = vec!();
    //     for i in 0..TOTAL_COLORS {
    //         if constructions[i].get().is_some() {
    //             keys.push(i as u32);
    //         }
    //     }
    //     keys.par_iter().for_each(|color| {
    //         for other_color in &keys {
    //             // if combination_mixed[*color as usize][*other_color as usize].load(Ordering::Relaxed) {
    //             //     continue;
    //             // }
    //             // combination_mixed[*color as usize][*other_color as usize].store(true, Ordering::Relaxed);
    //             for transparency in &TRANSPARENCIES {
    //                 if let Some(mixed_color) = average_colors(&ColorInt::from_index(*color), &ColorInt::from_index(*other_color), *transparency) {
    //                     let mixed_index = mixed_color.to_index() as usize;
    //                     if constructions[mixed_index].get().is_none() {
    //                         // let const1 = constructions.get(color).unwrap();
    //                         // let const2 = constructions.get(other_color).unwrap();
    //                         let construction = ColorConstruction {
    //                             color1: color.clone(),
    //                             color2: other_color.clone(),
    //                             transparency: transparency.clone(),
    //                             steps: 0, //max(const1_steps, const2_steps) + 1,
    //                         };
    //
    //                         if constructions[mixed_index].set(ColorMix::Mixed(construction.clone())).is_ok() {
    //                             if interesting_indices.contains(&(mixed_index as u32)) {
    //                                 let it = interesting_total.fetch_add(1, Ordering::Relaxed);
    //                                 println!("Interesting constructions found: {}", it);
    //                                 println!("{:#?}", trace_color(&ColorMix::Mixed(construction.clone()), &constructions));
    //                             }
    //                             let t = total.fetch_add(1, Ordering::Relaxed);
    //                             // if t & (2<<20)-1 == 0 {
    //                             //     std::process::exit(0);
    //                             // }
    //                             if t & (2<<16)-1 == 0 {
    //                                 println!("Constructions found: {} ({}%)", t, t as f64/(TOTAL_COLORS as f64) * 100.0);
    //                             }
    //                             if t == TOTAL_COLORS as u32 {
    //                                 println!("All colors constructed!");
    //                                 std::process::exit(0);
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     });
    //     iteration += 1;
    // }
}
