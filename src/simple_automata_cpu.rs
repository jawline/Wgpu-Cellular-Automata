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

struct DoubleBuffer {
    buffers: [Vec<Cell>; 2],
    i: usize,
}

impl DoubleBuffer {
    fn new(cells: Vec<Cell>) -> Self {
        let second = (0..cells.len()).map(|_| Cell::Empty).collect();
        Self {
            buffers: [cells, second],
            i: 0,
        }
    }

    fn read(&self) -> &Vec<Cell> {
        &self.buffers[self.i]
    }

    fn write(&mut self) -> &mut Vec<Cell> {
        &mut self.buffers[(self.i + 1) % 2]
    }

    fn next(&mut self) {
        self.i = (self.i + 1) % 2;
    }
}

pub struct Automata {
    pub dim: Vec3,
    buffer: DoubleBuffer,
}

impl Automata {
    // TODO: Consider using two buffers to store the cells
    pub fn new(dim: &Vec3) -> Self {
        Self {
            dim: *dim,
            buffer: DoubleBuffer::new(
                (0..(dim.x * dim.y * dim.z))
                    .map(|_| {
                        if rand::random::<f32>() <= 0.001 {
                            Cell::Alive
                        } else {
                            Cell::Empty
                        }
                    })
                    .collect(),
            ),
        }
    }

    pub fn offset(&self, pos: &Vec3) -> usize {
        let size_of_layer = self.dim.x * self.dim.y;
        let offset_in_layer_to_row = self.dim.x * pos.y;
        (size_of_layer * pos.z) + offset_in_layer_to_row + pos.x
    }

    pub fn get(&self, pos: &Vec3) -> Cell {
        let offset = self.offset(pos);
        self.buffer.read()[offset]
    }

    pub fn set(&mut self, pos: &Vec3, cell: Cell) {
        let offset = self.offset(pos);
        self.buffer.write()[offset] = cell;
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
                        let cell =
                            self.get(&Vec3::new(pos.x - 1 + x, pos.y - 1 + y, pos.z - 1 + z));
                        sum += cell.count();
                    }
                }
            }
            sum - self.get(pos).count()
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
                        2 | 3 | 4 => {
                            f(&pos);
                            Cell::Alive
                        }
                        _ => Cell::Empty,
                    };

                    self.set(&pos, cell);
                    let offset = self.offset(&pos);
                }
            }
        }
        self.buffer.next();
    }
}
