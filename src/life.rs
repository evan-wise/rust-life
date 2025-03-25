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
                world.raise(0, 0);
                world.raise(1, 0);
                world.raise(2, 0);
                world.raise(2, 1);
                world.raise(1, 2);
            }
            LifePattern::Blinker => {
                world.raise(0, 0);
                world.raise(0, 1);
                world.raise(0, 2);
            }
            LifePattern::Beacon => {
                world.raise(0, 0);
                world.raise(0, 1);
                world.raise(1, 0);
                world.raise(1, 1);
                world.raise(2, 2);
                world.raise(3, 2);
                world.raise(2, 3);
                world.raise(3, 3);
            }
            LifePattern::Random(size) => {
                let side = (*size as f64).sqrt().round() as i32;
                for _ in 0..*size {
                    let x = random::<i32>() % side;
                    let y = random::<i32>() % side;
                    world.raise(x, y);
                }
            }
        }
        world
    }

    pub fn raise(&mut self, x: i32, y: i32) {
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

    pub fn lower(&mut self, x: i32, y: i32) {
        self.active_cells.insert((x, y), false);
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
            let live_neighbors = self.get_neighbors(x, y).into_iter().filter(|c| *c).count();
            match (cell, live_neighbors) {
                (true, 2) | (true, 3) => (),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raise_at_single_locations() {
        for [x, y] in [[0, 0], [0, 1], [1, 0], [1, 1]] {
            let mut world = LifeWorld::new();
            world.raise(x, y);
            assert_eq!(world.get(x, y), Some(true));
            assert_eq!(world.get(x + 1, y), Some(false));
            assert_eq!(world.get(x - 1, y), Some(false));
            assert_eq!(world.get(x, y + 1), Some(false));
            assert_eq!(world.get(x, y - 1), Some(false));
            assert_eq!(world.get(x + 1, y + 1), Some(false));
            assert_eq!(world.get(x + 1, y - 1), Some(false));
            assert_eq!(world.get(x - 1, y - 1), Some(false));
            assert_eq!(world.get(x - 1, y + 1), Some(false));
            assert_eq!(world.get(x, y + 2), None);
            assert_eq!(world.get(x, y - 2), None);
            assert_eq!(world.get(x + 2, y), None);
            assert_eq!(world.get(x - 2, y), None);
        }
    }

    #[test]
    fn compute_counts_after_raising() {
        for expected in [1, 2, 5, 10, 25] {
            let mut world = LifeWorld::new();
            for i in 0..expected {
                world.raise(i, 0);
            }
            assert_eq!(world.num_alive(), expected);
        }
    }

    #[test]
    fn increment_generations_on_evolve() {
        for expected in [1, 2, 5, 10, 25] {
            let mut world = LifeWorld::new();
            for _ in 0..expected {
                world.evolve();
            }
            assert_eq!(world.generations, expected);
        }
    }

    #[test]
    fn live_cell_with_n_living_neighbors() {
        let positions = [
            [0, 1],
            [1, 0],
            [1, 1],
            [0, -1],
            [-1, 0],
            [-1, -1],
            [-1, 1],
            [1, -1],
        ];
        for n in 0..=8 {
            let mut world = LifeWorld::new();
            world.raise(0, 0);
            for [x, y] in &positions[0..n] {
                world.raise(*x, *y);
            }
            world.evolve();
            match n {
                2 | 3 => assert_eq!(world.get(0, 0), Some(true)),
                _ => assert_eq!(world.get(0, 0), Some(false)),
            }
        }
    }

    #[test]
    fn dead_cell_with_n_living_neighbors() {
        let positions = [
            [0, 1],
            [1, 0],
            [1, 1],
            [0, -1],
            [-1, 0],
            [-1, -1],
            [-1, 1],
            [1, -1],
        ];
        for n in 0..=8 {
            let mut world = LifeWorld::new();
            for [x, y] in &positions[0..n] {
                world.raise(*x, *y);
            }
            if n == 0 {
                world.lower(0, 0);
            }
            world.evolve();
            match n {
                3 => assert_eq!(world.get(0, 0), Some(true)),
                0 => assert_eq!(world.get(0, 0), None),
                _ => assert_eq!(world.get(0, 0), Some(false)),
            }
        }
    }
}
