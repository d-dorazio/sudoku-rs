use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;
use structopt::StructOpt;

use sudoku_rs::Sudoku;

/// Just a sudoku solver and generator.
#[derive(StructOpt, Debug)]
enum Cmd {
    /// Generate a given number of solvable sudoku. Uniqueness is not enforced.
    #[structopt(name = "generate")]
    Generate {
        /// How many free cells each generated sudoku should have.
        #[structopt(short = "f", long = "free-cells")]
        free_cells: usize,

        /// How many sudoku to generate.
        #[structopt(short = "c", long = "count")]
        count: usize,
    },

    /// Solve all the sudoku from the given file or input printing the total
    /// time elapsed.
    #[structopt(name = "solve")]
    Solve {
        /// Solve each sudoku in parallel.
        #[structopt(short = "p", long = "parallel")]
        parallel: bool,

        /// Be more verbose.
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,

        #[structopt(parse(from_os_str))]
        sudoku: Option<PathBuf>,
    },
}

fn main() -> io::Result<()> {
    let cmd = Cmd::from_args();

    let stdout = io::stdout();
    let stdout = stdout.lock();

    match cmd {
        Cmd::Generate { free_cells, count } => generate_sudoku(free_cells, count, stdout),
        Cmd::Solve {
            verbose,
            parallel,
            sudoku: Some(p),
        } => {
            let f = File::open(p)?;
            solve_sudoku(parallel, f, stdout, verbose)
        }
        Cmd::Solve {
            verbose,
            parallel,
            sudoku: None,
        } => {
            let stdin = io::stdin();
            let stdin = stdin.lock();
            solve_sudoku(parallel, stdin, stdout, verbose)
        }
    }
}

fn solve_sudoku(
    parallel: bool,
    r: impl Read,
    mut out: impl Write,
    verbose: bool,
) -> io::Result<()> {
    let buf = BufReader::new(r);

    let sudoku = buf
        .lines()
        .map(|l| Sudoku::from_line(&l.unwrap()).unwrap())
        .collect::<Vec<_>>();

    let start_t = Instant::now();

    let sudoku_fn = |i: usize, s: Sudoku| {
        let solution = s.first_solution();
        let is_solved = solution.map_or(false, |s| s.is_solved());

        if !is_solved {
            panic!(
                "all input sudoku should be solvable but #{} is not: {:?}",
                i, s
            );
        }

        is_solved
    };

    match (parallel, verbose) {
        (true, _) => {
            sudoku.into_par_iter().enumerate().for_each(|(i, s)| {
                sudoku_fn(i, s);
            });
        }
        (false, false) => {
            for (i, sudoku) in sudoku.into_iter().enumerate() {
                sudoku_fn(i, sudoku);
            }
        }
        (false, true) => {
            for (i, sudoku) in sudoku.into_iter().enumerate() {
                let is_solved = sudoku_fn(i, sudoku);

                writeln!(out, "#{} is solved {:?}", i, is_solved)?;
            }
        }
    }

    writeln!(out, "total time elapsed {:?}", start_t.elapsed())
}

fn generate_sudoku(free_cells: usize, count: usize, mut out: impl Write) -> io::Result<()> {
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let sudoku = Sudoku::generate_solvable(&mut rng, free_cells)
            .expect("cannot create a solvable sudoku");

        writeln!(out, "{}", sudoku.to_line())?;
    }

    Ok(())
}
