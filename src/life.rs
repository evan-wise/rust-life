use rand::random;
use rustc_hash::FxHashMap;

#[derive(Clone, Debug)]
pub enum LifePattern {
    Glider,
    Blinker,
    Beacon,
    Random(usize),
}

#[derive(Clone, Debug)]
pub struct LifeWorld {
    active_cells: FxHashMap<(i32, i32), bool>,
    pub generations: usize,
}

impl LifeWorld {
    pub fn new() -> LifeWorld {
        LifeWorld {
            active_cells: FxHashMap::default(),
            generations: 0,
        }
    }

    pub fn from(pattern: &LifePattern) -> LifeWorld {
        let mut world = LifeWorld::new();
        match pattern {
            LifePattern::Glider => {
                world.raise_cell(0, 0);
                world.raise_cell(1, 0);
                world.raise_cell(2, 0);
                world.raise_cell(2, 1);
                world.raise_cell(1, 2);
            }
            LifePattern::Blinker => {
                world.raise_cell(0, 0);
                world.raise_cell(0, 1);
                world.raise_cell(0, 2);
            }
            LifePattern::Beacon => {
                world.raise_cell(0, 0);
                world.raise_cell(0, 1);
                world.raise_cell(1, 0);
                world.raise_cell(1, 1);
                world.raise_cell(2, 2);
                world.raise_cell(3, 2);
                world.raise_cell(2, 3);
                world.raise_cell(3, 3);
            }
            LifePattern::Random(size) => {
                let side = (*size as f64).sqrt().round() as i32;
                for _ in 0..*size {
                    let x = random::<i32>() % side;
                    let y = random::<i32>() % side;
                    world.raise_cell(x, y);
                }
            }
        }
        world
    }

    pub fn raise_cell(&mut self, x: i32, y: i32) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    self.active_cells.insert((x, y), true);
                } else {
                    self.active_cells.entry((x + dx, y + dy)).or_insert(false);
                }
            }
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<bool> {
        match self.active_cells.get(&(x, y)) {
            Some(cell) => Some(*cell),
            None => None,
        }
    }

    pub fn get_neighbors(&self, x: i32, y: i32) -> Vec<bool> {
        let mut neighbors = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if let Some(cell) = self.get(x + dx, y + dy) {
                    neighbors.push(cell);
                }
            }
        }
        neighbors
    }

    pub fn evolve(&mut self) {

        let mut deltas = Vec::new();
        for (pos, cell) in &self.active_cells {
            let &(x, y) = pos;
            let live_neighbors = self
                .get_neighbors(x, y)
                .into_iter()
                .filter(|c| *c)
                .count();
            match (cell, live_neighbors) {
                (true, 2) | (true, 3) => {}
                (true, _) => {
                    deltas.push((Some(false), *pos));
                }
                (false, 3) => {
                    deltas.push((Some(true), *pos));
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let new = (x + dx, y + dy);
                            match self.active_cells.get(&new) {
                                None => deltas.push((Some(false), new)),
                                Some(_) => continue,
                            }
                        }
                    }
                }
                (false, 0) => {
                    deltas.push((None, *pos));
                }
                (false, _) => {
                    deltas.push((Some(false), *pos));
                }
            }
        }
        for (maybe_change, pos) in deltas {
            if let Some(change) = maybe_change {
                self.active_cells.insert(pos, change);
            } else {
                self.active_cells.remove(&pos);
            }
        }
        self.generations += 1;
    }

    pub fn num_alive(&self) -> i32 {
        let mut count = 0;
        for &cell in self.active_cells.values() {
            if cell {
                count += 1;
            }
        }
        count
    }
}
