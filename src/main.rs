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
use std::collections::HashMap;

// Solve 50 sudoku puzzles and sum the 3 digit answer in each top left
const SUDOKU_CT: usize = 1;
const GRID_SIZE: usize = 9;
const BOX_SIZE: usize = 3;
const BOX_SIZE_I: u32 = BOX_SIZE as u32;
const GRID_SIZE_I: u32 = GRID_SIZE as u32;

#[derive(Debug, Clone, Copy)]
struct Sudoku {
    grid: Grid,
    solved: bool,
    name: u32,
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
struct Grid {
    grid: [[Cell; GRID_SIZE]; GRID_SIZE],
    solved_cell_ct: u32,
    definite_values: ValueList,
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
struct Coordinate {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    defined: bool,
    value_as_int: u32,
    one: (ValueStatus, u32),
    two: (ValueStatus, u32),
    three: (ValueStatus, u32),
    four: (ValueStatus, u32),
    five: (ValueStatus, u32),
    six: (ValueStatus, u32),
    seven: (ValueStatus, u32),
    eight: (ValueStatus, u32),
    nine: (ValueStatus, u32),
    coordinate: Coordinate,
}

struct SudokusCollection {
    sudokus: [Sudoku; SUDOKU_CT],
    solved: bool
}

impl Cell {
    pub fn new(val: u32, x: u32, y: u32) -> std::result::Result<Cell, &'static str> {

        let mut cell = Cell {
            defined: true,
            value_as_int: val,
            one: (Impossible, 1),
            two: (Impossible, 2),
            three: (Impossible, 3),
            four: (Impossible, 4),
            five: (Impossible, 5),
            six: (Impossible, 6),
            seven: (Impossible, 7),
            eight: (Impossible, 8),
            nine: (Impossible, 9),
            coordinate: Coordinate {x, y}
        };

        match val {
            0 => {
                cell.one.0 = ValueStatus::Possible;
                cell.two.0 = ValueStatus::Possible;
                cell.three.0 = ValueStatus::Possible;
                cell.four.0 = ValueStatus::Possible;
                cell.five.0 = ValueStatus::Possible;
                cell.six.0 = ValueStatus::Possible;
                cell.seven.0 = ValueStatus::Possible;
                cell.eight.0 = ValueStatus::Possible;
                cell.nine.0 = ValueStatus::Possible;
                cell.defined = false;
                return Ok((cell))
            },
            1 => {
                cell.one.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            2 => {
                cell.two.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            3 => {
                cell.three.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            4 => {
                cell.four.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            5 => {
                cell.five.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            6 => {
                cell.six.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            7 => {
                cell.seven.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            8 => {
                cell.eight.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            9 => {
                cell.nine.0 = ValueStatus::Actual;
                return Ok((cell))
            },
            _ => Err("Can't make a cell with called value!"),
        }
    }


    pub fn get_val(&self) -> std::result::Result<u32, &'static str> {

        let mut impossibles:u32 = 0;

        if self.defined {
            return Ok((self.value_as_int))
        }

        if self.one.0 == Impossible { impossibles += 1; };
        if self.two.0 == Impossible { impossibles += 1; };
        if self.three.0 == Impossible { impossibles += 1; };
        if self.four.0 == Impossible { impossibles += 1; };
        if self.five.0 == Impossible { impossibles += 1; };
        if self.six.0 == Impossible { impossibles += 1; };
        if self.seven.0 == Impossible { impossibles += 1; };
        if self.eight.0 == Impossible { impossibles += 1; };
        if self.nine.0 == Impossible { impossibles += 1; };

        if impossibles >= GRID_SIZE_I {
            return Err(("All values marked as impossible!"))
        };
        Ok((0))
    }

    pub fn set_impossible(&mut self, value: u32) {
        println!("Setting value {} Impossible at {:?}, new status:", value, self.coordinate);
        println!("{}", self);

        match value {
            1 => self.one.0 = ValueStatus::Impossible,
            2 => self.two.0 = ValueStatus::Impossible,
            3 => self.three.0 = ValueStatus::Impossible,
            4 => self.four.0 = ValueStatus::Impossible,
            5 => self.five.0 = ValueStatus::Impossible,
            6 => self.six.0 = ValueStatus::Impossible,
            7 => self.seven.0 = ValueStatus::Impossible,
            8 => self.eight.0 = ValueStatus::Impossible,
            9 => self.nine.0 = ValueStatus::Impossible,
            _ => {}
        }

        self.check_guarantee();
    }

    pub fn check_guarantee(&mut self) {

        if self.defined {
            return;
        }

        let mut possible_ct: u32 = 0;
        let mut last = &mut (ValueStatus::Actual, 0);

        if self.one.0 == ValueStatus::Possible {
            last = &mut self.one;
        }
        if self.two.0 == ValueStatus::Possible {
            last = &mut self.two;
        }
        if self.three.0 == ValueStatus::Possible {
            last = &mut self.three;
        }
        if self.four.0 == ValueStatus::Possible {
            last = &mut self.four;
        }
        if self.five.0 == ValueStatus::Possible {
            last = &mut self.five;
        }
        if self.six.0 == ValueStatus::Possible {
            last = &mut self.six;
        }
        if self.seven.0 == ValueStatus::Possible {
            last = &mut self.seven;
        }
        if self.eight.0 == ValueStatus::Possible {
            last = &mut self.eight;
        }
        if self.nine.0 == ValueStatus::Possible {
            last = &mut self.nine;
        }

        if possible_ct > 1 {
            return
        }

        self.value_as_int = last.1;
        println!("New guaranteed value ({}) determined at {:?}!",self.value_as_int ,self.coordinate);
        last.0 = ValueStatus::Actual;
        self.defined = true;
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\
        Defined: {}\n\
        Value: {}\n\
        {:?}\n\
        Possibility:\n\
            1: {:?}\n\
            2: {:?}\n\
            3: {:?}\n\
            4: {:?}\n\
            5: {:?}\n\
            6: {:?}\n\
            7: {:?}\n\
            8: {:?}\n\
            9: {:?}\n",
        self.defined, self.value_as_int, self.coordinate,
               self.one, self.two, self.three, self.four, self.five, self.six, self.seven, self.eight, self.nine)
    }
}

#[derive(Debug, Clone, Copy)]
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

impl Grid {
    pub fn new() -> Grid {
        Grid {
            grid: [[Cell::new(0, 0, 0).unwrap(); GRID_SIZE]; GRID_SIZE],
            solved_cell_ct: 0,
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

        if self.grid[x_u][y_u].defined {
            println!("Cell {},{} defined as {}", x, y, self.grid[x as usize][y as usize].value_as_int);
            return (possible, true)
        }

        for index in 0..GRID_SIZE {

            if self.get_col(x).contains(&possible[index]) {
                self.grid[x_u][y_u].set_impossible(possible[index]);
                possible[index] = 0;
                continue;
            }

            if self.get_row(y).contains(&possible[index]) {
                self.grid[x_u][y_u].set_impossible(possible[index]);
                possible[index] = 0;
                continue;
            }

            if self.get_box(Box::new(x,y)).contains(&possible[index]) {
                self.grid[x_u][y_u].set_impossible(possible[index]);
                possible[index] = 0;
                continue;
            }

        };

        (possible, self.check_for_guarantees(x, y))
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
    pub fn check_for_guarantees(&mut self, x: u32, y: u32) -> bool {
        let mut ct: u32= 0;
        let mut last_val = 0;

        let possible = self.get_status_as_list(x as usize, y as usize);
        for poss in possible {
            if poss.0 == ValueStatus::Possible {
                ct+= 1;
                last_val = poss.1;
            }
        }
        if ct == 1 {
            println!("Guaranteed value found at {}, {}: {}", x, y, last_val);
            self.set_value(x, y, last_val);
            return true;
        }

        return false
    }


    // Returns a list of pointers to the ValueStatus of each possible value in the given cell
    pub fn get_status_as_list(&self, x: usize, y: usize) -> [&(ValueStatus, u32); GRID_SIZE] {

        [
            &self.grid[x][y].one,
            &self.grid[x][y].two,
            &self.grid[x][y].three,
            &self.grid[x][y].four,
            &self.grid[x][y].five,
            &self.grid[x][y].six,
            &self.grid[x][y].seven,
            &self.grid[x][y].eight,
            &self.grid[x][y].nine,
        ]

    }

    // Try to solve the Sudoku
    pub fn analyse(&mut self) {

        'a: for x in 0..GRID_SIZE {
            'b: for y in 0..GRID_SIZE {
                if self.grid[x][y].defined {
                    println!("{:?} is already {}", self.grid[x][y].coordinate,self.grid[x][y].value_as_int);
                    continue;
                }
                let poss = self.get_possibles_by_cell(x as u32, y as u32);
                if poss.1 {
                    continue;
                }

                // Sets possible and impossibles
                for index in 0..GRID_SIZE {
                    if poss.0[index] == 0 {
                        self.grid[x][y].set_impossible(index as u32 + 1);
                    }
                }
            }
        }

        for number in 1..(GRID_SIZE_I+1) {
            if self.get_val_by_int(number) >= GRID_SIZE_I {
                continue;
            }
            for coord in 0..GRID_SIZE_I {
                self.get_possibles_by_neighbourhood(number, GridVector::Row, coord);
                self.get_possibles_by_neighbourhood(number, GridVector::Col, coord);
                self.get_possibles_by_neighbourhood(number, GridVector::Box, coord);
            }
        }

        for row in 0..self.grid.len() {
            for col in 0..self.grid[row].len() {
                self.check_for_guarantees(col as u32, row as u32);
            }
        }
    }

    pub fn inc_val_by_int<'a>(&mut self, value: u32) {
        match value {
            1 => self.definite_values.one += 1,
            2 => self.definite_values.two += 1,
            3 => self.definite_values.three += 1,
            4 => self.definite_values.four += 1,
            5 => self.definite_values.five += 1,
            6 => self.definite_values.six += 1,
            7 => self.definite_values.seven += 1,
            8 => self.definite_values.eight += 1,
            9 => self.definite_values.nine += 1,
            _ => {}
        }
    }

    pub fn get_val_by_int<'a>(&mut self, value: u32) -> u32 {
        match value {
            1 => self.definite_values.one,
            2 => self.definite_values.two,
            3 => self.definite_values.three,
            4 => self.definite_values.four,
            5 => self.definite_values.five,
            6 => self.definite_values.six,
            7 => self.definite_values.seven,
            8 => self.definite_values.eight,
            9 => self.definite_values.nine,
            _ => 0
        }
    }

    pub fn set_value(&mut self, x: u32, y: u32, value: u32) {
        println!("Setting cell {},{} as {}", x, y, value);
        self.grid[x as usize][y as usize] = Cell::new(value, x, y).unwrap();

        self.inc_val_by_int(value);
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
        println!("Col {} contains {:?}", x, numbers);
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
        println!("Row {} contains {:?}", y, numbers);
        numbers
    }

    // Returns all values found in the given box
    fn get_box(&self, g_box: Box) -> [u32; GRID_SIZE] {
        let left_col = match g_box.x {
            0..=2 => Ok((0)),
            3..=5 => Ok((3)),
            6..=8 => Ok((6)),
            _ => Err(("X Co-ordinate ({}) given out of bounds", g_box.x)),
        };

        let top_row = match g_box.y {
            0..=2 => Ok((0)),
            3..=5 => Ok((3)),
            6..=8 => Ok((6)),
            _ => Err(("Y Co-ordinate ({}) given out of bounds", g_box.y)),
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

        println!("{:?} contains {:?}", g_box, numbers);
        numbers
    }
}

impl Sudoku {
    pub fn new<'a>(name: u32) -> Self {
        Self {
            grid: Grid::new(),
            solved: false,
            name,
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

impl SudokusCollection {
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
        for mut sudoku in self.sudokus {
            if !sudoku.solved {
                println!("Analysing {}", sudoku.name);
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
            let val = line_as_string
                .get(pos..pos+1)
                .unwrap()
                .parse::<u32>()
                .unwrap();
            let x = inner_line as u32 - 1;
            let y = pos as u32;
            sudokus[sudoku_num].grid.grid[inner_line-1][pos] = Cell::new(val, x, y).unwrap();
        }
        sudokus[sudoku_num].name = (sudoku_num + 1) as u32;

        inner_line += 1;
    }

    SudokusCollection::new(sudokus)
}