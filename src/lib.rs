mod utils;

use fixedbitset::FixedBitSet;
use std::fmt;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    //! Returns a pseudorandom number between 0 and 1.
    #[wasm_bindgen(js_namespace = Math)]
    pub fn random() -> f64;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        log(&format!( $( $t )* ));
    };
}

trait SetIterBool {
    fn set_iter(&mut self, from: usize, source: &[bool]);
}

impl SetIterBool for FixedBitSet {
    fn set_iter(&mut self, from: usize, source: &[bool]) {
        let limit = self.len();
        let mut idx = from;
        for item in source.iter() {
            if idx >= limit { () }
            self.set(idx, *item);
            idx += 1;
        }
    }
}

/**
 * A universe representation in the game of life.
 */
#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.cells.len() {
            if i % (self.width as usize) == 0 {
                write!(f, "\n")?;
            }
            let symbol = if self.cells[i] { '◻' } else { '◼' };
            write!(f, "{}", symbol)?;
        }
        Ok(())
    }
}

impl Universe {
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter() {
            let idx = self.get_index(*row, *col);
            self.cells.set(idx, true);
        }
    }

    pub fn empty_cells(&mut self) {
        self.cells.set_range(.., false);
    }
}

#[wasm_bindgen]
impl Universe {
    /**
     * Create and initialize a new universe.
     */
    pub fn new(width: u32, height: u32) -> Universe {
        utils::set_panic_hook();
        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, random() < 0.3);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..self.cells.len() {
            self.cells.set(i, random() < 0.3);
        }
    }

    /**
     * Toggle specified cell value.
     */
    pub fn toggle_cell(&mut self, row: u32, col: u32) {
        log!("toggling cell on row {} col {}", row, col);
        let idx = self.get_index(row, col);
        let cell = self.cells[idx];
        self.cells.set(idx, !cell);
    }

    /**
     * Put a [Glider](https://en.wikipedia.org/wiki/Glider_(Conway%27s_Life)#Hacker_emblem) with center in row, col.
     */
    pub fn put_glider(&mut self, row: u32, col: u32) {
        log!("putting glidder on row {} col {}", row, col);
        let col_left = (col + self.width - 1) % self.width;

        let glider_top = [false, true, false];
        let row_top = (row + self.height - 1) % self.height;
        self.cells.set_iter(self.get_index(row_top, col_left), &glider_top);

        let glider_middle = [false, false, true];
        self.cells.set_iter(self.get_index(row, col_left), &glider_middle);

        let glider_bottom = [true, true, true];
        let row_bottom = (row + 1) % self.height;
        self.cells.set_iter(self.get_index(row_bottom, col_left), &glider_bottom);
    }

    pub fn put_pulsar(&mut self, row: u32, col: u32) {
        log!("putting pulsar on row {} col {}", row, col);
        let p1 = &[false, false, true, true, true, false, false, false, true, true, true, false, false];
        let p2 = &[false, false, false, false, false, false, false, false, false, false, false, false, false];
        let p3 = &[true, false, false, false, false, true, false, true, false, false, false, false, true];

        let pulsar = [p1, p2, p3, p3, p3, p1, p2, p1, p3, p3, p3, p2, p1];
        let left_col = (col + self.width - 6) % self.width;
        let mut curr_row = (row + self.height - 6) % self.height;
        for &line in pulsar.iter() {
            let idx = self.get_index(curr_row, left_col);
            self.cells.set_iter(idx, line);
            curr_row = (curr_row + 1) % self.height;
        }
    }

    /**
     * Returns the width of the universe.
     */
    pub fn width(&self) -> u32 {
        self.width
    }

    /**
     * Returns the height of the universe.
     */
    pub fn height(&self) -> u32 {
        self.height
    }

    /**
     * Returns a pointer to the universe's cells in wasm memory.
     */
    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    /**
     * Returns a string representation of the universe.
     */
    pub fn render(&self) -> String {
        self.to_string()
    }

    /**
     * Simulate a step in the universe.
     */
    pub fn tick(&mut self) {
        let mut next_cells = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let index = self.get_index(row, col);
                let cell = self.cells[index];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours dies.
                    (true, x) if x < 2 => false,
                    // Rule 2: Any live cell with two or three live neighbours lives
                    // onto the next generation.
                    (true, 2) | (true, 3) => true,
                    // Rule 3: Any live cell with more than three neighbours dies.
                    (true, x) if x > 3 => false,
                    // Rule 4: Any dead cell with exactly three neighbours becomes a live cell.
                    (false, 3) => true,
                    // All other cells remain in the same state.
                    (other, _) => other,
                };
                next_cells.set(index, next_cell);
            }
        }

        self.cells = next_cells;
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let index = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[index] as u8;
            }
        }
        count
    }
}
