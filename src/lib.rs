use rand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    digits: u16,
}

impl Cell {
    pub fn from_digit(d: u16) -> Option<Self> {
        if d == 0 || d > 9 {
            return None;
        }

        Some(Cell { digits: 1 << d })
    }

    pub fn all_digits() -> Self {
        Cell {
            digits: 0b11_1111_1110,
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn len(self) -> u32 {
        self.digits.count_ones()
    }

    pub fn first_digit(self) -> u16 {
        15 - self.digits.leading_zeros() as u16
    }

    pub fn has_digit(self, d: u16) -> bool {
        (self.digits >> d) & 0x1 == 1
    }

    // pub fn add_digit(&mut self, d: u16) {
    //     self.digits |= 1 << d;
    // }

    pub fn remove_digit(&mut self, d: u16) {
        self.digits &= !(1 << d);
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Sudoku {
    cells: [[Cell; 9]; 9],
}

impl Sudoku {
    pub fn from_line(line: &str) -> Option<Self> {
        if line.chars().count() != 81 {
            return None;
        }

        let mut cells = [[Cell { digits: 0 }; 9]; 9];

        for (i, c) in line.chars().enumerate() {
            cells[i / 9][i % 9] = match c {
                '.' => Cell::all_digits(),
                d => Cell::from_digit(d.to_digit(10)? as u16)?,
            };
        }

        Some(Sudoku { cells })
    }

    pub fn to_line(&self) -> String {
        self.cells
            .iter()
            .flat_map(|r| r.iter())
            .map(|c| {
                if c.len() == 1 {
                    c.first_digit().to_string().chars().next().unwrap()
                } else {
                    '.'
                }
            })
            .collect()
    }

    /// Generate a random solvable sudoku with the given number of free cells.
    pub fn generate_solvable(rng: &mut impl Rng, free_cells: usize) -> Option<Sudoku> {
        if free_cells > 81 {
            return None;
        }

        let mut cells_to_choose = (0..81).collect::<Vec<_>>();
        cells_to_choose.shuffle(rng);

        let mut sudoku = Sudoku {
            cells: [[Cell::all_digits(); 9]; 9],
        }
        .first_solution()
        .unwrap();

        for _ in 0..free_cells {
            let i = cells_to_choose.swap_remove(rng.gen_range(0, cells_to_choose.len()));

            sudoku.cells[i / 9][i % 9] = Cell::all_digits();
        }

        Some(sudoku)
    }

    pub fn is_solved(&self) -> bool {
        let is_filled = self.cells.iter().all(|r| r.iter().all(|c| c.len() == 1));
        if !is_filled {
            return false;
        }

        let has_no_duplicates = |cells: [Cell; 9]| {
            let mut digits_set = Cell::all_digits();

            for cell in cells.iter() {
                let d = cell.first_digit();
                digits_set.remove_digit(d);
            }

            digits_set.is_empty()
        };

        let has_valid_rows = (0..9).all(|r| has_no_duplicates(self.row(r)));
        if !has_valid_rows {
            return false;
        }

        let has_valid_cols = (0..9).all(|r| has_no_duplicates(self.col(r)));
        if !has_valid_cols {
            return false;
        }

        let has_valid_quad = (0..9).all(|r| has_no_duplicates(self.quad(r / 3 * 3, r % 3 * 3)));
        if !has_valid_quad {
            return false;
        }

        true
    }

    pub fn first_solution(&self) -> Option<Sudoku> {
        self.solutions().next()
    }

    pub fn solutions(&self) -> impl Iterator<Item = Sudoku> {
        SolutionsIter {
            stack: vec![self.clone()],
        }
    }

    /// Simplify the grid as much as possible by first removing all digits that
    /// cannot be in a position because they must be in another one for sure and
    /// then by searching for digits that can only be in a position. Returns
    /// `None` if the sudoku is impossible, otherwise the simplified sudoku and
    /// whether the sudoku changed or not.
    fn simplified(&self) -> Option<(Sudoku, bool)> {
        let (mut sudoku, mut changed) = self.without_conflicts()?;

        changed = sudoku.find_unambiguities() || changed;

        Some((sudoku, changed))
    }

    /// Create a new sudoku where all the cells do not contain digits that
    /// cannot be there.
    fn without_conflicts(&self) -> Option<(Sudoku, bool)> {
        let mut new = self.clone();
        let mut changed = false;

        for r in 0..9 {
            let row = self.row(r);

            for (c, cell) in row.iter().enumerate() {
                match cell.len() {
                    0 => return None,
                    1 => {
                        let d = cell.first_digit();

                        let mut remove_d_at = |nr: usize, nc: usize| {
                            if nr != r || nc != c {
                                new.cells[nr][nc].remove_digit(d);

                                if new.cells[nr][nc] != self.cells[nr][nc] {
                                    changed = true;
                                }

                                if new.cells[nr][nc].is_empty() {
                                    return None;
                                }
                            }

                            Some(())
                        };

                        let (qr, qc) = self.quad_of(r, c);

                        for i in 0..9 {
                            remove_d_at(r, i)?;
                            remove_d_at(i, c)?;
                            remove_d_at(qr + i / 3, qc + i % 3)?;
                        }
                    }
                    _ => continue,
                };
            }
        }

        Some((new, changed))
    }

    /// Replace each cell with only the digit that can be in that position by
    /// looking at the row, column and quadrant.
    fn find_unambiguities(&mut self) -> bool {
        let mut changed = false;

        for i in 0..9 {
            for d in 1..=9 {
                changed = self.find_unambiguity(d, (0..9).map(|c| (i, c))) || changed;
                changed = self.find_unambiguity(d, (0..9).map(|r| (r, i))) || changed;

                let qr = i / 3 * 3;
                let qc = i % 3 * 3;
                changed =
                    self.find_unambiguity(d, (0..9).map(|i| (qr + i / 3, qc + i % 3))) || changed;
            }
        }

        changed
    }

    fn find_unambiguity(&mut self, d: u16, rng: impl IntoIterator<Item = (usize, usize)>) -> bool {
        let mut changed = false;
        let mut digit_ix = None;

        for (r, c) in rng.into_iter() {
            if !self.cells[r][c].has_digit(d) {
                continue;
            }

            // there's already a fixed cell with this digit therefore
            // there's nothing better we could do
            if self.cells[r][c].len() == 1 {
                digit_ix = None;
                break;
            }

            match digit_ix {
                None => digit_ix = Some((r, c)),
                Some(_) => {
                    digit_ix = None;
                    break;
                }
            };
        }

        if let Some((r, c)) = digit_ix {
            self.cells[r][c] = Cell::from_digit(d).unwrap();
            changed = true;
        }

        changed
    }

    pub fn row(&self, r: usize) -> [Cell; 9] {
        [
            self.cells[r][0],
            self.cells[r][1],
            self.cells[r][2],
            self.cells[r][3],
            self.cells[r][4],
            self.cells[r][5],
            self.cells[r][6],
            self.cells[r][7],
            self.cells[r][8],
        ]
    }

    pub fn col(&self, c: usize) -> [Cell; 9] {
        [
            self.cells[0][c],
            self.cells[1][c],
            self.cells[2][c],
            self.cells[3][c],
            self.cells[4][c],
            self.cells[5][c],
            self.cells[6][c],
            self.cells[7][c],
            self.cells[8][c],
        ]
    }

    pub fn quad_of(&self, r: usize, c: usize) -> (usize, usize) {
        (r / 3 * 3, c / 3 * 3)
    }

    pub fn quad(&self, r: usize, c: usize) -> [Cell; 9] {
        let (qr, qc) = self.quad_of(r, c);

        [
            self.cells[qr][qc],
            self.cells[qr][qc + 1],
            self.cells[qr][qc + 2],
            self.cells[qr + 1][qc],
            self.cells[qr + 1][qc + 1],
            self.cells[qr + 1][qc + 2],
            self.cells[qr + 2][qc],
            self.cells[qr + 2][qc + 1],
            self.cells[qr + 2][qc + 2],
        ]
    }
}

#[derive(Debug)]
struct SolutionsIter {
    stack: Vec<Sudoku>,
}

impl Iterator for SolutionsIter {
    type Item = Sudoku;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut solution = self.stack.pop()?;

            // simplify the sudoku until it settles
            let no_solution = loop {
                match solution.simplified() {
                    None => {
                        break true;
                    }
                    Some((s, changed)) => {
                        solution = s;

                        if !changed {
                            break false;
                        }
                    }
                }
            };

            if no_solution {
                continue;
            }

            if solution.is_solved() {
                return Some(solution);
            }

            // process cells with fewest possible digits first as it's more probable
            // we'll get those right
            let (r, c, cell) = solution
                .cells
                .iter()
                .enumerate()
                .flat_map(|(r, row)| row.iter().enumerate().map(move |(c, cell)| (r, c, *cell)))
                .filter(|(_, _, cell)| cell.len() > 1)
                .min_by_key(|(_, _, cell)| cell.len())
                .unwrap();

            // split the sudoku into two where one has a fixed value for the
            // cell with least possible values and another one that doesn't have
            // that digit as a possible digit in the same cell. Push the latter
            // first in the stack so that it will be processed later, because
            // I think it's more likely to find a solution in the former case.
            let mut candidate = solution.clone();
            candidate.cells[r][c] = cell;
            candidate.cells[r][c].remove_digit(cell.first_digit());
            self.stack.push(candidate);

            let mut candidate = solution.clone();
            candidate.cells[r][c] = Cell::from_digit(cell.first_digit()).unwrap();
            self.stack.push(candidate);
        }
    }
}

impl std::fmt::Debug for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for row in self.cells.iter() {
            for c in row.iter() {
                write!(f, "{:?} ", c)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:010b}", self.digits)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rand_xorshift::XorShiftRng;

    use super::*;

    #[test]
    fn test_solve_1() {
        let sudoku = Sudoku::from_line(
            ".4....179..2..8.54..6..5..8.8..7.91..5..9..3..19.6..4.3..4..7..57.1..2..928....6.",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());

        let sudoku = Sudoku::from_line(
            "8.2.5.7.1..7.8246..1.9.....6....18325.......91843....6.....4.2..9561.3..3.8.9.6.7",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());

        let sudoku = Sudoku::from_line(
            "........772.3.9..1..87.5.6.5.289.....4.5.1.9.....637.5.3.9.61..2..1.7.539........",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());
    }

    #[test]
    fn test_solve_2() {
        let sudoku = Sudoku::from_line(
            "2.6....49.37..9...1..7....6...58.9..7.5...8.4..9.62...9....4..1...3..49.41....2.8",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());

        let sudoku = Sudoku::from_line(
            ".25..7..4..1..5.2.7...2.5..5.9..48.............75..6.9..3.7...6.4.1..7..8..2..91.",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());

        let sudoku = Sudoku::from_line(
            "..1725....8..1...625....13..7....5.....1.6.....9....8..45....297...9..6....6483..",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());
    }

    #[test]
    fn test_solve_3() {
        let sudoku = Sudoku::from_line(
            ".5.2.....3....5.8.96..782......3..2.7.8...1.3.4..8......164..32.7.5....1.....9.5.",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());

        let sudoku = Sudoku::from_line(
            "8..2...46..79.....1.....5.....5...324.8...7.132...7.....6.....9.....32..28...6..3",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());

        let sudoku = Sudoku::from_line(
            "..1725....8..1....25....13..7....5.....186.....9....8..45....29....9..6....6483..",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());
    }

    #[test]
    fn test_solve_4() {
        let sudoku = Sudoku::from_line(
            "346795812258431697971862543129576438835214769764389251517948326493627185682153974",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());

        let sudoku = Sudoku::from_line(
            ".......12..8.3...........4.12.5..........47...6.......5.7...3.....62.......1.....",
        )
        .unwrap();
        assert!(sudoku.first_solution().unwrap().is_solved());
    }

    #[test]
    fn test_solvable_solution_fails_on_too_many_free_cells() {
        assert!(Sudoku::generate_solvable(&mut XorShiftRng::from_seed([0; 16]), 82).is_none());
    }

    proptest! {
        #[test]
        fn random_solvable_solutions_are_solvable(free_cells in 0..82_usize, seed: [u8; 16]) {
            let sudoku = Sudoku::generate_solvable(
                &mut XorShiftRng::from_seed(seed),
                free_cells
            ).unwrap();

            prop_assert!(sudoku.first_solution().unwrap().is_solved());
        }
    }
}
