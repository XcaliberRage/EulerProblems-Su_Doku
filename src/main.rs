#![feature(label_break_value)]
#![feature(in_band_lifetimes)]

use std::{env, io};
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::io::prelude::*;
use std::fmt::{Display, Formatter, Result, Debug, Error};
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
    grid: SudokuGrid,
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
struct SudokuGrid {
    grid: [[SudokuCell; GRID_SIZE]; GRID_SIZE],
    solved_cell_ct: u32,
    definite_values: ValueList,
    solved: bool,
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
struct SudokuCell {
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

impl SudokuCell {
    pub fn new(val: u32, x: u32, y: u32) -> SudokuCell {

        let mut cell = SudokuCell {
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
                return cell
            },
            1 => {
                cell.one.0 = ValueStatus::Actual;
                return cell
            },
            2 => {
                cell.two.0 = ValueStatus::Actual;
                return cell
            },
            3 => {
                cell.three.0 = ValueStatus::Actual;
                return cell
            },
            4 => {
                cell.four.0 = ValueStatus::Actual;
                return cell
            },
            5 => {
                cell.five.0 = ValueStatus::Actual;
                return cell
            },
            6 => {
                cell.six.0 = ValueStatus::Actual;
                return cell
            },
            7 => {
                cell.seven.0 = ValueStatus::Actual;
                return cell
            },
            8 => {
                cell.eight.0 = ValueStatus::Actual;
                return cell
            },
            9 => {
                cell.nine.0 = ValueStatus::Actual;
                return cell
            },
            _ => panic!()
        }
    }


    pub fn get_val(&self) -> u32 {

        let mut impossibles:u32 = 0;

        if self.defined {
            return self.value_as_int
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
            return panic!("All values marked as impossible! {:?}", self.coordinate)
        };
        0
    }

    // Returns an array of the value statuses
    pub fn get_val_stats_as_array(&self) -> Vec<(ValueStatus, u32)> {

        let mut arr = Vec::new();

        arr.push(self.one);
        arr.push(self.two);
        arr.push(self.three);
        arr.push(self.four);
        arr.push(self.five);
        arr.push(self.six);
        arr.push(self.seven);
        arr.push(self.eight);
        arr.push(self.nine);

        arr
            }

    pub fn set_impossible(&mut self, value: u32) {

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

        if self.get_impossible().len() >= 9 {
            panic!("All values in {:?} set to impossible! Was in processes of setting {}", self.coordinate, value);
        }
    }

    pub fn get_impossible(&self) -> Vec<u32> {

        let mut impossible = Vec::new();

        if self.one.0 == ValueStatus::Impossible {
            impossible.push(1 as u32);
        }
        if self.two.0 == ValueStatus::Impossible {
            impossible.push(2 as u32);
        }
        if self.three.0 == ValueStatus::Impossible {
            impossible.push(3 as u32);
        }
        if self.four.0 == ValueStatus::Impossible {
            impossible.push(4 as u32);
        }
        if self.five.0 == ValueStatus::Impossible {
            impossible.push(5 as u32);
        }
        if self.six.0 == ValueStatus::Impossible {
            impossible.push(6 as u32);
        }
        if self.seven.0 == ValueStatus::Impossible {
            impossible.push(7 as u32);
        }
        if self.eight.0 == ValueStatus::Impossible {
            impossible.push(8 as u32);
        }
        if self.nine.0 == ValueStatus::Impossible {
            impossible.push(9 as u32);
        }

        impossible
    }

    pub fn check_guarantee(&mut self) {

        if self.defined {
            return;
        }

        let mut possible_ct: u32 = 0;
        let mut last = &mut (ValueStatus::Actual, 0);

        if self.one.0 == ValueStatus::Possible {
            last = &mut self.one;
            possible_ct += 1;
        }
        if self.two.0 == ValueStatus::Possible {
            last = &mut self.two;
            possible_ct += 1;
        }
        if self.three.0 == ValueStatus::Possible {
            last = &mut self.three;
            possible_ct += 1;
        }
        if self.four.0 == ValueStatus::Possible {
            last = &mut self.four;
            possible_ct += 1;
        }
        if self.five.0 == ValueStatus::Possible {
            last = &mut self.five;
            possible_ct += 1;
        }
        if self.six.0 == ValueStatus::Possible {
            last = &mut self.six;
            possible_ct += 1;
        }
        if self.seven.0 == ValueStatus::Possible {
            last = &mut self.seven;
            possible_ct += 1;
        }
        if self.eight.0 == ValueStatus::Possible {
            last = &mut self.eight;
            possible_ct += 1;
        }
        if self.nine.0 == ValueStatus::Possible {
            last = &mut self.nine;
            possible_ct += 1;
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

impl Display for SudokuCell {
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
struct SudokuBox {
    x: u32,
    x_lim: u32,
    y: u32,
    y_lim: u32
}

impl SudokuBox {
    pub fn new(x: u32, y: u32) -> SudokuBox {
        SudokuBox {
            x,
            x_lim: x+2,
            y,
            y_lim: y+2,
        }
    }

    pub fn y_finder(value: u32) -> u32 {
        match value {
            0 | 3 | 6 => 0,
            1 | 4 | 7 => 3,
            2 | 5 | 8 => 6,
            _ => 0
        }
    }

    pub fn coord_translator(value: u32) -> u32 {

        return match value {
            0..=2 => 0,
            3..=5 => 3,
            6..=8 => 6,
            _ => {panic!("Coordinate out of bounds")}
        }

    }

    pub fn x_finder(value: u32) -> u32 {
        match value {
            0..=2 => 0,
            3..=5 => 3,
            6..=8 => 6,
            _ => 0
        }
    }

    // Return true if the given cell is within the bounds of this box
    pub fn in_box(&self, cell: SudokuCell) -> bool {
        if cell.coordinate.x < self.x {return false;}
        if cell.coordinate.x > self.x_lim {return false;}
        if cell.coordinate.y < self.y {return false;}
        if cell.coordinate.y > self.y_lim {return false;}
        true
    }
}

impl SudokuGrid {
    pub fn new() -> SudokuGrid {
        SudokuGrid {
            grid: [[SudokuCell::new(0, 0, 0); GRID_SIZE]; GRID_SIZE],
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
            },
            solved: false,
        }
    }

    // Returns a pointer to the specified location
    pub fn point_val(&mut self, x: u32, y: u32) -> &SudokuCell {
        &self.grid[y as usize][x as usize]
    }

    pub fn get_def_val_ct(&self) -> [u32; GRID_SIZE] {

        let mut def_vals = [0;GRID_SIZE];

        def_vals[0] = self.definite_values.one;
        def_vals[1] = self.definite_values.two;
        def_vals[2] = self.definite_values.three;
        def_vals[3] = self.definite_values.four;
        def_vals[4] = self.definite_values.five;
        def_vals[5] = self.definite_values.six;
        def_vals[6] = self.definite_values.seven;
        def_vals[7] = self.definite_values.eight;
        def_vals[7] = self.definite_values.nine;

        def_vals

    }

    // Looks at the given cell (by coordinate) and returns a tuple containing the list of possible values and an indicator
    // The indicator is true if only one value is possible
    // This list of possible values can be used to infer what
    pub fn get_possibles_by_cell(&self, x: u32, y: u32) -> (Vec<u32>, bool) {
        let mut possible = Vec::new();
        let y_u = y as usize;
        let x_u = x as usize;

        if self.grid[y_u][x_u].defined {
            println!("    Cell {:?} is defined as {}", self.grid[y_u][x_u].coordinate ,self.grid[y_u][x_u].value_as_int);
            possible.push(self.grid[y_u][y_u].value_as_int);
            return (possible, true)
        }

        let col = self.get_col(x)
                .iter()
                .map(|&n| n.value_as_int)
                .collect::<Vec<u32>>();
        let row = self.get_row(y)
                .into_iter()
                .map(|n| n.value_as_int)
                .collect::<Vec<u32>>();
        let bx = self.get_box(SudokuBox::new(SudokuBox::coord_translator(x), SudokuBox::coord_translator(y)))
                .into_iter()
                .map(|n| n.value_as_int)
                .collect::<Vec<u32>>();

        println!("  Cell {:?}", self.grid[y_u][x_u].coordinate);
        println!("    Col: {:?}", col);
        println!("    Row: {:?}", row);
        println!("    Box: {:?}", bx);

        // Check each neighbourhood for the presence of a value, if it's not there, add it.
        // If it is there and it was IN the array, pull it.
        for index in 1..GRID_SIZE_I+1 {
            if col.contains(&index) {
                if possible.contains(&index) {
                    possible.remove(possible.iter().position(|x| x == &index).unwrap());
                }
                continue;
            } else {
                if !possible.contains(&index) {
                    possible.push(index)
                }
            }

            if row.contains(&index) {
                if possible.contains(&index) {
                    possible.remove(possible.iter().position(|x| x == &index).unwrap());
                }
                continue;
            } else {
                if !possible.contains(&index) {
                    possible.push(index)
                }
            }

            if bx.contains(&index) {
                if possible.contains(&index) {
                    possible.remove(possible.iter().position(|x| x == &index).unwrap());
                }
                continue;
            } else {
                if !possible.contains(&index) {
                    possible.push(index)
                }
            }

        };

        if possible.len() == 1 {
            println!("    One possibility found at {:?}: {:?}", self.grid[y_u][x_u].coordinate, possible);
            return (possible, true);
        }

        println!("    Multiple possibilities found at {:?}: {:?}", self.grid[y_u][x_u].coordinate, possible);
        (possible, false)
    }

    // Takes one value and determines possible placement within a specific neighbourhood (row, column or box)
    // It identifies if the value is already in the neighbourhood,
    // in which case it eliminates the possibility from all cells in that neighbourhood
    pub fn get_possibles_by_neighbourhood(&mut self, value: u32, vector: GridVector, coord: u32) {

        let neighbourhood = match vector {
            GridVector::Col => self.get_col(coord),
            GridVector::Row => self.get_row(coord),
            GridVector::Box => self.get_box(SudokuBox::new(SudokuBox::x_finder(coord), SudokuBox::y_finder(coord))),
            _ => panic!("Attempted to give non-vector where a vector was required!")
        };

        // We just need the list of values here
        let nbhd_vals = neighbourhood
            .into_iter()
            .map(|n| n.value_as_int)
            .collect::<Vec<u32>>();

        if !nbhd_vals.contains(&value) {
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
                let x_offset = SudokuBox::x_finder(coord) as usize;
                let y_offset = SudokuBox::y_finder(coord) as usize;

                for y in y_offset..(y_offset + BOX_SIZE) {
                    for x in x_offset..(x_offset + BOX_SIZE) {
                        if self.grid[y][x].defined {
                            continue;
                        }
                        self.grid[y][x].set_impossible(value);
                    }
                }
            },
            _ => {}
        }

    }

    // Take a given neighbourhood
    // For each empty space, identify the possible values
    // If there's only two or three in the given cell, identify if there are any other cells in the neighbourhood that have the same limits
    // For twins only two other cells needed
    // For triplets 3 cells needed
    // If so, eliminate these possible values from the rest of the neighbourhood
    pub fn match_possibles(&mut self) {

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let &mut cell = &mut self.grid[row][col];
                let vals = cell.get_val_stats_as_array();

                let mut poss_vals = Vec::new();

                for val in vals {
                    if val.0 == Possible {
                        poss_vals.push(val)
                    }
                }

                let poss_len = poss_vals.len();
                if poss_len != 2 && poss_len != 3 {
                    continue
                }
                
                println!("Possible sibling found at {:?}: {:?}", cell.coordinate, poss_vals);

                let mut cell_row = Vec::from(self.grid[cell.coordinate.y as usize]);
                let mut cell_col = self.grid.iter().map(|c| c[cell.coordinate.x as usize]).collect::<Vec<SudokuCell>>();
                let c_box = SudokuBox::new(SudokuBox::x_finder(cell.coordinate.x), SudokuBox::y_finder(cell.coordinate.y));
                let mut cell_box = Vec::new();
                'box_check: for row in self.grid {
                    for col in row {
                        if c_box.in_box(col) {
                            println!("    {:?} is in box {:?}", col.coordinate, c_box);
                            cell_box.push(col)
                        }
                        if cell_box.len() == 9 {
                            break 'box_check;
                        }
                    }
                }

                // Check neighbourhoods for similar cells
                println!("    Row: {:?}", cell_row.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
                self.check_siblings(cell_row, cell, &poss_vals);
                println!("    Col: {:?}", cell_col.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
                self.check_siblings(cell_col, cell, &poss_vals);
                println!("    Box: {:?}", cell_box.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
                self.check_siblings(cell_box, cell, &poss_vals);
            }
        }

    }
    
    // Check the possibles cells in the given neighbourhood and if any pairs or triples are found
    // drop those possible values from the rest of the neighbourhood
    pub fn check_siblings(&mut self, nbhd: Vec<SudokuCell>, cell: SudokuCell, poss_vals: &Vec<(ValueStatus, u32)>) {

        // The neighbourhood trimmed down to only those cells that aren't defined
        let mut nbhd_poss = self.return_poss(nbhd);
        // Number of possible values in this siblingdom (i.e. 2 or 3)
        let mut poss_len = poss_vals.len();
        
        // Drop the cell we're looking at from this neighbourhood
        'nbd: for index in 0..nbhd_poss.len() {
            if equal_coords(cell.coordinate, nbhd_poss[index].coordinate) {
                nbhd_poss.remove(index);
                break 'nbd;
            }
        }


        // Drop any cell that has something other than 2 or 3 possibles
        // (we might have some with 1 possible since we last checked for guarantees)
        let mut index = 0;
        while index < nbhd_poss.len() {
            let nbhd_poss_this = nbhd_poss[index].get_val_stats_as_array()
                .iter().filter_map(|&c| if c.0 == Possible {Some(c)} else {None}).collect::<Vec<(ValueStatus, u32)>>();
            let length = nbhd_poss_this.len();
            if length != 2 && length != 3 {
                nbhd_poss.remove(index);
                continue;
            }
            index += 1;
        }
        //println!("        Looking for possible siblings: {:?}", nbhd_poss.iter().map(|c| (c.coordinate, c.value_as_int)).collect::<Vec<(Coordinate, u32)>>());

        // Count matches between poss_vals and each [nbhd]_poss
        // If == we either have a pair or need to find a third cell

        let mut sibling_cells = Vec::new();
        let mut looking_for_third = false;

        'find_siblings: for nbhd_check in nbhd_poss.iter() {

            // Grab all possible values as a vector
            let mut t = nbhd_check
                .get_val_stats_as_array()
                .iter()
                .filter_map(|c| if c.0 == Possible {Some(c)} else {None})
                .map(|a| *a)
                .collect::<Vec<(ValueStatus, u32)>>();

            // Compare the two cells to see if they are siblings
            // (they both have got to have exactly 2 or 3 possibles and be identical)
            let matching = poss_vals.into_iter()
                .zip(t.clone())
                .filter(|(a,b)| a.1 == b.1)
                .count();

            if  !(matching == poss_len && matching == t.len()) {
                continue;
            }

            if !looking_for_third {
                sibling_cells.push(cell);
            }

            sibling_cells.push(*nbhd_check);

            // Check if it's a pair we found
            if poss_len == 2 {
                break 'find_siblings;
            }
            
            // Otherwise if we found our third
            if looking_for_third {
                break 'find_siblings
            }
            
            looking_for_third = true;

        }

        //println!("Siblings found: {:?}", sibling_cells);
        
        // If we didn't find any siblings then move on
        if sibling_cells.is_empty() {
            return;
        }

        // We also need to skip if we found a broken sibling (3 possibles but only 2 matching cells)
        if sibling_cells.len() != poss_len {
            return;
        }

        println!("        All siblings found for {:?}: {:?}", poss_vals ,sibling_cells.iter().map(|c| c.coordinate).collect::<Vec<Coordinate>>());

        // Lets drop the other siblings from nbhd_poss
        for cell in &sibling_cells {
            let mut index = 0 as usize;
            let mut nbhd_len = nbhd_poss.len();
            while index < nbhd_len {
                if cell.coordinate.y == nbhd_poss[index].coordinate.y && cell.coordinate.x == nbhd_poss[index].coordinate.x {
                    nbhd_poss.remove(index);
                    nbhd_len = nbhd_poss.len();
                    continue;
                }
                index += 1;
            }
        }

        println!("        Cells {:?} will have the sibling set {:?} set to impossible",
                 nbhd_poss.iter()
                     .map(|c| c.coordinate).collect::<Vec<Coordinate>>(),
                 poss_vals);

        // If we did, we want to set the possible values of these siblings in each OTHER cell in the neighbourhood to impossible
        'set_nbhd: for col_cell in &nbhd_poss {
            for sibling in &mut sibling_cells {
                // Skip if it's one of the siblings
                if equal_coords(sibling.coordinate, col_cell.coordinate) {
                    continue 'set_nbhd;
                }
            }
                
            for val in poss_vals.iter() {
                let y = col_cell.coordinate.y as usize;
                let x = col_cell.coordinate.x as usize;
                self.grid[y][x] = SudokuCell::new(col_cell.value_as_int, col_cell.coordinate.x, col_cell.coordinate.y);
                let mut cell = &mut self.grid[y][x];

                // This is a little awkward but due to lifetimes I essentially remake the cell
                let mut imposs = col_cell.get_impossible();
                imposs.push(val.1);

                for impossible in imposs {
                    cell.set_impossible(impossible);
                }
            }

        }

        //println!("Neighbourhood updated: {:?}", nbhd_poss);
    }

    // Returns a vector of each cell from the given neighbourhood that is not defined
    pub fn return_poss(&self, nbhd: Vec<SudokuCell>) -> Vec<SudokuCell> {

        let mut vec = Vec::new();

        for cell in nbhd {
            if !cell.defined {
                vec.push(cell);
            }
        }

        vec
    }

    // Checks to identify if there is only one possible value for the cell and sets it if so
    // Returns true if this is done
    pub fn check_for_guarantees(&mut self, x: u32, y: u32) -> bool {

        let possibles = self.get_possibles_by_cell(x, y);

        if possibles.1 {
            let val = possibles.0.iter().sum();
            if val > 0 {
                self.set_value(x, y, val);
                return true;
            }
            // Return false if all you found was a defined value
            return false;
        }

        return false
    }


    // Returns a list of pointers to the ValueStatus of each possible value in the given cell
    pub fn get_status_as_list(&self, x: usize, y: usize) -> [&(ValueStatus, u32); GRID_SIZE] {

        [
            &self.grid[y][x].one,
            &self.grid[y][x].two,
            &self.grid[y][x].three,
            &self.grid[y][x].four,
            &self.grid[y][x].five,
            &self.grid[y][x].six,
            &self.grid[y][x].seven,
            &self.grid[y][x].eight,
            &self.grid[y][x].nine,
        ]

    }

    // Try to solve the Sudoku
    pub fn analyse(&mut self) {

        // First execute simple determinism, check what could possibly go in a give cell and if it's one number, assign
        if self.check_each() {
            return
        }

        // Next are neighbourhood checks against a specific value, assigning any guaranteed values
        for number in 1..(GRID_SIZE_I+1) {
            //println!("Checking neighbourhoods for value {}:", number);
            if self.get_val_by_int(number) >= GRID_SIZE_I {
                println!("{} occurences of {} found, no need to check!", self.get_val_by_int(number), number);
                continue;
            }
            //println!("{} occurences found...", self.get_val_by_int(number));
            for coord in 0..GRID_SIZE_I {
                self.get_possibles_by_neighbourhood(number, GridVector::Row, coord);
                self.get_possibles_by_neighbourhood(number, GridVector::Col, coord);
                self.get_possibles_by_neighbourhood(number, GridVector::Box, coord);
            }
        }

        if self.check_each() {
            return
        }

        // Now we look at twins and triplets
        println!("Matching siblings");
        self.match_possibles();
        if self.check_each() {
            return
        }
    }

    // Look for guarantees, if yu find any keep looking until you find none OR you just solved the whole thing
    pub fn check_each(&mut self) -> bool {
        println!("Checking each cell for guaranteed values until none are found or the sudoku is completed");
        let mut no_guarantees_found = false;
        let mut all_solved = self.solved;

        while !no_guarantees_found && !all_solved && !self.solved {
            no_guarantees_found = true;
            for row in 0..self.grid.len() {
                for col in 0..self.grid[row].len() {
                    if !self.grid[row][col].defined {
                        if self.check_for_guarantees(col as u32, row as u32) {
                            no_guarantees_found = false;
                        }
                    }
                }
            }
            // If all values are placed (i.e. there's 9 of each value, they would sum to 9*9)
            all_solved = self.get_def_val_ct().iter().sum::<u32>() == GRID_SIZE_I.pow(2);
        }

        println!("Check ended: Solved = {}, No guarantees found = {}", self.solved, no_guarantees_found);
        all_solved
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
        self.grid[y as usize][x as usize] = SudokuCell::new(value, x, y);
        println!("New cell status:");
        println!("{}", self.grid[y as usize][x as usize]);

        for mut row in self.grid[y as usize] {
            if row.defined {
                continue
            }

            row.set_impossible(value);
        }
        for mut col in self.grid {
            if col[y as usize].defined {
                continue
            }

            col[y as usize].set_impossible((value));
        }

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
    fn get_col(&self, x: u32) -> Vec<SudokuCell> {
        let x_size = x as usize;
        let mut numbers= Vec::new();
        for row in 0..GRID_SIZE {
            let cell = self.grid[row][x_size];
            if cell.value_as_int != 0 {
                numbers.push(cell);
                continue;
            }
        };
        //println!("Col {} contains {:?}", x, numbers.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
        numbers
    }

    // Returns all values found in the given row
    fn get_row(&self, y: u32) -> Vec<SudokuCell> {
        let y_size = y as usize;
        let mut numbers = Vec::new();
        for col in 0..GRID_SIZE {
            let cell = self.grid[y_size][col];
            if cell.value_as_int != 0 {
                numbers.push(cell);
                continue;
            }
        };
        //println!("Row {} contains {:?}", y, numbers.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
        numbers
    }

    // Returns all values found in the given box
    fn get_box(&self, g_box: SudokuBox) -> Vec<SudokuCell> {
        let left_col = match g_box.x {
            0..=2 => 0,
            3..=5 => 3,
            6..=8 => 6,
            _ => panic!("X Co-ordinate ({}) given out of bounds", g_box.x),
        };

        let top_row = match g_box.y {
            0..=2 => 0,
            3..=5 => 3,
            6..=8 => 6,
            _ => panic!("Y Co-ordinate ({}) given out of bounds", g_box.y),
        };

        let mut numbers = Vec::new();

        for row in 0..3 {
            for col in 0..3 {
                let cell = self.grid[row+top_row][col+left_col];
                if cell.value_as_int != 0 {
                    numbers.push(cell);
                    continue;
                }
            }
        };

        //println!("{:?} contains {:?}", g_box, numbers.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
        numbers
    }
}

// Compares two cell coordinates and returns true if they match
fn equal_coords(a: Coordinate, b: Coordinate) -> bool {
    if a.y == b.y {
        return a.x == b.x;
    }
    false
}

impl Sudoku {
    pub fn new<'a>(name: u32) -> Self {
        Self {
            grid: SudokuGrid::new(),
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
        self.set_solved();
        true
    }

    pub fn set_solved(&mut self) {
        self.solved = true;
        self.grid.solved = true;
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

            println!("\nSudoku {}: [Solved = {}]", sudoku.name, sudoku.solved);
            for line in sudoku.grid.grid.iter() {
                let vals = line.iter().map(|c| c.get_val()).collect::<Vec<u32>>();
                println!("{:?}", vals);
            }
        }
    }

    pub fn analyse(&mut self) {
        let mut solved_all = true;
        for mut sudoku in self.sudokus.iter_mut() {
            if !sudoku.solved {
                println!("Analysing {}", sudoku.name);
                sudoku.grid.analyse();
                if solved_all {
                    solved_all = sudoku.check_solved();
                }
            }
        }
        self.solved = solved_all;
        self.print("all");
        println!("Sudokus solved: {}", self.solved);

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
    let mut ct: u32 = 0;

    'main: while !sudokus.solved {
        ct += 1;
        sudokus.print("unsolved");
        sudokus.analyse();
        if sudokus.solved {
            break 'main
        }

        let mut cont = String::new();
        println!("Keep going? [Y/N]");
        stdin.read_line(&mut cont).expect("Failed to understand input");
        cont = cont.to_lowercase().trim().parse().unwrap();
        println!("{} received... {}", cont, cont == "n");
        if cont == "n" {
            break 'main;
        }
        let ct_max = 100;
        if ct > ct_max {
            panic!("Anti-infinity protection protocols activated! [currently set at {} passes]", ct_max)
        }
    }

    println!("Sudokus solved = {}", sudokus.solved);
    sudokus.print("all");
    println!("All sudokus solved in {} analyses", ct);

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
            let y = inner_line as u32 - 1;
            let x = pos as u32;
            sudokus[sudoku_num].grid.grid[inner_line-1][pos] = SudokuCell::new(val, x, y);
            if val > 0 {
                sudokus[sudoku_num].grid.inc_val_by_int(val);
            }
        }
        sudokus[sudoku_num].name = (sudoku_num + 1) as u32;

        inner_line += 1;
    }

    SudokusCollection::new(sudokus)
}