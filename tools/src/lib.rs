#![allow(non_snake_case, unused_macros)]

use rand::prelude::*;
use proconio::{input, marker::*};
use svg::node::{element::{Rectangle, Line, Group, Title, Definitions, Use, Path}, Text};
use std::f64::consts::PI;

pub trait SetMinMax {
	fn setmin(&mut self, v: Self) -> bool;
	fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T where T: PartialOrd {
	fn setmin(&mut self, v: T) -> bool {
		*self > v && { *self = v; true }
	}
	fn setmax(&mut self, v: T) -> bool {
		*self < v && { *self = v; true }
	}
}

#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

pub type Output = Vec<i32>;

pub const N: usize = 30;
const DIJ: [(usize, usize); 4] = [(0, !0), (!0, 0), (0, 1), (1, 0)];

#[derive(Clone, Debug)]
pub struct Input {
	tiles: Vec<Vec<usize>>,
}

impl std::fmt::Display for Input {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for i in 0..N {
			for j in 0..N {
				write!(f, "{}", self.tiles[i][j])?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

pub fn parse_input(f: &str) -> Input {
	let f = proconio::source::once::OnceSource::from(f);
	input! {
		from f,
		tiles: [Chars; N]
	}
	let tiles = tiles.iter().map(|ts| ts.iter().map(|&c| (c as u8 - b'0') as usize).collect()).collect();
	Input { tiles }
}

pub fn parse_output(_input: &Input, f: &str) -> Result<Vec<Output>, String> {
	let mut out = vec![];
	for line in f.lines() {
		let line = line.trim();
		let mut tmp = vec![];
		for c in line.chars() {
			if c < '0' || '3' < c {
				return Err(format!("Illegal output: {}", c));
			}
			tmp.push((c as u8 - b'0') as i32);
		}
		if tmp.len() != N * N {
			return Err(format!("Illegal output length: {}", line.len()));
		}
		out.push(tmp);
	}
	if out.len() == 0 {
		return Err(format!("empty output"));
	}
	Ok(out)
}

const ROTATE: [usize; 8] = [1, 2, 3, 0, 5, 4, 7, 6];

const TO: [[usize; 4]; 8] = [
	[1, 0, !0, !0],
	[3, !0, !0, 0],
	[!0, !0, 3, 2],
	[!0, 2, 1, !0],
	[1, 0, 3, 2],
	[3, 2, 1, 0],
	[2, !0, 0, !0],
	[!0, 3, !0, 1],
];

pub fn compute_score(input: &Input, out: &Output) -> (i64, String, (Vec<Vec<usize>>, Vec<Vec<Vec<(i32, i32)>>>)) {
	let mut tiles = input.tiles.clone();
	for i in 0..N {
		for j in 0..N {
			for _ in 0..out[i * N + j] {
				tiles[i][j] = ROTATE[tiles[i][j]];
			}
		}
	}
	let mut ls = vec![];
	let mut used = mat![false; N; N; 4];
	let mut cycle = mat![(0, 0); N; N; 4];
	for i in 0..N {
		for j in 0..N {
			for d in 0..4 {
				if TO[tiles[i][j]][d] != !0 && !used[i][j][d] {
					let mut i2 = i;
					let mut j2 = j;
					let mut d2 = d;
					let mut length = 0;
					let mut tmp = vec![];
					while !used[i2][j2][d2] {
						if TO[tiles[i2][j2]][d2] == !0 {
							break;
						}
						length += 1;
						used[i2][j2][d2] = true;
						tmp.push((i2, j2, d2));
						d2 = TO[tiles[i2][j2]][d2];
						used[i2][j2][d2] = true;
						tmp.push((i2, j2, d2));
						i2 += DIJ[d2].0;
						j2 += DIJ[d2].1;
						if i2 >= N || j2 >= N {
							break;
						}
						d2 = (d2 + 2) % 4;
					}
					if (i, j, d) == (i2, j2, d2) {
						ls.push((length, tmp.clone()));
						for (i, j, d) in tmp {
							cycle[i][j][d].0 = length;
						}
					}
				}
			}
		}
	}
	let score = if ls.len() <= 1 {
		0
	} else {
		ls.sort();
		for &(i, j, d) in &ls[ls.len() - 1].1 {
			cycle[i][j][d].1 = 1;
		}
		for &(i, j, d) in &ls[ls.len() - 2].1 {
			cycle[i][j][d].1 = 2;
		}
		ls[ls.len() - 1].0 * ls[ls.len() - 2].0
	};
	(score as i64, String::new(), (tiles, cycle))
}

pub fn gen(seed: u64) -> Input {
	let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
	let mut tiles = mat![0; N; N];
	for i in 0..N {
		for j in 0..N {
			tiles[i][j] = match rng.gen_range(0, 4i32) {
				0 => rng.gen_range(0, 4i32) as usize,
				3 => rng.gen_range(6, 8i32) as usize,
				_ => rng.gen_range(4, 6i32) as usize,
			};
		}
	}
	Input { tiles }
}

fn rect(x: i32, y: i32, w: i32, h: i32, fill: &str) -> Rectangle {
	Rectangle::new().set("x", x).set("y", y).set("width", w).set("height", h).set("fill", fill)
}

pub fn vis(input: &Input, out: &Output, simple: bool) -> String {
	const W: usize = 30;
	let (_, _, (tiles, used)) = compute_score(input, out);
	let mut doc = svg::Document::new().set("id", "vis").set("viewBox", (-5, -5, W * N + 10, W * N + 10)).set("width", W * N + 10).set("height", W * N + 10);
	doc = doc.add(rect(-5, -5, (W * N + 10) as i32, (W * N + 10) as i32, "white"));
	for i in 0..=N {
		doc = doc.add(Line::new().set("x1", 0).set("y1", i * W).set("x2", W * N).set("y2", i * W).set("stroke", "lightgray").set("stroke-width", 1));
		doc = doc.add(Line::new().set("y1", 0).set("x1", i * W).set("y2", W * N).set("x2", i * W).set("stroke", "lightgray").set("stroke-width", 1));
	}
	if simple {
		doc = doc.add(Definitions::new()
			.add(Group::new().set("id", "rail1")
				.add(Line::new().set("x1", W / 2).set("y1", 0).set("x2", W / 2).set("y2", W))
				.set("stroke-width", 2)
			)
			.add(Group::new().set("id", "rail2")
				.add(Line::new().set("x1", W / 2).set("y1", 0).set("x2", 0).set("y2", W / 2))
				.set("stroke-width", 2)
			)
		);
	} else {
		doc = doc.add(Definitions::new()
			.add(Group::new().set("id", "rail1")
				.add(Line::new().set("x1", W / 2 - 3).set("y1", 0).set("x2", W / 2 - 3).set("y2", W))
				.add(Line::new().set("x1", W / 2 + 3).set("y1", 0).set("x2", W / 2 + 3).set("y2", W))
				.add(Line::new().set("x1", W / 2 - 3).set("y1", W / 10 * 1).set("x2", W / 2 + 3).set("y2", W / 10 * 1))
				.add(Line::new().set("x1", W / 2 - 3).set("y1", W / 10 * 3).set("x2", W / 2 + 3).set("y2", W / 10 * 3))
				.add(Line::new().set("x1", W / 2 - 3).set("y1", W / 10 * 5).set("x2", W / 2 + 3).set("y2", W / 10 * 5))
				.add(Line::new().set("x1", W / 2 - 3).set("y1", W / 10 * 7).set("x2", W / 2 + 3).set("y2", W / 10 * 7))
				.add(Line::new().set("x1", W / 2 - 3).set("y1", W / 10 * 9).set("x2", W / 2 + 3).set("y2", W / 10 * 9))
				.set("stroke-width", 2)
			)
			.add(Group::new().set("id", "rail2")
				.add(Path::new().set("d", "M12,0 A12,12 0 0,1 0,12"))
				.add(Path::new().set("d", "M18,0 A18,18 0 0,1 0,18"))
				.add(Line::new().set("x1", ((W / 2 - 3) as f64 * f64::cos(PI / 20.0 * 1.0)).round() as i32).set("y1", ((W / 2 - 3) as f64 * f64::sin(PI / 20.0 * 1.0)).round() as i32).set("x2", ((W / 2 + 3) as f64 * f64::cos(PI / 20.0 * 1.0)).round() as i32).set("y2", ((W / 2 + 3) as f64 * f64::sin(PI / 20.0 * 1.0)).round() as i32))
				.add(Line::new().set("x1", ((W / 2 - 3) as f64 * f64::cos(PI / 20.0 * 3.0)).round() as i32).set("y1", ((W / 2 - 3) as f64 * f64::sin(PI / 20.0 * 3.0)).round() as i32).set("x2", ((W / 2 + 3) as f64 * f64::cos(PI / 20.0 * 3.0)).round() as i32).set("y2", ((W / 2 + 3) as f64 * f64::sin(PI / 20.0 * 3.0)).round() as i32))
				.add(Line::new().set("x1", ((W / 2 - 3) as f64 * f64::cos(PI / 20.0 * 5.0)).round() as i32).set("y1", ((W / 2 - 3) as f64 * f64::sin(PI / 20.0 * 5.0)).round() as i32).set("x2", ((W / 2 + 3) as f64 * f64::cos(PI / 20.0 * 5.0)).round() as i32).set("y2", ((W / 2 + 3) as f64 * f64::sin(PI / 20.0 * 5.0)).round() as i32))
				.add(Line::new().set("x1", ((W / 2 - 3) as f64 * f64::cos(PI / 20.0 * 7.0)).round() as i32).set("y1", ((W / 2 - 3) as f64 * f64::sin(PI / 20.0 * 7.0)).round() as i32).set("x2", ((W / 2 + 3) as f64 * f64::cos(PI / 20.0 * 7.0)).round() as i32).set("y2", ((W / 2 + 3) as f64 * f64::sin(PI / 20.0 * 7.0)).round() as i32))
				.add(Line::new().set("x1", ((W / 2 - 3) as f64 * f64::cos(PI / 20.0 * 9.0)).round() as i32).set("y1", ((W / 2 - 3) as f64 * f64::sin(PI / 20.0 * 9.0)).round() as i32).set("x2", ((W / 2 + 3) as f64 * f64::cos(PI / 20.0 * 9.0)).round() as i32).set("y2", ((W / 2 + 3) as f64 * f64::sin(PI / 20.0 * 9.0)).round() as i32))
				.set("fill", "none")
				.set("stroke-width", 2)
			)
		);
	}
	for i in 0..N {
		for j in 0..N {
			for d in 0..4 {
				if TO[tiles[i][j]][d] != !0 && d < TO[tiles[i][j]][d] {
					let d2 = TO[tiles[i][j]][d];
					if used[i][j][d].0 > 0 {
						let title = format!("length = {}", used[i][j][d].0);
						let color = if used[i][j][d].1 == 1 {
							"brown"
						} else if used[i][j][d].1 == 2 {
							"darkblue"
						} else {
							"black"
						};
						if d == 0 && d2 == 1 {
							doc = doc.add(Group::new().add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", color)).add(Title::new().add(Text::new(title))));
						} else if d == 1 && d2 == 2 {
							doc = doc.add(Group::new().add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", color).set("transform", format!("rotate(90,{},{})", j * W + W / 2, i * W + W / 2))).add(Title::new().add(Text::new(title))));
						} else if d == 2 && d2 == 3 {
							doc = doc.add(Group::new().add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", color).set("transform", format!("rotate(180,{},{})", j * W + W / 2, i * W + W / 2))).add(Title::new().add(Text::new(title))));
						} else if d == 0 && d2 == 3 {
							doc = doc.add(Group::new().add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", color).set("transform", format!("rotate(270,{},{})", j * W + W / 2, i * W + W / 2))).add(Title::new().add(Text::new(title))));
						} else if tiles[i][j] == 6 {
							doc = doc.add(Group::new().add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail1").set("transform", format!("rotate(90,{},{})", j * W + W / 2, i * W + W / 2)).set("stroke", color)).add(Title::new().add(Text::new(title))));
						} else if tiles[i][j] == 7 {
							doc = doc.add(Group::new().add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail1").set("stroke", color)).add(Title::new().add(Text::new(title))));
						} else {
							unreachable!()
						}
					} else {
						if d == 0 && d2 == 1 {
							doc = doc.add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", "silver"));
						} else if d == 1 && d2 == 2 {
							doc = doc.add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", "silver").set("transform", format!("rotate(90,{},{})", j * W + W / 2, i * W + W / 2)));
						} else if d == 2 && d2 == 3 {
							doc = doc.add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", "silver").set("transform", format!("rotate(180,{},{})", j * W + W / 2, i * W + W / 2)));
						} else if d == 0 && d2 == 3 {
							doc = doc.add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail2").set("stroke", "silver").set("transform", format!("rotate(270,{},{})", j * W + W / 2, i * W + W / 2)));
						} else if tiles[i][j] == 6 {
							doc = doc.add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail1").set("transform", format!("rotate(90,{},{})", j * W + W / 2, i * W + W / 2)).set("stroke", "silver"));
						} else if tiles[i][j] == 7 {
							doc = doc.add(Use::new().set("x", j * W).set("y", i * W).set("href", "#rail1").set("stroke", "silver"));
						} else {
							unreachable!()
						}
					}
				}
			}
		}
	}
	doc.to_string()
}
