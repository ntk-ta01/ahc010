use proconio::{input, marker::Chars};
use rand::prelude::*;
#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

type Output = Vec<i32>;

const TIMELIMIT: f64 = 1.97;
const N: usize = 30;
const DIJ: [(usize, usize); 4] = [(0, !0), (!0, 0), (0, 1), (1, 0)];
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

#[derive(Clone, Debug)]
struct Input {
    tiles: Vec<Vec<usize>>,
}

fn parse_input() -> Input {
    input! {
        tiles: [Chars; N]
    }
    let tiles = tiles
        .iter()
        .map(|ts| ts.iter().map(|&c| (c as u8 - b'0') as usize).collect())
        .collect();
    Input { tiles }
}

fn main() {
    let mut timer = Timer::new();
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(93216000);
    let input = parse_input();
    let mut out = vec![0; N * N];
    annealing(&input, &mut out, &mut timer, &mut rng);
    println!("{}", out.iter().map(|&n| n.to_string()).collect::<String>());
    // eprintln!("{:?}", compute_score(&input, &out).1 .1);
}

#[allow(dead_code)]
fn initial_sol() {}

fn annealing(
    input: &Input,
    output: &mut Output,
    timer: &mut Timer,
    rng: &mut rand_chacha::ChaCha20Rng,
    // s_temp: f64,
    // e_temp: f64,
) -> i64 {
    const T0: f64 = 100.0;
    const T1: f64 = 0.00001;
    let mut temp = T0;
    // let mut temp = s_temp;
    let mut prob;

    let mut count = 0;
    let (mut now_score, (_, _), mut total_length) = compute_score2(input, output);

    let mut best_score = now_score;
    let mut best_output = output.clone();
    const NEIGH_COUNT: i32 = 2;
    loop {
        if count >= 100 {
            // let now = timer.get_time();
            // let passed = now / TIMELIMIT;
            let passed = timer.get_time() / TIMELIMIT;
            if passed >= 1.0 {
                break;
            }
            // eprintln!("{} {}", temp, now_score);
            temp = T0.powf(1.0 - passed) * T1.powf(passed);
            // temp = s_temp.powf(1.0 - passed) * e_temp.powf(passed);
            count = 0;
            // eprintln!("{} {}", now, best_score);
        }
        count += 1;

        let mut new_out = output.clone();
        let neigh_type = rng.gen_range(0, NEIGH_COUNT);
        match neigh_type {
            0 => {
                // update
                let update_index = rng.gen_range(0, new_out.len());
                let update_rotate = rng.gen_range(0, 3);
                new_out[update_index] = update_rotate;
            }
            1 => {
                // update 複数
                for _ in 0..90 {
                    let update_index = rng.gen_range(0, new_out.len());
                    let update_rotate = rng.gen_range(0, 3);
                    new_out[update_index] = update_rotate;
                }
            }
            _ => unreachable!(),
        }
        let (new_score, (_, _), new_total_length) = compute_score2(input, &new_out);
        prob = f64::exp(
            (new_score * new_total_length * new_total_length * new_total_length
                - now_score * total_length * total_length * total_length) as f64
                / temp,
        );
        if now_score < new_score || rng.gen_bool(prob) {
            now_score = new_score;
            total_length = new_total_length;
            // tiles = new_tiles;
            // cycle = new_cycle;
            *output = new_out;
        }

        if best_score < now_score {
            best_score = now_score;
            best_output = output.clone();
            // println!(
            //     "{}",
            //     best_output
            //         .iter()
            //         .map(|&n| n.to_string())
            //         .collect::<String>()
            // );
        }
    }
    eprintln!("{}", best_score);
    *output = best_output;
    best_score
}

#[allow(dead_code)]
fn search_cycle_around(tiles: &[Vec<usize>], cycle: &[Vec<Vec<(i32, i32)>>]) -> Vec<usize> {
    let mut around = vec![];
    for i in 0..N {
        for j in 0..N {
            for d in 0..4 {
                if TO[tiles[i][j]][d] != !0 && d < TO[tiles[i][j]][d] {
                    // let d2 = TO[tiles[i][j]][d];
                    if cycle[i][j][d].1 == 1 || cycle[i][j][d].1 == 2 {
                        for &(di, dj) in DIJ.iter() {
                            let ni = i + di;
                            let nj = j + dj;
                            if 30 * ni + nj < N * N {
                                around.push(30 * ni + nj)
                            }
                        }
                    }
                }
            }
        }
    }
    around
}

#[allow(clippy::type_complexity)]
fn compute_score(
    input: &Input,
    out: &Output,
) -> (i64, (Vec<Vec<usize>>, Vec<Vec<Vec<(i32, i32)>>>)) {
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
    (score as i64, (tiles, cycle))
}

#[allow(clippy::type_complexity)]
fn compute_score2(
    input: &Input,
    out: &Output,
) -> (i64, (Vec<Vec<usize>>, Vec<Vec<Vec<(i32, i32)>>>), i64) {
    let mut tiles = input.tiles.clone();
    for i in 0..N {
        for j in 0..N {
            for _ in 0..out[i * N + j] {
                tiles[i][j] = ROTATE[tiles[i][j]];
            }
        }
    }
    let mut total_length = 0;
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
                        total_length += 1;
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
    (score as i64, (tiles, cycle), total_length)
}

fn get_time() -> f64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9
}

struct Timer {
    start_time: f64,
}

impl Timer {
    fn new() -> Timer {
        Timer {
            start_time: get_time(),
        }
    }

    fn get_time(&self) -> f64 {
        get_time() - self.start_time
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.start_time = 0.0;
    }
}
