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

    // I think slices should be a mutable iterator?
    pub fn get_slices(&self) -> Vec<&mut Vec<T>> {
        let (major, minor) = match self.orientation {
            Orientation::RowMajor => (self.rows, self.cols),
            Orientation::ColMajor => (self.cols, self.rows),
        };
        let result = Vec::new();
        for i in 0..major {
            let start = i * minor;    
            let end = start + minor;
            result.push(&self.data[start..end]);
        }
        result
    }
    /*
    pub fn get_seq(&self, idx: usize) -> Vec<T> {
        match self.orientation {
            Orientation::RowMajor {
                
            },
            Orientation::ColMajor {
                
            }
        }    
    }
    */
}
