use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::io::prelude::*;
use std::fmt::{Display, Formatter, Result, Debug};
use crate::ValueStatus::{Possible, Actual, Impossible};

// Solve 50 sudoku puzzles and sum the 3 digit answer in each top left
const SUDOKU_CT: usize = 1;
const GRID_SIZE: usize = 9;
const GRID_SIZE_I: u32 = GRID_SIZE as u32;

#[derive(Debug, Clone)]
struct Sudoku {
    grid: Grid,
    solved: bool,
}

#[derive(Debug, Clone)]
struct Grid {
    grid: [[Cell; GRID_SIZE]; GRID_SIZE]
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum ValueStatus {
    Actual,
    Possible,
    Impossible,
}

#[derive(Debug, Clone, Copy)]
struct Cell {
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

impl Cell {
    pub fn new(val: u32) -> std::result::Result<Cell, &'static str> {
        match val {
            0 => Ok((Cell {
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

        if self.one == Actual {
            return Ok((1))
        } else if self.one == Impossible { impossibles += 1; };
        if self.two == Actual {
            return Ok((2))
        } else if self.one == Impossible { impossibles += 1; };
        if self.three == Actual {
            return Ok((3))
        } else if self.one == Impossible { impossibles += 1; };
        if self.four == Actual {
            return Ok((4))
        } else if self.one == Impossible { impossibles += 1; };
        if self.five == Actual {
            return Ok((5))
        } else if self.one == Impossible { impossibles += 1; };
        if self.six == Actual {
            return Ok((6))
        } else if self.one == Impossible { impossibles += 1; };
        if self.seven == Actual {
            return Ok((7))
        } else if self.one == Impossible { impossibles += 1; };
        if self.eight == Actual {
            return Ok((8))
        } else if self.one == Impossible { impossibles += 1; };
        if self.nine == Actual {
            return Ok((9))
        } else if self.one == Impossible { impossibles += 1; };
        
        if impossibles >= GRID_SIZE_I {
            return Err(("All values marked as impossible!"))
        };
        Ok((0))
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
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            grid: [[Cell::new(0).unwrap(); GRID_SIZE]; GRID_SIZE]
        }
    }

    // Returns a pointer to the specified location
    pub fn point_val(&self, x: u32, y: u32) -> &Cell {
        &self.grid[y as usize][x as usize]
    }

    // Returns a list of all possible values for the give coordinate
    pub fn get_possibles(&self, x: u32, y: u32) -> [u32; GRID_SIZE] {
        let mut possible: [u32; GRID_SIZE] = [1,2,3,4,5,6,7,8,9];

        for index in 0..GRID_SIZE {

            if self.get_col(x).contains(&possible[index]) {
                possible[index] = 0;
                continue;
            }

            if self.get_row(y).contains(&possible[index]) {
                possible[index] = 0;
                continue;
            }

            if self.get_box(Box::new(x,y)).contains(&possible[index]) {
                possible[index] = 0;
                continue;
            }

        };

        print!("Possible values for cell {},{}: ", x, y);
        let mut ct: u32 = 0;
        for poss in possible.iter() {
            if *poss != 0 {
                print!("{} ", *poss);
                ct += 1;
            }
        }
        if ct == 1 {print!(" <- ONE VALUE POSSIBLE");}
        println!();
        possible
    }

    pub fn analyse(&self) {
        for x in 0..GRID_SIZE_I {
            for y in 0..GRID_SIZE_I {
                if self.grid[x as usize][y as usize].get_val().unwrap() > 0 {
                    println!("Cell {},{} is {}", x, y, self.grid[x as usize][y as usize].get_val().unwrap());
                    continue;
                }
                self.get_possibles(x,y);
            }
            println!();
        }
    }



    // Returns all values found in the given column
    fn get_col(&self, x: u32) -> [u32; GRID_SIZE] {
        let x_size = x as usize;
        let mut numbers: [u32; GRID_SIZE] = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            let value = &self.grid[x_size][row].get_val().unwrap();
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

    let mut file = File::open(&path)?;

    let mut sudokus = get_sudokus(&file);
    let mut ct = 0;

    for sudoku in sudokus.iter() {
        println!("Sudoku {}:", ct+1);
        ct += 1;
        for line in sudoku.grid.grid.iter() {
            let vals = line.iter().map(|c|c.get_val().unwrap()).collect::<Vec<u32>>();
            println!("{:?}", vals);
        }

        println!("\n Now solving:");

        sudoku.grid.analyse();
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
            sudokus[sudoku_num].grid.grid[inner_line-1][pos] = Cell::new(line_as_string
                .get(pos..pos+1)
                .unwrap()
                .parse::<u32>()
                .unwrap())
                .unwrap();
        }

        inner_line += 1;
    }

    sudokus
}