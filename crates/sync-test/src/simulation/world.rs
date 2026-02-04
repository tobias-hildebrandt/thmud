#[derive(Debug, Clone, Copy)]
pub struct CellLocation {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct WorldData<Data> {
    pub(crate) world_size: usize,
    pub data: Box<[Box<[Data]>]>,
}

impl<Data: Default> WorldData<Data> {
    pub(crate) fn new(world_size: usize, generate: impl Fn(usize, usize) -> Data) -> Self {
        Self {
            world_size,
            data: (0..world_size)
                .map(|row| {
                    Vec::from_iter((0..world_size).map(|column| generate(row, column)))
                        .into_boxed_slice()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub(crate) fn new_default(world_size: usize) -> Self {
        Self::new(world_size, |_, _| Default::default())
    }
}

pub(crate) type WorldCells = WorldData<WorldCell>;

impl WorldCells {
    pub(crate) fn new_random(world_size: usize) -> Self {
        Self::new(world_size, |_, _| WorldCell {
            state: rand::random(),
        })
    }
    pub(crate) fn random_mutation(&mut self) {
        let row = rand::random_range(0..self.world_size);
        let column = rand::random_range(0..self.world_size);

        self.data[row][column].state = rand::random();
    }
}

impl std::fmt::Display for WorldCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.data {
            for cell in row {
                write!(f, "{:02x} ", cell.state)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

pub(crate) struct TwoWorldDisplay<'a> {
    pub(crate) server: &'a WorldCells,
    pub(crate) client: &'a WorldCells,
}

impl<'a> std::fmt::Display for TwoWorldDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let world_size = self.server.world_size;

        // each row takes up 2 hex chars and a space
        let row_print_len = world_size * 3;

        for world_name in ["server", "client"] {
            let mut padded = world_name.to_string();
            while padded.len() < row_print_len {
                padded.push(' ');
            }
            write!(f, "{padded}\t")?;
        }
        writeln!(f)?;

        for row in 0..world_size {
            for world in [&self.server, &self.client] {
                for column in 0..world_size {
                    let cell = &world.data[row][column];

                    write!(f, "{:02x} ", cell.state)?;
                }

                write!(f, "\t")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct WorldCell {
    pub state: u8,
}

impl WorldCell {
    fn new() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

impl Default for WorldCell {
    fn default() -> Self {
        Self::new()
    }
}
