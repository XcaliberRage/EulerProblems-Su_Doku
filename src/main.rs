use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::io::prelude::*;
use std::fmt::{Display, Formatter, Result, Debug};

// Solve 50 sudoku puzzles and sum the 3 digit answer in each top left
const SUDOKU_CT: usize = 1;
const GRID_SIZE: usize = 9;
const GRID_SIZE_I: u32 = GRID_SIZE as u32;

#[derive(Debug, Clone, Copy)]
struct Sudoku {
    grid: Grid,
    solved: bool,
}

struct Grid {
    grid: [[u32; GRID_SIZE]; GRID_SIZE]
}

struct Box {
    x: u32,
    y: u32,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            grid: [[0; GRID_SIZE]; GRID_SIZE]
        }
    }

    // Returns a pointer to the specified location
    pub fn point_val(&self, x: u32, y: u32) -> &u32 {
        &self.grid[y][x]
    }

    pub fn get_possibles(&self, x: u32, y: u32) -> [u32; GRID_SIZE] {
        let value = self.point_val(x,y);
        let mut possible: [u32; GRID_SIZE] = [1,2,3,4,5,6,7,8,9];

        for index in 0..GRID_SIZE {
            possible[index] = if self.get_col(y).contains(&possible[index]) {0} else {possible[index]};
        };

        possible
    }

    // Returns all values found in the given column
    fn get_col(&self, y: u32) -> [u32; GRID_SIZE] {}

    // Returns all values found in the given row
    fn get_row(&self, x: u32) -> [u32; GRID_SIZE] {}

    // Returns all values found in the given box
    fn get_box(&self, g_box: Box) -> [u32; GRID_SIZE] {}
}

impl Sudoku {
    pub fn new() -> Sudoku {
        Sudoku {
            grid: Grid::new(),
            solved: false,
        }
    }

    // Solve this sudoku!
    pub fn solve(&mut self) {

    }
}

fn main() -> std::io::Result<()> {

    let path = Path::new("sudoku.txt");
    let display = path.display();

    let mut file = File::open(&path)?;

    let mut sudokus = get_sudokus(&file);
    let mut ct = 0;

    for sudoku in sudokus.iter() {
        println!("Sudoku {}:", ct+1);
        ct += 1;
        for line in sudoku.grid.iter() {
            println!("{:?}", line);
        }
    }

    Ok(())
}

// Parses a text file into an array of Sudokus to solve
fn get_sudokus(file: &File) -> [Sudoku; SUDOKU_CT] {

    let mut sudokus = [
        Sudoku::new(); SUDOKU_CT
    ];

    let mut reader = BufReader::new(file);
    let mut true_line = 0; // Tracks the actual line of the file
    let mut inner_line: usize = 0; // Tracks the line for the individual Sudoku read
    let mut sudoku_num: usize = 0; // Tracks the current sudoku

    for line in reader.lines() {
        if true_line > (SUDOKU_CT * 10)-1 {
            println!("All done!");
            return sudokus
        }

        let line_as_string = line.unwrap();

        if line_as_string.starts_with('G') && true_line > 0 {
            println!("Reset line ct");
            inner_line = 0;
            sudoku_num += 1;
        };

        println!("Line {}({}) is {}",inner_line ,true_line, line_as_string);
        true_line += 1;

        if inner_line == 0 {
            println!("Skip header");
            inner_line += 1;
            continue;
        }

        for pos in 0..GRID_SIZE {
            sudokus[sudoku_num].grid[inner_line-1][pos] = line_as_string
                .get(pos..pos+1)
                .unwrap()
                .parse::<u32>()
                .unwrap();
        }

        inner_line += 1;
    }

    sudokus
}