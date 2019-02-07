use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;

use sudoku_rs::Sudoku;

fn main() -> io::Result<()> {
    let fp = env::args()
        .nth(1)
        .expect("please provide an input sudoku archive");

    let f = File::open(fp)?;
    let buf = BufReader::new(f);

    let sudokus = buf
        .lines()
        .map(|l| Sudoku::from_line(&l.unwrap()).unwrap())
        .collect::<Vec<_>>();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let start_t = Instant::now();
    for (i, sudoku) in sudokus.into_iter().enumerate() {
        let solution = sudoku.first_solution();
        let is_solved = solution.map_or(false, |s| s.is_solved());

        writeln!(stdout, "#{} is solved {:?}", i, is_solved)?;

        if !is_solved {
            panic!(
                "all input sudoku should be solvable but #{} is not: {:?}",
                i, sudoku
            );
        }
    }

    println!("total time elapsed {:?}", start_t.elapsed());

    Ok(())
}
