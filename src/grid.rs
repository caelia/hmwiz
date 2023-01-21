#[derive(Debug)]
pub enum Orientation {
    RowMajor,
    ColMajor,
}

#[derive(Debug)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    orientation: Orientation,
    data: Vec<T>,
}

impl<T> Grid<T>
where
    T: Clone,
{
    pub fn new(rows: usize, cols: usize, orientation: Orientation, default: T) -> Self {
        let data = vec![default; rows * cols];
        Grid {
            rows,
            cols,
            orientation,
            data,
        }
    }

    fn get_index(&self, row: usize, col: usize) -> usize {
        match self.orientation {
            Orientation::RowMajor => row * self.cols + col,
            Orientation::ColMajor => col * self.rows + row,
        }
    }

    pub fn get(&self, row: usize, col: usize) -> &T {
        let idx = self.get_index(row, col);
        &self.data[idx]
    }

    pub fn set(&mut self, row: usize, col: usize, val: T) {
        let mut idx = self.get_index(row, col);
        self.data[idx] = val;
    }
}
