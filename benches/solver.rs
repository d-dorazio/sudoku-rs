use criterion::{criterion_group, criterion_main, Criterion};

use sudoku_rs::Sudoku;

fn solve_sudoku(c: &mut Criterion) {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let f = File::open("data/sudoku17.txt").unwrap();
    let f = BufReader::new(f);
    let sudoku = f
        .lines()
        .map(|l| Sudoku::from_line(&l.unwrap()).unwrap())
        .take(100)
        .collect::<Vec<_>>();

    c.bench_function("solve all sudoku solutions", move |b| {
        b.iter(|| {
            for s in &sudoku {
                let ok = s.solutions().all(|s| s.is_solved());
                if !ok {
                    panic!();
                }
            }
        })
    });
}

criterion_group!(benches, solve_sudoku);
criterion_main!(benches);
