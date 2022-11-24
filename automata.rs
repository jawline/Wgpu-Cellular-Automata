#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl Vec3 {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Cell {
    Alive,
    Empty,
}

impl Cell {
    fn count(&self) -> usize {
        match self {
            Cell::Alive => 1,
            Cell::Empty => 0,
        }
    }
}

pub struct Automata {
    pub dim: Vec3,
    pub cells: Vec<Cell>,
}

impl Automata {
    // TODO: Consider using two buffers to store the cells
    pub fn new(dim: &Vec3) -> Self {
        Self {
            dim: *dim,
            cells: vec![
                if rand::random() {
                    Cell::Alive
                } else {
                    Cell::Empty
                };
                dim.x * dim.y * dim.z
            ],
        }
    }

    fn offset(&self, pos: &Vec3) -> usize {
        let size_of_layer = self.dim.x * self.dim.y;
        let offset_in_layer_to_row = self.dim.x * pos.y;
        size_of_layer + offset_in_layer_to_row + pos.x
    }

    pub fn get(&self, pos: &Vec3) -> Cell {
        self.cells[self.offset(pos)]
    }

    pub fn set(&mut self, pos: &Vec3, cell: Cell) {
        let offset = self.offset(pos);
        self.cells[offset] = cell;
    }

    fn neighbors(&self, pos: &Vec3) -> usize {
        // TODO: Skip the illegal bounds sensibly
        if pos.x == 0
            || pos.x == self.dim.x - 1
            || pos.y == 0
            || pos.y == self.dim.y - 1
            || pos.z == 0
            || pos.z == self.dim.z - 1
        {
            0
        } else {
            let mut sum = 0;
            for x in 0..3 {
                for y in 0..3 {
                    for z in 0..3 {
                        sum += self
                            .get(&Vec3::new(pos.x - 1 + x, pos.y - 1 + y, pos.z - 1 + z))
                            .count();
                    }
                }
            }
            sum
        }
    }

    pub fn update<F>(&mut self, mut f: F)
    where
        F: FnMut(&Vec3),
    {
        for x in 0..self.dim.x {
            for y in 0..self.dim.y {
                for z in 0..self.dim.z {
                    let pos = Vec3::new(x, y, z);
                    let neighbors = self.neighbors(&pos);

                    let cell = match neighbors {
                        3 | 4 | 5 => {
                            f(&pos);
                            Cell::Alive
                        }
                        _ => Cell::Empty,
                    };

                    self.set(&pos, cell);
                }
            }
        }
    }
}
