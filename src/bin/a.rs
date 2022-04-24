use itertools::Itertools;
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

const TIMELIMIT: f64 = 1.94;
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
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(7_300_000_000);
    let input = parse_input();
    let mut out = vec![0; N * N];
    // let mut out = (0..N * N)
    //     .into_iter()
    //     .map(|_| rng.gen_range(0, 4))
    //     .collect_vec();
    annealing(&input, &mut out, &mut timer, &mut rng);
    println!("{}", out.iter().map(|&n| n.to_string()).collect::<String>());
    // eprintln!("{}", compute_score(&input, &out).0);
}

fn annealing(
    input: &Input,
    output: &mut Output,
    timer: &mut Timer,
    rng: &mut rand_chacha::ChaCha20Rng,
    // s_temp: f64,
    // e_temp: f64,
) -> i64 {
    const T0: f64 = 100.0;
    const T1: f64 = 0.01;
    let mut temp = T0;
    // let mut temp = s_temp;
    let mut prob;

    let mut count = 0;
    let mut now_score = compute_score(input, output).0;

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
        let new_score = compute_score(input, &new_out).0;
        prob = f64::exp((new_score - now_score) as f64 / temp);
        if now_score < new_score || rng.gen_bool(prob) {
            now_score = new_score;
            *output = new_out;
        }

        if best_score < now_score {
            best_score = now_score;
            best_output = output.clone();
        }
    }
    eprintln!("{}", best_score);
    *output = best_output;
    best_score
}

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
