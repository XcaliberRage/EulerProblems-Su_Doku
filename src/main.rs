#![feature(label_break_value)]

use std::{env, io};
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::io::prelude::*;
use std::fmt::{Display, Formatter, Result, Debug};
use crate::ValueStatus::{Possible, Actual, Impossible};
use crate::GridVector::{Row, Col};
use std::pin::Pin;
use std::ptr::NonNull;

// Solve 50 sudoku puzzles and sum the 3 digit answer in each top left
const SUDOKU_CT: usize = 3;
const GRID_SIZE: usize = 9;
const BOX_SIZE: usize = 3;
const BOX_SIZE_I: u32 = BOX_SIZE as u32;
const GRID_SIZE_I: u32 = GRID_SIZE as u32;

#[derive(Debug, Clone, Copy)]
struct Sudoku<'a> {
    grid: Grid<'a>,
    solved: bool,
    name: u32,
    definite_values: ValueList
}

#[derive(Debug, Clone, Copy)]
struct ValueList {
    one: u32,
    two: u32,
    three: u32,
    four: u32,
    five: u32,
    six: u32,
    seven: u32,
    eight: u32,
    nine: u32,
}

#[derive(Debug, Clone, Copy)]
struct Grid<'a> {
    grid: [[Cell; GRID_SIZE]; GRID_SIZE],
    solved_cell_ct: u32,
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum ValueStatus {
    Actual,
    Possible,
    Impossible,
}

enum GridVector {
    Box,
    Row,
    Col,
    Cel,
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    defined: bool,
    value_as_int: u32,
    one: ValueStatus,
    two: ValueStatus,
    three: ValueStatus,
    four: ValueStatus,
    five: ValueStatus,
    six: ValueStatus,
    seven: ValueStatus,
    eight: ValueStatus,
    nine: ValueStatus,
}

struct SudokusCollection<'a> {
    sudokus: [Sudoku<'a>; SUDOKU_CT],
    solved: bool
}

impl Cell {
    pub fn new(val: u32) -> std::result::Result<Cell, &'static str> {
        match val {
            0 => Ok((Cell {
                defined: false,
                value_as_int: val,
                one: Possible,
                two: Possible,
                three: Possible,
                four: Possible,
                five: Possible,
                six: Possible,
                seven: Possible,
                eight: Possible,
                nine: Possible,
            })),
            1 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Actual,
                two: Impossible,
                three: Impossible,
                four: Impossible,
                five: Impossible,
                six: Impossible,
                seven: Impossible,
                eight: Impossible,
                nine: Impossible,
            })),
            2 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Actual,
                three: Impossible,
                four: Impossible,
                five: Impossible,
                six: Impossible,
                seven: Impossible,
                eight: Impossible,
                nine: Impossible,
            })),
            3 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Impossible,
                three: Actual,
                four: Impossible,
                five: Impossible,
                six: Impossible,
                seven: Impossible,
                eight: Impossible,
                nine: Impossible,
            })),
            4 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Impossible,
                three: Impossible,
                four: Actual,
                five: Impossible,
                six: Impossible,
                seven: Impossible,
                eight: Impossible,
                nine: Impossible,
            })),
            5 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Impossible,
                three: Impossible,
                four: Impossible,
                five: Actual,
                six: Impossible,
                seven: Impossible,
                eight: Impossible,
                nine: Impossible,
            })),
            6 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Impossible,
                three: Impossible,
                four: Impossible,
                five: Impossible,
                six: Actual,
                seven: Impossible,
                eight: Impossible,
                nine: Impossible,
            })),
            7 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Impossible,
                three: Impossible,
                four: Impossible,
                five: Impossible,
                six: Impossible,
                seven: Actual,
                eight: Impossible,
                nine: Impossible,
            })),
            8 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Impossible,
                three: Impossible,
                four: Impossible,
                five: Impossible,
                six: Impossible,
                seven: Impossible,
                eight: Actual,
                nine: Impossible,
            })),
            9 => Ok((Cell {
                defined: true,
                value_as_int: val,
                one: Impossible,
                two: Impossible,
                three: Impossible,
                four: Impossible,
                five: Impossible,
                six: Impossible,
                seven: Impossible,
                eight: Impossible,
                nine: Actual,
            })),
            _ => Err("Can't make a cell with called value!"),
        }
    }


    pub fn get_val(&self) -> std::result::Result<u32, &'static str> {

        let mut impossibles:u32 = 0;

        if self.defined {
            return Ok((self.value_as_int))
        }

        if self.one == Impossible { impossibles += 1; };
        if self.two == Impossible { impossibles += 1; };
        if self.three == Impossible { impossibles += 1; };
        if self.four == Impossible { impossibles += 1; };
        if self.five == Impossible { impossibles += 1; };
        if self.six == Impossible { impossibles += 1; };
        if self.seven == Impossible { impossibles += 1; };
        if self.eight == Impossible { impossibles += 1; };
        if self.nine == Impossible { impossibles += 1; };
        
        if impossibles >= GRID_SIZE_I {
            return Err(("All values marked as impossible!"))
        };
        Ok((0))
    }

    pub fn set_impossible(&mut self, value: u32) {
        match value {
            1 => self.one = ValueStatus::Impossible,
            2 => self.two = ValueStatus::Impossible,
            3 => self.three = ValueStatus::Impossible,
            4 => self.four = ValueStatus::Impossible,
            5 => self.five = ValueStatus::Impossible,
            6 => self.six = ValueStatus::Impossible,
            7 => self.seven = ValueStatus::Impossible,
            8 => self.eight = ValueStatus::Impossible,
            9 => self.nine = ValueStatus::Impossible,
            _ => {}
        }
    }
}

struct Box {
    x: u32,
    y: u32,
}

impl Box {
    pub fn new(x: u32, y: u32) -> Box {
        Box {
            x,
            y
        }
    }

    pub fn x_finder(value: u32) -> u32 {
        match value {
            0 | 3 | 6 => 0,
            1 | 4 | 7 => 3,
            2 | 5 | 8 => 6,
            _ => 0
        }
    }

    pub fn y_finder(value: u32) -> u32 {
        match value {
            0..=2 => 0,
            3..=5 => 3,
            6..=8 => 6,
            _ => 0
        }
    }
}

impl Grid<'_> {
    pub fn new<'a>() -> Grid<'a> {
        Grid {
            grid: [[Cell::new(0).unwrap(); GRID_SIZE]; GRID_SIZE],
            solved_cell_ct: 0,
        }
    }

    // Returns a pointer to the specified location
    pub fn point_val(&mut self, x: u32, y: u32) -> &Cell {
        &self.grid[y as usize][x as usize]
    }

    // Looks at the given cell (by coordinate) and returns a tuple containing the list of possible values and an indicator
    // The indicator is true if only one value is possible
    pub fn get_possibles_by_cell(&mut self, x: u32, y: u32) -> ([u32; GRID_SIZE], bool) {
        let mut possible: [u32; GRID_SIZE] = [1,2,3,4,5,6,7,8,9];
        let y_u = y as usize;
        let x_u = x as usize;

        if self.grid[x as usize][y as usize].defined {
            println!("Cell {},{} defined as {}", x, y, self.grid[x as usize][y as usize].value_as_int);
            return (possible, true)
        }

        for index in 0..GRID_SIZE {

            if self.get_col(x).contains(&possible[index]) {
                possible[index] = 0;
                self.grid[x_u][y_u].set_impossible(possible[index]);
                continue;
            }

            if self.get_row(y).contains(&possible[index]) {
                possible[index] = 0;
                self.grid[x_u][y_u].set_impossible(possible[index]);
                continue;
            }

            if self.get_box(Box::new(x,y)).contains(&possible[index]) {
                possible[index] = 0;
                self.grid[x_u][y_u].set_impossible(possible[index]);
                continue;
            }

        };

        (possible, self.check_for_guarantees(x, y, &possible))
    }

    // Takes one value and determines possible placement within a specific neighbourhood (row, column or box)
    // It identifies if the value is already in the neighbourhood,
    // in which case it eliminates the possibility from all cells in that neighbourhood
    pub fn get_possibles_by_neighbourhood(&mut self, value: u32, vector: GridVector, coord: u32) {

        let neighbourhood = match vector {
            GridVector::Col => self.get_col(coord),
            GridVector::Row => self.get_row(coord),
            GridVector::Box => self.get_box(Box::new(Box::x_finder(coord), Box::y_finder(coord))),
            _ => [0; GRID_SIZE]
        };

        if !neighbourhood.contains(&value) {
            return;
        }

        let coord_u = coord as usize;
        match vector {
            GridVector::Col => {
                for i in 0..GRID_SIZE {
                    if self.grid[i][coord_u].defined {
                        continue;
                    }
                    self.grid[i][coord_u].set_impossible(value);
                }
            },
            GridVector::Row => {
                for i in 0..GRID_SIZE {
                    if self.grid[coord_u][i].defined {
                        continue;
                    }
                    self.grid[coord_u][i].set_impossible(value);
                }
            },
            GridVector::Box => {
                let x_offset = Box::x_finder(coord) as usize;
                let y_offset = Box::y_finder(coord) as usize;

                for y in y_offset..(y_offset + BOX_SIZE) {
                    for x in x_offset..(x_offset + BOX_SIZE) {
                        self.grid[x][y].set_impossible(value);
                    }
                }
            },
            _ => {}
        }

    }

    // Checks to identify if there is only one possible value for the cell and sets it if so
    // Returns true if this is done
    pub fn check_for_guarantees(&mut self, x: u32, y: u32, possible: &[u32; GRID_SIZE]) -> bool {
        let mut ct: u32= 0;
        let mut last_val = 0;
        for poss in possible.iter() {
            if *poss != 0 {
                ct+= 1;
                last_val = *poss;
            }
        }
        if ct == 1 {
            println!("Guaranteed value found at {}, {}: {}", x, y, last_val);
            self.set_value(x, y, last_val);
            return true
        }

        return false
    }

    pub fn analyse(&mut self) {
        for x in 0..GRID_SIZE_I {
            for y in 0..GRID_SIZE_I {
                if self.grid[x as usize][y as usize].defined {
                    continue;
                }
                self.get_possibles_by_cell(x, y);
            }
        }

        for number in 1..(GRID_SIZE_I+1) {
            if *self.get_val_by_int(number) >= GRID_SIZE_I {
                continue;
            }
            for coord in 0..GRID_SIZE_I {
                self.get_possibles_by_neighbourhood(number, GridVector::Row, coord);
                self.get_possibles_by_neighbourhood(number, GridVector::Col, coord);
                self.get_possibles_by_neighbourhood(number, GridVector::Box, coord);
            }
        }
    }

    pub fn get_val_by_int(&self, value: u32) -> &u32 {
        match value {
            1 => &self.sudoku.definite_values.one,
            2 => &self.sudoku.definite_values.two,
            3 => &self.sudoku.definite_values.three,
            4 => &self.sudoku.definite_values.four,
            5 => &self.sudoku.definite_values.five,
            6 => &self.sudoku.definite_values.six,
            7 => &self.sudoku.definite_values.seven,
            8 => &self.sudoku.definite_values.eight,
            9 => &self.sudoku.definite_values.nine,
            _ => &1
        }
    }

    pub fn set_value(&mut self, x: u32, y: u32, value: u32) {
        println!("Setting cell {},{} as {}", x, y, value);
        self.grid[x as usize][y as usize] = Cell::new(value).unwrap();

        // TODO Increment each time a value is defined somewhere
    }

    pub fn get_solved(&mut self) -> u32 {
        for col in self.grid.iter() {
            for row in col.iter() {
                if row.defined {
                    self.solved_cell_ct += 1;
                }
            }
        }

        self.solved_cell_ct
    }

    // Returns all values found in the given column
    fn get_col(&self, x: u32) -> [u32; GRID_SIZE] {
        let x_size = x as usize;
        let mut numbers: [u32; GRID_SIZE] = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            let value = &self.grid[x_size][row].value_as_int;
            if *value != 0 {
                numbers[*value as usize - 1] = *value;
                continue;
            }
        };
        numbers
    }

    // Returns all values found in the given row
    fn get_row(&self, y: u32) -> [u32; GRID_SIZE] {
        let y_size = y as usize;
        let mut numbers: [u32; GRID_SIZE] = [0;GRID_SIZE];
        for col in 0..GRID_SIZE {
            let value = &self.grid[col][y_size].get_val().unwrap();
            if *value != 0 {
                numbers[*value as usize - 1] = *value;
                continue;
            }
        };
        numbers
    }

    // Returns all values found in the given box
    fn get_box(&self, g_box: Box) -> [u32; GRID_SIZE] {
        let left_col = match g_box.x {
            0..=2 => Ok((0)),
            3..=5 => Ok((3)),
            6..=8 => Ok((6)),
            _ => Err(("X Co-ordinate given out of bounds")),
        };

        let top_row = match g_box.y {
            0..=2 => Ok((0)),
            3..=5 => Ok((3)),
            6..=8 => Ok((6)),
            _ => Err(("X Co-ordinate given out of bounds")),
        };

        let mut numbers: [u32; GRID_SIZE] = [0; GRID_SIZE];

        for row in 0..3 {
            for col in 0..3 {
                let val = &self.grid[col+left_col.unwrap()][row+top_row.unwrap()].get_val().unwrap();
                if *val != 0 {
                    numbers[*val as usize - 1] = *val;
                    continue;
                }
            }
        };
        numbers
    }
}

impl Sudoku<'_> {
    pub fn new<'a>(name: u32) -> Self<'a> {
        Self {
            grid: Grid::new(),
            solved: false,
            name,
            definite_values: ValueList {
                one: 0,
                two: 0,
                three: 0,
                four: 0,
                five: 0,
                six: 0,
                seven: 0,
                eight: 0,
                nine: 0
            }
        }
    }

    pub fn check_solved(&mut self) -> bool {
        for row in self.grid.grid.iter() {
            for col in row.iter() {
                if !col.defined {
                    return false;
                }
            }
        }
        self.solved = true;
        true
    }
}

impl SudokusCollection<'_> {
    pub fn new(collection: [Sudoku; SUDOKU_CT]) -> SudokusCollection {
        SudokusCollection {
            sudokus: collection,
            solved: false
        }
    }

    pub fn print(&self, param: &str) {
        for sudoku in self.sudokus.iter() {
            if !sudoku.solved && param == "solved" {
                continue;
            }
            if sudoku.solved && param == "unsolved" {
                continue;
            }

            println!("\nSudoku {}:", sudoku.name);
            for line in sudoku.grid.grid.iter() {
                let vals = line.iter().map(|c| c.get_val().unwrap()).collect::<Vec<u32>>();
                println!("{:?}", vals);
            }
        }
    }

    pub fn analyse(&mut self) {
        let mut solved_all = true;
        for sudoku in self.sudokus.iter_mut() {
            if !sudoku.solved {
                sudoku.grid.analyse();
                if solved_all {
                    solved_all = sudoku.check_solved();
                }
            }
        }
        self.solved = solved_all;
    }
}

fn main() -> std::io::Result<()> {

    let path = Path::new("sudoku.txt");
    let mut file = File::open(&path)?;
    let mut sudokus = get_sudokus(&file);

    let mut buffer = String::new();
    let mut stdin = io::stdin();

    sudokus.print("all");
    println!("Ready to solve?");
    stdin.read_line(&mut buffer).expect("Error reading");

    'main: while !sudokus.solved {
        sudokus.print("unsolved");
        sudokus.analyse();

        println!("Keep going? [Y/N]");
        stdin.read_line(&mut buffer).expect("Failed to understand input");
        if buffer.trim().to_lowercase() == "n" {
            break 'main;
        }
    }

    sudokus.print("all");

    Ok(())
}

// Parses a text file into an array of sudokus to solve
fn get_sudokus(file: &File) -> SudokusCollection {

    let mut sudokus = [
        Sudoku::new(0); SUDOKU_CT
    ];

    let mut reader = BufReader::new(file);
    let mut true_line = 0; // Tracks the actual line of the file
    let mut inner_line: usize = 0; // Tracks the line for the individual Sudoku read
    let mut sudoku_num: usize = 0; // Tracks the current sudoku

    for line in reader.lines() {
        if true_line > (SUDOKU_CT * 10)-1 {
            println!("All done!");
            return SudokusCollection::new(sudokus)
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
            sudokus[sudoku_num].grid.grid[inner_line-1][pos] = Cell::new(line_as_string
                .get(pos..pos+1)
                .unwrap()
                .parse::<u32>()
                .unwrap())
                .unwrap();
        }
        sudokus[sudoku_num].name = (sudoku_num + 1) as u32;

        inner_line += 1;
    }

    SudokusCollection::new(sudokus)
}