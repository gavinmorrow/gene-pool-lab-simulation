use rand::Rng;
use rand::seq::SliceRandom;
use std::sync::mpsc;
use std::thread;

const NUM_POOLS: usize = 20;
const GENS_PER_POOL: i32 = 100_000;
const TOTAL_ALLELES: i32 = 100;
const MALARIA_SURVIVAL: f64 = 0.5;
const DEBUG: bool = false;

fn main() {
    let (tx, rx) = mpsc::channel();

    let mut children = Vec::new();
    for _ in 0..NUM_POOLS {
        let tx = tx.clone();
        let child = thread::spawn(move || {
            tx.send(run_simulation(GENS_PER_POOL)).unwrap();
        });
        children.push(child);
    }

    let mut totals = Vec::with_capacity(NUM_POOLS as usize);
    for _ in 0..NUM_POOLS {
        totals.push(rx.recv().unwrap());
    }

    for child in children {
        child.join().unwrap();
    }

    println!();

    let (totals_a, totals_s) = totals.into_iter().unzip();

    // Get median
    let median_a = median(&totals_a);
    let median_s = median(&totals_s);
    println!("  med | A: {}, S: {}", median_a, median_s);

    // Get average
    let avg_a: f64 = totals_a.iter().sum::<i32>() as f64 / totals_a.len() as f64;
    let avg_s: f64 = totals_s.iter().sum::<i32>() as f64 / totals_s.len() as f64;
    println!("  avg | A: {}, S: {}", avg_a, avg_s);

    // Get min/max/range
    let min_a = totals_a.iter().min().expect("has at least one element");
    let min_s = totals_s.iter().min().expect("has at least one element");
    let max_a = totals_a.iter().max().expect("has at least one element");
    let max_s = totals_s.iter().max().expect("has at least one element");
    let range_a = max_a - min_a;
    let range_s = max_s - min_s;
    println!("  min | A: {}, S: {}", min_a, min_s);
    println!("  max | A: {}, S: {}", max_a, max_s);
    println!("range | A: {}, S: {}", range_a, range_s);
}

fn median(data: &Vec<i32>) -> f64 {
    let mut data = data.clone();
    data.sort();
    if data.len() % 2 == 0 {
        // Get average of middle two
        let a = data[data.len() / 2 - 1];
        let b = data[data.len() / 2];
        (a + b) as f64 / 2_f64
    } else {
        data[data.len() / 2] as f64
    }
}

fn run_simulation(generations: i32) -> (i32, i32) {
    let mut alleles = generate_alleles(75, 25);
    for gen_num in 0..generations {
        let gen = run_generation(alleles);
        let (percent_a, percent_s, real_a, real_s) = parse_alleles(&gen);

        if DEBUG {
            println!(
                "gen {} | {}",
                gen_num,
                parsed_to_string((percent_a, percent_s, real_a, real_s))
            );
        }

        alleles = generate_alleles(percent_a, percent_s);
    }
    let parsed = parse_alleles(&alleles);
    println!("total | {}", parsed_to_string(parsed));

    (parsed.0, parsed.1)
}

fn run_generation(alleles: Vec<Allele>) -> Vec<Allele> {
    let genes = get_genes(alleles);
    assert_eq!(genes.len(), (TOTAL_ALLELES / 2) as usize);

    let parsed = parse_alleles(
        &genes
            .clone()
            .into_iter()
            .flat_map(|(a, s)| vec![a, s])
            .collect(),
    );
    if DEBUG {
        println!("> {}", parsed_to_string(parsed));
    }

    let mut rng = rand::thread_rng();

    genes
        .into_iter()
        .filter(|(a, b)| match (a, b) {
            (Allele::A, Allele::A) => rng.gen::<f64>() < MALARIA_SURVIVAL,
            (Allele::A, Allele::S) | (Allele::S, Allele::A) => true,
            (Allele::S, Allele::S) => false,
        })
        .flat_map(|(a, b)| vec![a, b])
        .collect()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Allele {
    A,
    S,
}

fn parse_alleles(alleles: &Vec<Allele>) -> (i32, i32, i32, i32) {
    let real_a = alleles
        .into_iter()
        .filter(|&&allele| allele == Allele::A)
        .count() as i32;
    let real_s = alleles
        .into_iter()
        .filter(|&&allele| allele == Allele::S)
        .count() as i32;

    assert_eq!(real_a + real_s, alleles.len() as i32);

    let percent_a = real_a as f32 / alleles.len() as f32;
    let percent_s = real_s as f32 / alleles.len() as f32;

    let new_a = (percent_a * 100.0) as i32;
    let new_s = (percent_s * 100.0) as i32;

    if (new_a + new_s) == 100 {
        (new_a, new_s, real_a, real_s)
    } else {
        // println!("{}, {}", new_a, new_s);
        if rand::random() {
            (new_a + 1, new_s, real_a, real_s)
        } else {
            (new_a, new_s + 1, real_a, real_s)
        }
    }
}

fn parsed_to_string(parsed: (i32, i32, i32, i32)) -> String {
    format!(
        "A: {} ({}%), S: {} ({}%)",
        parsed.2, parsed.0, parsed.1, parsed.3
    )
}

fn generate_alleles(a: i32, s: i32) -> Vec<Allele> {
    assert_eq!(a + s, TOTAL_ALLELES);
    vec![Allele::A; a as usize]
        .into_iter()
        .chain(vec![Allele::S; s as usize])
        .collect()
}

fn get_genes(mut alleles: Vec<Allele>) -> Vec<(Allele, Allele)> {
    // let mut alleles = alleles.collect::<Vec<_>>();
    assert_eq!(alleles.len() % 2, 0);
    alleles.shuffle(&mut rand::thread_rng());
    alleles
        .chunks(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect()
}
