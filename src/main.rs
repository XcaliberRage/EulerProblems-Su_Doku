#![feature(label_break_value)]
#![feature(in_band_lifetimes)]

use crate::ValueStatus::{Impossible, Possible};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

// TODO Gotta investigate why number 7 is not completing, step by step!

// Solve 50 sudoku puzzles and sum the 3 digit answer in each top left
const SUDOKU_CT: usize = 50; // How many Sudokus you want to solve (out of the entire file given) inclusive, this is for testing really, normally you'd just want to do all of them
const SUDOKU_START: usize = 1; // Which of the given Sudokus you start with (1 is the first one)
const SUDOKU_END: usize = ((SUDOKU_START * 10) - 10) + (10 * SUDOKU_CT) - 1;
// There's almost no way changing these numbers won't really break the program
const GRID_SIZE: usize = 9;
const GRID_SIZE_I: u32 = GRID_SIZE as u32;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Sudoku {
    grid: SudokuGrid,
    solved: bool,
    name: u32,
    analysis_ct: u32,
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

impl PartialEq for ValueList {
    fn eq(&self, other: &Self) -> bool {
        self.one == other.one
            && self.two == other.two
            && self.three == other.three
            && self.four == other.four
            && self.five == other.five
            && self.six == other.six
            && self.seven == other.seven
            && self.eight == other.eight
            && self.nine == other.nine
    }
}

#[derive(Debug, Clone, Copy)]
struct SudokuGrid {
    grid: [[SudokuCell; GRID_SIZE]; GRID_SIZE],
    solved_cell_ct: u32,
    definite_values: ValueList,
    solved: bool,
}

impl PartialEq for SudokuGrid {
    fn eq(&self, other: &Self) -> bool {
        self.grid
            .iter()
            .zip(other.grid.iter())
            .all(|(a, b)| a.iter().zip(b.iter()).all(|(c, d)| c == d))
            && self.solved_cell_ct == other.solved_cell_ct
            && self.definite_values == other.definite_values
            && self.solved == other.solved
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum ValueStatus {
    Actual,
    Possible,
    Impossible,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl PartialEq for SudokuCell {
    fn eq(&self, other: &Self) -> bool {
        self.defined == other.defined
            && self.value_as_int == other.value_as_int
            && self.one.1 == other.one.1
            && self.two.1 == other.two.1
            && self.three.1 == other.three.1
            && self.four.1 == other.four.1
            && self.five.1 == other.five.1
            && self.six.1 == other.six.1
            && self.seven.1 == other.seven.1
            && self.eight.1 == other.eight.1
            && self.nine.1 == other.nine.1
            && self.coordinate == other.coordinate
    }
}

#[derive(Debug)]
struct CellRef {
    coordinate: Coordinate,
    value: u32,
}

struct SudokusCollection {
    sudokus: Vec<Sudoku>,
    solved: bool,
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
            coordinate: Coordinate { x, y },
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
                return cell;
            }
            1 => {
                cell.one.0 = ValueStatus::Actual;
                return cell;
            }
            2 => {
                cell.two.0 = ValueStatus::Actual;
                return cell;
            }
            3 => {
                cell.three.0 = ValueStatus::Actual;
                return cell;
            }
            4 => {
                cell.four.0 = ValueStatus::Actual;
                return cell;
            }
            5 => {
                cell.five.0 = ValueStatus::Actual;
                return cell;
            }
            6 => {
                cell.six.0 = ValueStatus::Actual;
                return cell;
            }
            7 => {
                cell.seven.0 = ValueStatus::Actual;
                return cell;
            }
            8 => {
                cell.eight.0 = ValueStatus::Actual;
                return cell;
            }
            9 => {
                cell.nine.0 = ValueStatus::Actual;
                return cell;
            }
            _ => panic!(),
        }
    }

    // Copy an array of value states into this cell
    pub fn new_with_states(x: u32, y: u32, states: Vec<(ValueStatus, u32)>) -> SudokuCell {
        let cell = SudokuCell {
            defined: false,
            value_as_int: 0,
            one: states[0],
            two: states[1],
            three: states[2],
            four: states[3],
            five: states[4],
            six: states[5],
            seven: states[6],
            eight: states[7],
            nine: states[8],
            coordinate: Coordinate { x, y },
        };

        cell
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

    pub fn set_impossible(&mut self, value: u32) -> (bool, u32) {
        if self.get_impossible().contains(&value) {
            return (false, 0);
        }

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

        //println!("        Set {} as impossible in {:?}: ValStats = {:?}", value, self.coordinate, self.get_val_stats_as_array());

        let poss = self.get_possible();
        //println!("          Poss = {:?}", poss);
        if poss.len() == 1 {
            //println!("    Only one possible value left! = {}", poss[0]);
            return (true, poss[0]);
        }

        if self.get_impossible().len() >= 9 {
            panic!(
                "All values in {:?} set to impossible! Was in processes of setting {}",
                self.coordinate, value
            );
        }

        (false, 0)
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

    // Returns a list of all possible ints for this cell
    fn get_possible(&self) -> Vec<u32> {
        let mut possibles = Vec::new();

        if self.one.0 == Possible {
            possibles.push(1);
        }
        if self.two.0 == Possible {
            possibles.push(2);
        }
        if self.three.0 == Possible {
            possibles.push(3);
        }
        if self.four.0 == Possible {
            possibles.push(4);
        }
        if self.five.0 == Possible {
            possibles.push(5);
        }
        if self.six.0 == Possible {
            possibles.push(6);
        }
        if self.seven.0 == Possible {
            possibles.push(7);
        }
        if self.eight.0 == Possible {
            possibles.push(8);
        }
        if self.nine.0 == Possible {
            possibles.push(9);
        }

        possibles
    }
}

impl Display for SudokuCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "\
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
            self.defined,
            self.value_as_int,
            self.coordinate,
            self.one,
            self.two,
            self.three,
            self.four,
            self.five,
            self.six,
            self.seven,
            self.eight,
            self.nine
        )
    }
}

#[derive(Debug, Clone)]
struct SudokuBox {
    x: u32,
    x_lim: u32,
    y: u32,
    y_lim: u32,
    content: Vec<Vec<SudokuCell>>,
}

impl SudokuBox {
    pub fn new(x: u32, y: u32, grid: SudokuGrid) -> SudokuBox {
        let mut g_box = Vec::new();

        for i_y in 0..3 {
            let mut g_row = Vec::new();
            for i_x in 0..3 {
                g_row.push(grid.grid[y as usize + i_y][i_x + x as usize]);
            }
            g_box.push(g_row);
        }

        SudokuBox {
            x,
            x_lim: x + 2,
            y,
            y_lim: y + 2,
            content: g_box,
        }
    }

    pub fn x_finder(value: u32) -> u32 {
        match value {
            0 | 3 | 6 => 0,
            1 | 4 | 7 => 3,
            2 | 5 | 8 => 6,
            _ => 0,
        }
    }

    pub fn coord_translator(value: u32) -> u32 {
        return match value {
            0..=2 => 0,
            3..=5 => 3,
            6..=8 => 6,
            _ => {
                panic!("Coordinate out of bounds")
            }
        };
    }

    pub fn y_finder(value: u32) -> u32 {
        match value {
            0..=2 => 0,
            3..=5 => 3,
            6..=8 => 6,
            _ => 0,
        }
    }

    pub fn get_vals(&self) -> Vec<u32> {
        let mut values = Vec::new();

        for y in &self.content {
            for x in y {
                values.push(x.value_as_int);
            }
        }

        values
    }

    // Return true if the given cell is within the bounds of this box
    pub fn in_box(&self, cell: SudokuCell) -> bool {
        if cell.coordinate.x < self.x {
            return false;
        }
        if cell.coordinate.x > self.x_lim {
            return false;
        }
        if cell.coordinate.y < self.y {
            return false;
        }
        if cell.coordinate.y > self.y_lim {
            return false;
        }
        true
    }

    pub fn as_vec(&self) -> Vec<SudokuCell> {
        let mut cells = Vec::new();

        for row in &self.content {
            for cell in row {
                if cell.defined {
                    cells.push(SudokuCell::new(
                        cell.value_as_int,
                        cell.coordinate.x,
                        cell.coordinate.y,
                    ));
                } else {
                    cells.push(SudokuCell::new_with_states(
                        cell.coordinate.x,
                        cell.coordinate.y,
                        cell.get_val_stats_as_array(),
                    ));
                }
            }
        }

        cells
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
                nine: 0,
            },
            solved: false,
        }
    }

    /*pub fn print(&self) {
        for row in self.grid {
            println!("{:?}", row.iter().map(|c| c.value_as_int).collect::<Vec<_>>())
        }
    }*/

    pub fn get_def_val_ct(&self) -> [u32; GRID_SIZE] {
        let mut def_vals = [0; GRID_SIZE];

        def_vals[0] = self.definite_values.one;
        def_vals[1] = self.definite_values.two;
        def_vals[2] = self.definite_values.three;
        def_vals[3] = self.definite_values.four;
        def_vals[4] = self.definite_values.five;
        def_vals[5] = self.definite_values.six;
        def_vals[6] = self.definite_values.seven;
        def_vals[7] = self.definite_values.eight;
        def_vals[8] = self.definite_values.nine;

        def_vals
    }

    // Looks at the given cell (by coordinate) and returns a tuple containing the list of possible values and an indicator
    // The indicator is true if only one value is possible
    // This list of possible values can be used to infer what
    pub fn get_possibles_by_cell(&mut self, x: u32, y: u32) -> (Vec<u32>, bool) {
        let mut possible = Vec::new();
        let y_u = y as usize;
        let x_u = x as usize;

        if self.grid[y_u][x_u].defined {
            //println!("    Cell {:?} is defined as {}", self.grid[y_u][x_u].coordinate ,self.grid[y_u][x_u].value_as_int);
            possible.push(self.grid[y_u][y_u].value_as_int);
            return (possible, true);
        }

        let col = self
            .get_vals_in_col(x)
            .iter()
            .map(|&n| n.value_as_int)
            .collect::<Vec<u32>>();
        let row = self
            .get_vals_in_row(y)
            .into_iter()
            .map(|n| n.value_as_int)
            .collect::<Vec<u32>>();
        let bx = self
            .get_vals_in_box(SudokuBox::new(
                SudokuBox::coord_translator(x),
                SudokuBox::coord_translator(y),
                *self,
            ))
            .into_iter()
            .map(|n| n.value_as_int)
            .collect::<Vec<u32>>();

        // Check each neighbourhood for the presence of a value, if it's not there, add it.
        // If it is there and it was IN the array, pull it.
        possible = self.grid[y_u][x_u].get_possible();

        for index in 1..GRID_SIZE_I + 1 {
            if col.contains(&index) {
                if possible.contains(&index) {
                    possible.remove(possible.iter().position(|x| x == &index).unwrap());
                }
                continue;
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
        }

        if possible.len() == 1 {
            //println!("    One possibility found at {:?}: {:?}", self.grid[y_u][x_u].coordinate, possible);
            return (possible, true);
        }

        for n in 0..GRID_SIZE_I {
            let i = n + 1;

            if possible.contains(&i) {
                continue;
            }

            if self.grid[y_u][x_u].get_impossible().contains(&i) {
                continue;
            }

            let p = self.grid[y_u][x_u].set_impossible(i);
            if p.0 {
                self.set_value(x, y, p.1);
            }
        }

        //println!("    Multiple possibilities found at {:?}: {:?}", self.grid[y_u][x_u].coordinate, possible);
        (possible, false)
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
                    continue;
                }

                //println!("Possible sibling found at {:?}: {:?}", cell.coordinate, poss_vals);
                let cell_row = Vec::from(self.grid[cell.coordinate.y as usize].clone());

                // Check neighbourhoods for similar cells
                //println!("    Row: {:?}", cell_row.iter().clone().map(|c| c.value_as_int).collect::<Vec<u32>>());
                self.check_siblings(cell_row, cell, &poss_vals);

                let cell_col = self
                    .grid
                    .iter()
                    .clone()
                    .map(|c| c[cell.coordinate.x as usize])
                    .collect::<Vec<SudokuCell>>();
                //println!("    Col: {:?}", cell_col.iter().clone().map(|c| c.value_as_int).collect::<Vec<u32>>());
                self.check_siblings(cell_col, cell, &poss_vals);

                let c_box = SudokuBox::new(
                    SudokuBox::coord_translator(cell.coordinate.x),
                    SudokuBox::coord_translator(cell.coordinate.y),
                    *self,
                );
                let cell_box = c_box.clone().as_vec();
                //println!("    Box: {:?}", cell_box.iter().clone().map(|c| c.value_as_int).collect::<Vec<u32>>());
                self.check_siblings(cell_box, cell, &poss_vals);
            }
        }
    }

    // Check the possibles cells in the given neighbourhood and if any pairs or triples are found
    // drop those possible values from the rest of the neighbourhood
    pub fn check_siblings(
        &mut self,
        nbhd: Vec<SudokuCell>,
        cell: SudokuCell,
        poss_vals: &Vec<(ValueStatus, u32)>,
    ) {
        // The neighbourhood trimmed down to only those cells that aren't defined
        let mut nbhd_poss = self.return_poss(nbhd.clone());
        // Number of possible values in this siblingdom (i.e. 2 or 3)
        let poss_len = poss_vals.len();

        // Drop the cell we're looking at from this neighbourhood
        'nbd: for index in 0..nbhd_poss.len() {
            if equal_coords(cell.coordinate, nbhd_poss[index].coordinate) {
                nbhd_poss.remove(index);
                break 'nbd;
            }
        }
        //println!("        Possible cells to check: {:?}", nbhd_poss);

        // Drop any cell that has something other than 2 or 3 possibles
        // (we might have some with 1 possible since we last checked for guarantees)
        let mut index = 0;
        while index < nbhd_poss.len() {
            let this_poss = nbhd_poss[index].get_possible();
            let length = this_poss.len();
            //println!("         Cell {:?} possibles ({}): {:?}", nbhd_poss[index].coordinate, length, this_poss);

            if length != poss_len {
                //println!("        Cell  has a different number of possibles, removing.");
                nbhd_poss.remove(index);
                continue;
            }
            index += 1;
        }
        //println!("        Looking for possible siblings: {:?}", nbhd_poss.iter().map(|c| c.coordinate).collect::<Vec<Coordinate>>());

        // Count matches between poss_vals and each [nbhd]_poss
        // If == we either have a pair or need to find a third cell

        let mut sibling_cells = Vec::new();
        let mut looking_for_third = false;

        'find_siblings: for nbhd_check in nbhd_poss.clone() {
            // Grab all possible values as a vector
            let t = nbhd_check
                .get_val_stats_as_array()
                .iter()
                .clone()
                .filter_map(|c| if c.0 == Possible { Some(c) } else { None })
                .map(|a| *a)
                .collect::<Vec<(ValueStatus, u32)>>();

            // Compare the two cells to see if they are siblings
            // (they both have got to have exactly 2 or 3 possibles and be identical)
            let matching = poss_vals
                .iter()
                .clone()
                .zip(t.iter().clone())
                .filter(|(&a, &b)| a.1 == b.1)
                .count();

            if !(matching == poss_len && matching == t.len()) {
                continue;
            }

            if !looking_for_third {
                sibling_cells.push(cell);
            }

            sibling_cells.push(nbhd_check.clone());

            // Check if it's a pair we found
            if poss_len == 2 {
                break 'find_siblings;
            }

            // Otherwise if we found our third
            if looking_for_third {
                break 'find_siblings;
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

        //println!("        All siblings found for {:?}: {:?}", poss_vals ,sibling_cells.iter().clone().map(|c| c.coordinate).collect::<Vec<Coordinate>>());

        let mut nbhd_new = nbhd.clone();

        // Lets drop the other siblings from nbhd_poss
        for cell in sibling_cells.clone() {
            let mut index = 0 as usize;
            let mut nbhd_len = nbhd_new.len();
            while index < nbhd_len {
                if equal_coords(cell.coordinate, nbhd_new[index].coordinate) {
                    nbhd_new.remove(index);
                    nbhd_len = nbhd_new.len();
                    continue;
                }
                index += 1;
            }
        }

        /*println!("        Cells {:?} will have the sibling set {:?} set to impossible",
        nbhd_new.iter().clone()
            .map(|c| c.coordinate).collect::<Vec<Coordinate>>(),
        poss_vals);*/

        // If we did, we want to set the possible values of these siblings in each OTHER cell in the neighbourhood to impossible
        'set_nbhd: for col_cell in nbhd.clone() {
            let y = col_cell.coordinate.y as usize;
            let x = col_cell.coordinate.x as usize;

            for sibling in sibling_cells.clone() {
                // Skip if it's one of the siblings
                if equal_coords(sibling.coordinate, col_cell.coordinate) || self.grid[y][x].defined
                {
                    //println!("    Skip {:?} ", col_cell.coordinate);
                    continue 'set_nbhd;
                }
            }

            for val in poss_vals.iter() {
                // println!("    Setting {} impossible in {},{}", val.1, x, y);

                let p = self.grid[y][x].set_impossible(val.1);

                if p.0 {
                    self.set_value(
                        self.grid[y][x].coordinate.x,
                        self.grid[y][x].coordinate.y,
                        p.1,
                    );
                }
            }
        }
    }

    // In a given neighbourhood, if a set of possible values can only go into a number of cells equal to the number of possibles
    // then no other values can go in these cells
    pub fn find_exclusives(&mut self, nbhd: Vec<SudokuCell>) {
        // Take each number and note each position in the array that that number _can_ go
        let mut all_possible = Vec::new();
        for cell in nbhd.iter() {
            let poss = cell.get_possible();
            if poss.len() == 0 {
                continue;
            }
            //println!("      - Poss in cell {:?}: {:?}", cell.coordinate, poss);
            for value in poss.iter() {
                if all_possible.contains(value) {
                    continue;
                }

                all_possible.push(*value);
            }
        }

        all_possible.sort();
        //println!("      {:?}", all_possible);

        let mut all_poss_array = HashMap::new();

        for value in all_possible.iter() {
            let mut poss_coords = Vec::new();
            for cell in nbhd.iter() {
                if cell.get_possible().contains(value) {
                    poss_coords.push(cell.coordinate);
                }
            }
            if poss_coords.len() >= 2 {
                all_poss_array.insert(*value, poss_coords);
            }
        }

        // Now we look to see if any values are limited to only the same cells as some other values
        // If so, the number of values that share these cell limits must be the same as the number of cells they are limited to
        // i.e. 2 values need to be limited to 2 possible cells in the neighbourhood
        //println!("    Look for matches:");
        let mut excludes = look_for_matches(all_poss_array.clone(), Vec::new());

        // Eliminate any sets that do not match their number of values to the number of possible cells
        let mut excludes_len = excludes.len();
        let mut i: usize = 0;
        while i < excludes_len {
            if all_poss_array[&excludes[i][0]].len() != excludes[i].len() {
                //println!("    Set {:?} fits in {} cells; removing.", excludes[i], all_poss_array[&excludes[i][0]].len());
                excludes.remove(i);
                excludes_len = excludes.len();
                continue;
            }
            i += 1;
        }

        /* if excludes.len() > 0 {
            println!("    Match groups found:");
            for exclude in excludes.clone() {
                println!("      {:?}: {:?}", exclude, all_poss_array[&exclude[0]]);
            }
        }*/

        let nbhd_poss = self.return_poss(nbhd.clone());
        for set in excludes.clone() {
            // Look in each cell, if it could contain all of the values in this set, kill any values that are not in this set
            for cell in nbhd_poss.iter() {
                let cell_set = cell.get_possible();
                if set.iter().all(|e| cell_set.contains(e)) {
                    for value in cell_set.clone() {
                        if set.contains(&value) {
                            continue;
                        }

                        let p = self.grid[cell.coordinate.y as usize][cell.coordinate.x as usize]
                            .set_impossible(value);
                        if p.0 {
                            self.set_value(cell.coordinate.x, cell.coordinate.y, p.1);
                        }
                    }
                }
            }
        }

        // Return a set of vectors,
        // each one is a group of values that are limited to a number of cells that is not greater than the number of values in the group
        // This function is recursive
        fn look_for_matches(
            all_poss_array: HashMap<u32, Vec<Coordinate>>,
            mut matches: Vec<Vec<u32>>,
        ) -> Vec<Vec<u32>> {
            //If we're looking at the last value in the list, just return cause we're done
            // This is the escape
            if all_poss_array.len() <= 1 {
                //println!("      Escape;");
                return matches;
            }

            // Take one of the values from the Hash
            let mut new_hash = all_poss_array.clone();
            let k_1 = *all_poss_array.keys().take(1).last().unwrap();

            new_hash.remove(&k_1);
            //println!("      Looking at {}: {:?}", k_1, all_poss_array[&k_1]);

            // I'll shove it in matches, if it's invalid later I can remove it
            matches.push(vec![k_1]);

            let mut matching = false;
            let index = matches.iter().rposition(|c| c.contains(&k_1)).unwrap();
            for (k_2, v_2) in new_hash.clone() {
                // Gets us the index we care about

                // Now we compare coordinates
                if v_2 == all_poss_array[&k_1] {
                    matching = true;
                    // If you find a match, make sure you didn't already put this second value in here, if not, put it in
                    if !matches[index].contains(&k_2) {
                        matches[index].push(k_2);
                    }

                    // If the number of values exceeds the number of cells that these values can go in, just forget it and move on
                    if matches[index].len() > all_poss_array[&k_1].len() {
                        matches.remove(index);
                        //println!("      Too big!");
                        break;
                    }
                    //println!(  "Match found: {}: {:?}", k_2, v_2);
                }
            }

            if !matching {
                //println!("    Matching = {}, Pulling k_1: {}", matching, k_1);
                // If we never found a match, then obviously this number doesn't have one so we take it back out
                matches.remove(
                    matches
                        .iter()
                        .rposition(|e| e == matches.iter().last().unwrap())
                        .unwrap(),
                );
            }

            //println!("        Passing {:?} WITH {:?}", new_hash, matches);
            look_for_matches(new_hash, matches)
        }
    }

    // Returns a vector of each cell from the given neighbourhood that is not defined
    pub fn return_poss(&self, nbhd: Vec<SudokuCell>) -> Vec<SudokuCell> {
        let mut vec = Vec::new();

        for cell in &nbhd {
            if !cell.defined {
                vec.push(SudokuCell::new_with_states(
                    cell.coordinate.x,
                    cell.coordinate.y,
                    cell.get_val_stats_as_array(),
                ));
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

        return false;
    }

    // Try to solve the Sudoku
    pub fn analyse(&mut self) {
        //let mut pass_ct: u32 = 0;
        loop {
            // This loop gets broken manually generally after returning void
            //pass_ct += 1;
            let old = self.clone();
            // First execute simple determinism, check what could possibly go in a give cell and if it's one number, assign
            if self.check_each() {
                //println!("    Took {} passes.", pass_ct);
                return;
            }

            // Next are neighbourhood checks against a specific value, assigning any guaranteed values
            for number in 1..(GRID_SIZE_I + 1) {
                //println!("Checking neighbourhoods for value {}:", number);
                if self.get_val_by_int(number) == GRID_SIZE_I {
                    //println!("{} occurences of {} found, no need to check!", self.get_val_by_int(number), number);
                    continue;
                }

                self.solve_for_number(number);
            }

            // Use box placements to eliminate possible placements in cols and rows
            self.box_elimination();
            // Do similar but for rows and cols, eliminating box possibles
            self.line_elimination();

            // Now we look at twins and triplets
            // Only bother if the "easy" methods didn't work this time
            if *self == old {
                //println!("Matching siblings");
                self.match_possibles();
            }

            if *self == old {
                self.x_wing();
            }

            // Now we look to see if a pair can only go in two cells in a neighbourhood
            // if so, eliminate the other possibilities from that cell
            //println!("----- Find Exclusives:");
            for i in 0..GRID_SIZE {
                let n_box = SudokuBox::new(
                    SudokuBox::x_finder(i as u32),
                    SudokuBox::y_finder(i as u32),
                    *self,
                )
                .as_vec();
                //println!("    Box {}: {:?}", i ,n_box.iter().clone().map(|c| c.coordinate).collect::<Vec<_>>());
                self.find_exclusives(n_box);
                self.find_exclusives(Vec::from(self.grid[i]));
                self.find_exclusives(self.grid.iter().clone().map(|&c| c[i]).collect::<Vec<_>>());
            }

            if self.check_each() {
                //println!("    Took {} passes.", pass_ct);
                return;
            }

            /*if *self == old {
                println!("    Changes made = {:?}", *self != old);
                changes_made = false;
                println!("    Took {} passes.", pass_ct);
            }*/
        }
    }

    // Look for guarantees, if you find any keep looking until you find none OR you just solved the whole thing
    pub fn check_each(&mut self) -> bool {
        //println!("Checking each cell for guaranteed values until none are found or the sudoku is completed");
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

        //println!("Check ended: Solved = {}, No guarantees found = {}", self.solved, no_guarantees_found);
        all_solved
    }

    pub fn inc_val_by_int<'a>(&mut self, value: u32) {
        //println!(" Added a new definite to grid: {}", value);

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

        //println!(" Value ct for {} is now {}", value ,self.get_def_val_ct()[(value - 1) as usize])
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
            _ => 0,
        }
    }

    // Sets a a cell in the grid using the given value and coordinates
    // Often, this results in chains of additional sets that can be checked for
    // To ensure no problematic race casing, if there are additional sets, resolve them first-in-first out
    pub fn set_value(&mut self, mut x: u32, mut y: u32, mut value: u32) {
        let mut sets = vec![CellRef {
            coordinate: Coordinate { x, y },
            value,
        }];
        let mut set_len = sets.len();
        let mut ct = 0;

        while set_len > 0 {
            if self.grid[sets[0].coordinate.y as usize][sets[0].coordinate.x as usize].defined {
                sets.remove(0);
                set_len = sets.len();
                continue;
            }

            //println!(" Sets left: {:?}", sets);
            x = sets[0].coordinate.x;
            y = sets[0].coordinate.y;
            value = sets[0].value;
            sets.remove(0);

            //println!("Setting cell {},{} as {}", x, y, value);
            self.grid[y as usize][x as usize] = SudokuCell::new(value, x, y);
            /*println!("New cell status:");
            println!("{}", self.grid[y as usize][x as usize]);*/

            // This is temporary value that doesn't actually get changed, it copies the values in the real grid, so it must be called each time alas
            // This is because memory hates me, or I hate me, I can't remember
            let bx = SudokuBox::new(
                SudokuBox::coord_translator(x),
                SudokuBox::coord_translator(y),
                *self,
            )
            .as_vec();

            // Check neighbourhoods for guaranteed values
            //println!(" Eliminating {} from rest of box, col and row:", value);
            for i in bx.clone() {
                if i.defined {
                    continue;
                }

                let p = self.grid[i.coordinate.y as usize][i.coordinate.x as usize]
                    .set_impossible(value);

                if p.0 {
                    sets.push(CellRef {
                        coordinate: i.coordinate,
                        value: p.1,
                    });
                    //println!("    Pushing new set: {:?} = {}", i, p.1);
                }
            }

            // TODO this is stupid and should not be three variant repetitions on doing the same exact thing, I need to learn macro rules!
            let bx_empty = bx
                .iter()
                .clone()
                .filter(|&&c| c.defined == false)
                .collect::<Vec<_>>();
            if bx_empty.len() == 1 {
                let last_cell = bx_empty.last().unwrap();
                let mut last_val = 0;

                let present_vals = self.collect_values(bx.clone());
                for i in 1..(GRID_SIZE_I + 1) {
                    if !present_vals.contains(&i) {
                        last_val = i;
                        break;
                    }
                }
                //println!(" One space left in box, must be {}.", last_val);

                sets.push(CellRef {
                    coordinate: last_cell.coordinate,
                    value: last_val,
                });
                //println!("    Pushing new set: {:?} = {}", last_cell.coordinate, last_val );
            }

            // Look in each cell in the row
            for row in self.grid[y as usize] {
                let row_vals = self.collect_values(self.get_vals_in_row(y));
                if row.defined || row_vals.contains(&row.value_as_int) {
                    continue;
                }

                let p = self.grid[row.coordinate.y as usize][row.coordinate.x as usize]
                    .set_impossible(value);

                if p.0 {
                    sets.push(CellRef {
                        coordinate: row.coordinate,
                        value: p.1,
                    });
                    //println!("    Pushing new set: {:?} = {}", row, p.1);
                }
            }

            let rw_empty = self.grid[y as usize]
                .iter()
                .clone()
                .filter(|&&c| c.defined == false)
                .collect::<Vec<_>>();
            if rw_empty.len() == 1 {
                //println!("        {:?}", rw_empty);
                let last_cell = *rw_empty[0];
                let mut last_val = 0;

                let present_vals = self.collect_values(self.get_vals_in_row(y));
                for i in 1..(GRID_SIZE_I + 1) {
                    if !present_vals.contains(&i) {
                        last_val = i;
                        break;
                    }
                }
                //println!(" One space left in row {}, must be {}.", y,last_val);

                sets.push(CellRef {
                    coordinate: last_cell.coordinate,
                    value: last_val,
                });
                //println!("    Pushing new set: {:?} = {}", last_cell.coordinate, last_val );
            }

            for col in self.grid {
                let col_vals = self.collect_values(self.get_vals_in_col(x));
                if col[x as usize].defined || col_vals.contains(&col[x as usize].value_as_int) {
                    continue;
                }

                let p = self.grid[col[x as usize].coordinate.y as usize]
                    [col[x as usize].coordinate.x as usize]
                    .set_impossible(value);

                if p.0 {
                    sets.push(CellRef {
                        coordinate: col[x as usize].coordinate,
                        value: p.1,
                    });
                    //println!("    Pushing new set: {:?} = {}", col[x as usize], p.1);
                }
            }

            let mut cl_empty = Vec::new();
            for cl in &self.grid {
                if cl[x as usize].defined {
                    continue;
                }

                cl_empty.push(cl[x as usize].clone())
            }
            if cl_empty.len() == 1 {
                let last_cell = cl_empty[0];
                let mut last_val = 0;

                let present_vals = self.collect_values(self.get_vals_in_col(x));
                for i in 1..(GRID_SIZE_I + 1) {
                    if !present_vals.contains(&i) {
                        last_val = i;
                        break;
                    }
                }

                //println!(" One space left in col, must be {}.", last_val);

                sets.push(CellRef {
                    coordinate: last_cell.coordinate,
                    value: last_val,
                });
                //println!("    Pushing new set: {:?} = {}", last_cell.coordinate, last_cell.value_as_int );
            }

            self.inc_val_by_int(value);
            set_len = sets.len();

            ct += 1;
            if ct >= 100 {
                panic!("Anti-infinity protector!");
            }
        }

        //println!("No new sets found.");
    }

    // For a given uncompleted number
    // Identify if it has at least two rows that it can only appear in two cells
    // If between two of those rows (and no more) those cells share a column
    // The number must appear somewhere in those 4 cells and therefore not in any other cell of matching column but different row
    // Row and Column may be swapped
    pub fn x_wing(&mut self) {
        //println!("XXXXXXXXXX Trying an X-Wing pass");

        let mut changed = false;

        'number_ct: for number in 1..(GRID_SIZE_I + 1) {
            if changed {
                break;
            }

            if self.get_def_val_ct()[(number - 1) as usize] == GRID_SIZE_I {
                continue 'number_ct;
            }

            // row_coords tracks for each row that _could_ take this number, which columns in that row the number fits
            let mut row_coords = HashMap::new();

            // Do rows first
            for row in 0..self.grid.len() {
                'cell_check: for col in 0..self.grid[row].len() {
                    if self.grid[row][col].defined {
                        continue 'cell_check;
                    }

                    if self.grid[row][col].get_possible().contains(&number) {
                        if !row_coords.contains_key(&row) {
                            row_coords.insert(row, vec![col]);
                        } else {
                            let mut vec = row_coords[&row].clone();
                            row_coords.remove(&row);
                            vec.push(col);
                            row_coords.insert(row, vec);
                        }
                    }
                }
            }

            /*if row_coords.len() > 0 {
                println!("    Check rows for number {}", number);
            }
            for (a,b) in row_coords.clone() {

                println!("      Row {} -> Columns {:?}", a, b);

            }*/

            // Look for a situation where exactly two rows have the same columns
            let mut matches = self.get_matched_lines(row_coords.clone());
            matches = self.trim_matches(matches);
            //println!("        Trimmed matches: {:?}", matches);

            for rows in matches.clone() {
                for row in 0..GRID_SIZE {
                    if rows.contains(&row) {
                        continue;
                    }

                    if !row_coords.contains_key(&row) {
                        continue;
                    }

                    //println!("       row_coords: {:?}", row_coords);
                    let target = row_coords.get(&row).unwrap().clone();
                    let target_columns = row_coords.get(&rows[0]).unwrap().clone();
                    //println!("        Any instances of {} in columns {:?} outside of rows {:?} are impossible", number, target_columns , rows);
                    //println!("        Check row {}, got columns {:?}", row, target);
                    for column in target.into_iter() {
                        if !target_columns.contains(&column) {
                            continue;
                        }

                        let p = self.grid[row][column].set_impossible(number);
                        if p.0 {
                            self.set_value(column as u32, row as u32, p.1);
                        }
                        changed = true;
                    }
                }
            }
        }
    }

    // Trim any rows that don't fit what we're looking for
    pub fn trim_matches(&self, mut matches: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
        let mut match_len = matches.len();
        let mut i: usize = 0;
        while i < match_len {
            if matches[i].len() == 2 {
                i += 1;
                continue;
            }

            matches.remove(i);
            match_len = matches.len();
        }

        matches
    }

    // Iterate over the group and return each group that have matches
    pub fn get_matched_lines(&self, lines: HashMap<usize, Vec<usize>>) -> Vec<Vec<usize>> {
        let mut matches: Vec<Vec<usize>> = Vec::new();
        let mut trim = lines.clone();

        'line_check: for (line, coords) in lines {
            let mut add_line = false;
            let mut coords_vec = Vec::new();
            trim.remove(&line);

            if coords.len() != 2 {
                continue;
            }

            for i in matches.clone() {
                if i.contains(&line) {
                    continue 'line_check;
                }
            }

            for (trim_line, trim_coords) in trim.clone() {
                if trim_coords == coords {
                    coords_vec.push(trim_line);
                    add_line = true;
                }
            }

            if add_line {
                coords_vec.push(line);
                matches.push(coords_vec);
            }
        }

        matches
    }

    // Returns all values found in the given column
    fn get_vals_in_col(&self, x: u32) -> Vec<SudokuCell> {
        let x_size = x as usize;
        let mut numbers = Vec::new();
        for row in 0..GRID_SIZE {
            let cell = self.grid[row][x_size];
            if cell.value_as_int != 0 {
                numbers.push(cell);
                continue;
            }
        }
        //println!("Col {} contains {:?}", x, numbers.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
        numbers
    }

    // Returns all values found in the given row
    fn get_vals_in_row(&self, y: u32) -> Vec<SudokuCell> {
        let y_size = y as usize;
        let mut numbers = Vec::new();
        for col in 0..GRID_SIZE {
            let cell = self.grid[y_size][col];
            if cell.value_as_int != 0 {
                numbers.push(cell);
                continue;
            }
        }
        //println!("Row {} contains {:?}", y, numbers.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
        numbers
    }

    // Returns all values found in the given box
    fn get_vals_in_box(&self, g_box: SudokuBox) -> Vec<SudokuCell> {
        let mut numbers = Vec::new();

        for row in 0..3 {
            for col in 0..3 {
                let cell = self.grid[row + (g_box.y as usize)][col + (g_box.x as usize)];
                if cell.value_as_int != 0 {
                    numbers.push(cell);
                    continue;
                }
            }
        }

        //println!("{:?} contains {:?}", g_box, numbers.iter().map(|c| c.value_as_int).collect::<Vec<u32>>());
        numbers
    }

    // Step through the grid trying to find placements for a specific number
    // Specifically looking at if the cell, row, column relationship forces a placement of that number
    // If a change is found, return true
    pub fn solve_for_number(&mut self, number: u32) {
        //println!(" - - - - - Solving for {}:", number);
        //println!("            Current ct in grid = {}", self.get_def_val_ct()[(number - 1) as usize]);
        //println!("Current number ct = {}", self.get_def_val_ct()[(number - 1) as usize]);

        if self.get_def_val_ct()[(number - 1) as usize] == GRID_SIZE_I {
            //println!("Number {} has completed all placements ({})", number, self.get_def_val_ct()[(number -1) as usize]);
            return;
        }

        for row in self.grid {
            for cell in row {
                if cell.defined {
                    continue;
                }

                // Skip if we know this number isn't possible already
                if !self
                    .get_possibles_by_cell(cell.coordinate.x, cell.coordinate.y)
                    .0
                    .contains(&number)
                {
                    continue;
                }

                let nbhd_col = self
                    .grid
                    .iter()
                    .map(|c| c[cell.coordinate.x as usize])
                    .collect::<Vec<_>>();
                let nbhd_col_vals = self.collect_values(nbhd_col.clone());
                let nbhd_row = self.grid[cell.coordinate.y as usize]
                    .iter()
                    .map(|&c| c)
                    .collect::<Vec<_>>();
                let nbhd_row_vals = self.collect_values(nbhd_row.clone());
                let nbhd_box = SudokuBox::new(
                    SudokuBox::coord_translator(cell.coordinate.x),
                    SudokuBox::coord_translator(cell.coordinate.y),
                    *self,
                );
                let nbhd_box_vals = nbhd_box.get_vals();

                // Skip if this number can't be here
                if nbhd_col_vals.contains(&number) {
                    continue;
                }

                if nbhd_row_vals.contains(&number) {
                    continue;
                }

                if nbhd_box_vals.contains(&number) {
                    continue;
                }

                /*println!("    Neighbourhoods for {:?}", cell.coordinate);
                println!("      Row: {:?}", nbhd_row_vals);
                println!("      Col: {:?}", nbhd_col_vals);
                println!("      Box: {:?}", nbhd_box_vals);*/

                let mut possible_in_other_box = false;
                let mut possible_in_other_row = false;
                let mut possible_in_other_col = false;

                // Look at each other cell in this box, if you find a possibility for this number anywhere else in the box, skip it
                //println!("    Checking box:");
                'row_iterator: for box_row in &nbhd_box.content {
                    'col_iterator: for box_cell in box_row {
                        if equal_coords(box_cell.coordinate, cell.coordinate) {
                            continue 'col_iterator;
                        }

                        if box_cell.defined {
                            //println!("        {:?} is defined, skip.", box_cell.coordinate);
                            continue 'col_iterator;
                        }

                        let possibles = box_cell.get_possible();
                        //println!("        Possibles in cell {:?}: {:?}", box_cell.coordinate, possibles);
                        if !possibles.contains(&number) {
                            //println!("        Can't take {}, skip.", number);
                            continue 'col_iterator;
                        }

                        possible_in_other_box = true;
                    }
                    if possible_in_other_box {
                        break 'row_iterator;
                    }
                }

                // Now look at the row and column silly!
                // (if you need to)
                if possible_in_other_box {
                    /*println!("    Box does not eliminate possibility");
                    println!("    Checking row:");*/
                    'cell_iterator: for row_cell in &nbhd_row {
                        if equal_coords(row_cell.coordinate, cell.coordinate) {
                            continue 'cell_iterator;
                        }

                        if row_cell.defined {
                            //println!("        {:?} is defined, skip.", row_cell.coordinate);
                            continue 'cell_iterator;
                        }

                        let possibles = row_cell.get_possible();
                        //println!("        Possibles in cell {:?}: {:?}", row_cell.coordinate, possibles);
                        if !possibles.contains(&number) {
                            //println!("          Can't take {}, skip.", number);
                            continue 'cell_iterator;
                        }

                        possible_in_other_row = true;
                        break 'cell_iterator;
                    }
                    if possible_in_other_row {
                        /*println!("   Row does not eliminate possibility");
                        println!("    Checking col:");*/
                        'cell_iterator_2: for col_cell in &nbhd_col {
                            if equal_coords(col_cell.coordinate, cell.coordinate) {
                                continue 'cell_iterator_2;
                            }

                            if col_cell.defined {
                                //println!("        {:?} is defined, skip.", col_cell.coordinate);
                                continue 'cell_iterator_2;
                            }

                            let possibles = col_cell.get_possible();
                            //println!("        Possibles in cell {:?}: {:?}", col_cell.coordinate, possibles);
                            if !possibles.contains(&number) {
                                //println!("          Can't take {}, skip.", number);
                                continue 'cell_iterator_2;
                            }

                            possible_in_other_col = true;
                            break 'cell_iterator_2;
                        }
                    }
                }

                if possible_in_other_box && possible_in_other_col && possible_in_other_row {
                    continue;
                }

                /*println!("Can go elsewhere in box: {}", possible_in_other_box);
                println!("Can go elsewhere in row: {}", possible_in_other_row);
                println!("Can go elsewhere in col: {}", possible_in_other_col);*/
                self.set_value(cell.coordinate.x, cell.coordinate.y, number);
            }
        }

        //println!("- - - Finished solving for {}", number);
    }

    pub fn box_elimination(&mut self) {
        //println!("--- Box elimination:");
        for boxdex in 0..GRID_SIZE_I {
            let new_box = SudokuBox::new(
                SudokuBox::x_finder(boxdex),
                SudokuBox::y_finder(boxdex),
                *self,
            );
            /*println!("  Possibles for box {},{} to {},{}", new_box.x, new_box.y, new_box.x_lim, new_box.y_lim);
            for box_row in new_box.content.clone().into_iter() {
                for cell in box_row {
                    print!("{:?}: ", cell.coordinate);
                    if cell.defined {
                        println!("defined");
                    } else {
                        println!("{:?}", cell.get_possible());
                    }
                }
            }*/

            // Identify if the possibles of a number are limited to one column or row
            for number in 1..GRID_SIZE_I + 1 {
                if new_box.get_vals().contains(&number) {
                    continue;
                }
                //println!("        Try number {}:", number);

                // An array of cells that can take this number
                let poss_vec = new_box
                    .as_vec()
                    .into_iter()
                    .filter(|c| c.get_possible().contains(&number))
                    .collect::<Vec<_>>();

                // If all the cells either match in x or y coord, we have an elimination.
                let poss_vec_ys = poss_vec
                    .iter()
                    .clone()
                    .map(|c| c.coordinate.y)
                    .collect::<Vec<_>>();
                let poss_vec_xs = poss_vec
                    .iter()
                    .clone()
                    .map(|c| c.coordinate.x)
                    .collect::<Vec<_>>();
                /*println!("          X coords: {:?}", poss_vec_xs);
                println!("          Y coords: {:?}", poss_vec_ys);*/

                let mut same_row_ct = 0;
                for cell in &poss_vec_ys {
                    if *cell == poss_vec_ys[0] {
                        same_row_ct += 1;
                    }
                }
                //println!("           {:?} == {:?}", same_row_ct, poss_vec.len());
                // For every other cell in the same row (but not this box)
                if same_row_ct == poss_vec.len() {
                    let row = poss_vec_ys[0] as usize;
                    for cell in self.grid[row] {
                        if new_box.in_box(cell) {
                            //println!("            {:?} is in box, skip",cell.coordinate);
                            continue;
                        }
                        let p = self.grid[cell.coordinate.y as usize][cell.coordinate.x as usize]
                            .set_impossible(number);
                        if p.0 {
                            self.set_value(cell.coordinate.x, cell.coordinate.y, p.1);
                        }
                    }
                    continue;
                }

                // Check for column if it's not the row
                let mut same_col_ct = 0;
                for cell in &poss_vec_xs {
                    if *cell == poss_vec_xs[0] {
                        same_col_ct += 1;
                    }
                }
                //println!("           {:?} == {:?}", same_col_ct, poss_vec.len());
                // For every other cell in the same row (but not this box)
                if same_col_ct == poss_vec.len() {
                    let col = poss_vec_xs[0] as usize;
                    for row in self.grid {
                        if new_box.in_box(row[col]) {
                            //println!("            {:?} is in box, skip", row[col].coordinate);
                            continue;
                        }
                        let p = self.grid[row[col].coordinate.y as usize]
                            [row[col].coordinate.x as usize]
                            .set_impossible(number);
                        if p.0 {
                            self.set_value(row[col].coordinate.x, row[col].coordinate.y, p.1);
                        }
                    }
                }
            }
        }
    }

    // Look at each column and row, if a value is limited to a specific box in that line, any other possible position in that box is eliminated for that value
    pub fn line_elimination(&mut self) {
        //println!("--- Line Elimination:");

        // by row
        for row in self.grid.clone() {
            /*print!("  Check row:");
            println!(" {}", row[0].coordinate.y);*/

            let mut empty_row = Vec::new();
            let mut defined_nums = Vec::new();

            for cell in row {
                if !cell.defined {
                    empty_row.push(cell.clone());
                    continue;
                }

                defined_nums.push(cell.value_as_int);
            }

            //println!("    {:?}", empty_row.iter().clone().map(|c| c.get_possible()).collect::<Vec<_>>());

            for num in 1..GRID_SIZE_I + 1 {
                if defined_nums.contains(&num) {
                    continue;
                }

                let cells = empty_row
                    .iter()
                    .clone()
                    .filter(|c| c.get_possible().contains(&num))
                    .collect::<Vec<_>>();
                let mut all_in_one_box = true;
                let check_box = SudokuBox::new(
                    SudokuBox::coord_translator(cells[0].coordinate.x),
                    SudokuBox::coord_translator(cells[0].coordinate.y),
                    *self,
                );
                'box_check: for check_i in 1..cells.len() {
                    if !check_box.in_box(*cells[check_i]) {
                        all_in_one_box = false;
                        break 'box_check;
                    }
                }

                if !all_in_one_box {
                    continue;
                }

                //println!("    Possible value {} is found only in box {},{} to {},{}", num ,check_box.x, check_box.y, check_box.x_lim, check_box.y_lim);

                for box_cell in check_box.as_vec() {
                    // row[0] could be _any_ cell in the row as I just want the row value here
                    if box_cell.coordinate.y == row[0].coordinate.y || row[0].defined {
                        continue;
                    }

                    let p = self.grid[box_cell.coordinate.y as usize]
                        [box_cell.coordinate.x as usize]
                        .set_impossible(num);
                    if p.0 {
                        self.set_value(box_cell.coordinate.x, box_cell.coordinate.y, p.1);
                    }
                }
            }
        }

        // by Col

        for col in 0..GRID_SIZE {
            /*print!("  Check column:");
            println!(" {}", col);*/

            let mut empty_col = Vec::new();
            let mut defined_nums = Vec::new();

            for cell in self.grid.clone() {
                if !cell[col].defined {
                    empty_col.push(cell[col].clone());
                    continue;
                }

                defined_nums.push(cell[col].value_as_int);
            }

            //println!("    {:?}", empty_col.iter().clone().map(|c| c.get_possible()).collect::<Vec<_>>());

            for num in 1..GRID_SIZE_I + 1 {
                if defined_nums.contains(&num) {
                    continue;
                }

                let cells = empty_col
                    .iter()
                    .clone()
                    .filter(|&&c| c.get_possible().contains(&num))
                    .collect::<Vec<_>>();
                let mut all_in_one_box = true;
                let check_box = SudokuBox::new(
                    SudokuBox::coord_translator(cells[0].coordinate.x),
                    SudokuBox::coord_translator(cells[0].coordinate.y),
                    *self,
                );
                'box_check_2: for check_i in 1..cells.len() {
                    if !check_box.in_box(*cells[check_i]) {
                        all_in_one_box = false;
                        break 'box_check_2;
                    }
                }

                if !all_in_one_box {
                    continue;
                }
                //println!("    Possible value {} is found only in box {},{} to {},{}", num ,check_box.x, check_box.y, check_box.x_lim, check_box.y_lim);

                for box_cell in check_box.as_vec() {
                    // self.grid[0][col] could be _any_ cell in the column as I just want the column value here
                    if box_cell.coordinate.x == self.grid[0][col].coordinate.x {
                        continue;
                    }

                    let p = self.grid[box_cell.coordinate.y as usize]
                        [box_cell.coordinate.x as usize]
                        .set_impossible(num);
                    if p.0 {
                        self.set_value(box_cell.coordinate.x, box_cell.coordinate.y, p.1);
                    }
                }
            }
        }
    }

    pub fn collect_values(&self, vector: Vec<SudokuCell>) -> Vec<u32> {
        vector.iter().map(|x| x.value_as_int).collect()
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
            name: name + SUDOKU_START as u32,
            analysis_ct: 0,
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
    pub fn new(collection: Vec<Sudoku>) -> SudokusCollection {
        SudokusCollection {
            sudokus: collection,
            solved: false,
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

            /*println!("\nSudoku {}: [Solved = {}]", sudoku.name, sudoku.solved);
            for line in sudoku.grid.grid.iter() {
                let vals = line.iter().map(|c| c.get_val()).collect::<Vec<u32>>();
                println!("{:?}", vals);
            }*/
            /*if !sudoku.solved {
                println!("Placements made:");
                let defs = sudoku.grid.get_def_val_ct();
                for i in 0..GRID_SIZE{
                    println!("    {} = {}/9", i+1, defs[i]);
                }
            }*/
        }
    }

    pub fn analyse(&mut self) {
        let mut solved_all = true;
        for mut sudoku in self.sudokus.iter_mut() {
            if !sudoku.solved {
                //println!(":::::::::::::::::::::Analysing {}", sudoku.name);
                sudoku.analysis_ct += 1;
                sudoku.grid.analyse();
                let this_solved = sudoku.check_solved();
                if solved_all {
                    solved_all = this_solved;
                }
            }
        }
        self.solved = solved_all;
        /*self.print("unsolved");
        println!("Sudokus solved: {}", self.solved);*/
    }
}

fn main() -> std::io::Result<()> {
    let path = Path::new("sudoku.txt");
    let file = File::open(&path)?;
    let mut sudokus = get_sudokus(&file);

    let mut buffer = String::new();
    let stdin = io::stdin();

    sudokus.print("all");
    println!("Ready to solve?");
    stdin.read_line(&mut buffer).expect("Error reading");
    let mut ct: u32 = 0;

    let now = Instant::now();
    'main: while !sudokus.solved {
        ct += 1;
        //sudokus.print("unsolved");
        sudokus.analyse();
        if sudokus.solved {
            break 'main;
        }

        let cont = "y"; // Change this value to ask() if you want to stop after each analysis, or "y" if you want it to go until it's done
        if cont == "n" {
            break 'main;
        }
        let ct_max = 100; // After this many loops the program assumes it will never complete the sudokus and panic
        if ct > ct_max {
            panic!(
                "Anti-infinity protection protocols activated! [currently set at {} passes]",
                ct_max
            )
        }
    }

    /*println!("Sudokus solved = {}", sudokus.solved);
    sudokus.print("all");*/
    println!("All sudokus solved in {} analyses", ct);
    println!("Solve took {} ms", now.elapsed().as_millis());

    Ok(())
}

/*fn ask() -> String {
    let stdin = io::stdin();
    let mut cont = String::new();
    println!("Keep going? [Y/N]");
    stdin.read_line(&mut cont).expect("Failed to understand input");
    cont = cont.to_lowercase().trim().parse().unwrap();
    println!("{} received... {}", cont, cont == "n");
    cont
}*/

// Parses a text file into an array of sudokus to solve
fn get_sudokus(file: &File) -> SudokusCollection {
    let mut sudokus = Vec::new();

    let reader = BufReader::new(file);
    let mut true_line = 0; // Tracks the actual line of the file
    let mut inner_line: usize = 0; // Tracks the line for the individual Sudoku read
    let mut sudoku_num: usize = 0; // Tracks the current sudoku

    for line in reader.lines() {
        // For skipping earlier sudokus
        if true_line < (10 * SUDOKU_START) - 10 {
            //println!("Skip line {}: {:?}", true_line ,line);
            true_line += 1;
            continue;
        }

        if true_line > SUDOKU_END {
            //println!("All done!");
            return SudokusCollection::new(sudokus);
        }

        let line_as_string = line.unwrap();

        if line_as_string.starts_with('G') && true_line >= SUDOKU_START * 10 {
            //println!("Reset line ct");
            inner_line = 0;
            sudoku_num += 1;
        };

        //println!("Line {}({}) is {}",inner_line ,true_line, line_as_string);
        true_line += 1;

        if inner_line == 0 {
            //println!("New Sudoku!");
            sudokus.push(Sudoku::new((sudoku_num) as u32));
            inner_line += 1;
            continue;
        }

        for pos in 0..GRID_SIZE {
            let val = line_as_string
                .get(pos..pos + 1)
                .unwrap()
                .parse::<u32>()
                .unwrap();
            let y = inner_line as u32 - 1;
            let x = pos as u32;
            sudokus[sudoku_num].grid.grid[inner_line - 1][pos] = SudokuCell::new(val, x, y);
            if val > 0 {
                sudokus[sudoku_num].grid.inc_val_by_int(val);
            }
        }

        inner_line += 1;
    }

    SudokusCollection::new(sudokus)
}
