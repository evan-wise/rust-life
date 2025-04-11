use rand::random;
use rustc_hash::FxHashMap;

#[derive(PartialEq, Clone, Debug)]
pub struct LifeCell {
    pub alive: bool,
    pub num_neighbors: u8,
}

impl LifeCell {
    pub fn new(alive: bool) -> LifeCell {
        LifeCell {
            alive,
            num_neighbors: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LifeWorld {
    active_cells: FxHashMap<(i32, i32), LifeCell>,
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
            LifePattern::Blank => (),
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
        self.set_cell(x, y, true);
    }

    pub fn lower(&mut self, x: i32, y: i32) {
        self.set_cell(x, y, false);
    }

    pub fn toggle(&mut self, x: i32, y: i32) {
        self.set_cell(x, y, !self.alive(x, y));
    }

    pub fn get(&self, x: i32, y: i32) -> Option<bool> {
        self.active_cells.get(&(x, y)).map(|cell| cell.alive)
    }

    pub fn alive(&self, x: i32, y: i32) -> bool {
        self.get(x, y).unwrap_or(false)
    }

    pub fn evolve(&mut self) {
        let mut deltas = Vec::new();
        for (pos, cell) in &self.active_cells {
            match cell {
                LifeCell {
                    alive: true,
                    num_neighbors: 2 | 3,
                } => (),
                LifeCell {
                    alive: true,
                    num_neighbors: _,
                } => {
                    deltas.push((false, *pos));
                }
                LifeCell {
                    alive: false,
                    num_neighbors: 3,
                } => {
                    deltas.push((true, *pos));
                }
                LifeCell {
                    alive: false,
                    num_neighbors: _,
                } => (),
            }
        }
        for (change, pos) in deltas {
            let (x, y) = pos;
            self.set_cell(x, y, change);
        }
        self.generations += 1;
    }

    pub fn num_alive(&self) -> i32 {
        let mut count = 0;
        for cell in self.active_cells.values() {
            if cell.alive {
                count += 1;
            }
        }
        count
    }

    fn set_cell(&mut self, x: i32, y: i32, alive: bool) {
        let dirty: bool;
        let mut new = false;

        match self.active_cells.entry((x, y)) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                let cell = occupied.get_mut();
                dirty = cell.alive != alive;
                cell.alive = alive;
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(LifeCell::new(alive));
                dirty = alive;
                new = alive;
            }
        }

        if !dirty {
            return;
        }

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if new && self.alive(x + dx, y + dy) {
                    self.active_cells
                        .entry((x, y))
                        .and_modify(|cell| cell.num_neighbors += 1);
                }
                if alive {
                    let cell = self
                        .active_cells
                        .entry((x + dx, y + dy))
                        .or_insert(LifeCell::new(false));
                    cell.num_neighbors += 1;
                } else {
                    match self.active_cells.entry((x + dx, y + dy)) {
                        std::collections::hash_map::Entry::Occupied(mut occupied) => {
                            let cell = occupied.get_mut();
                            cell.num_neighbors -= 1;
                            if cell.num_neighbors == 0 && !cell.alive {
                                occupied.remove();
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        match self.active_cells.entry((x, y)) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                let cell = occupied.get_mut();
                if cell.num_neighbors == 0 && !cell.alive {
                    occupied.remove();
                }
            }
            _ => (),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LifePattern {
    Blank,
    Glider,
    Blinker,
    Beacon,
    Random(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    const POSITIONS: [[i32; 2]; 8] = [
        [0, 1],
        [1, 0],
        [1, 1],
        [0, -1],
        [-1, 0],
        [-1, -1],
        [-1, 1],
        [1, -1],
    ];

    #[test]
    fn raise_raises_cells() {
        for [x, y] in POSITIONS {
            let mut world = LifeWorld::new();
            world.raise(x, y);
            assert_eq!(world.alive(x, y), true);
        }
    }

    #[test]
    fn lower_lowers_cells() {
        for [x, y] in POSITIONS {
            let mut world = LifeWorld::new();
            world.lower(x, y);
            assert_eq!(world.alive(x, y), false);
        }
    }

    #[test]
    fn toggle_raises_and_lowers_cells() {
        for [x, y] in POSITIONS {
            let mut world = LifeWorld::new();
            world.toggle(x, y);
            assert_eq!(world.alive(x, y), true);
            world.toggle(x, y);
            assert_eq!(world.alive(x, y), false);
        }
    }

    #[test]
    fn raise_fills_surrounding_cells() {
        for [x, y] in POSITIONS {
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
    fn lower_clears_cells() {
        for [x, y] in POSITIONS {
            let mut world = LifeWorld::new();
            world.raise(x, y);
            world.lower(x, y);
            assert_eq!(world.get(x, y), None);
            assert_eq!(world.get(x + 1, y), None);
            assert_eq!(world.get(x - 1, y), None);
            assert_eq!(world.get(x, y + 1), None);
            assert_eq!(world.get(x, y - 1), None);
            assert_eq!(world.get(x + 1, y + 1), None);
            assert_eq!(world.get(x + 1, y - 1), None);
            assert_eq!(world.get(x - 1, y - 1), None);
            assert_eq!(world.get(x - 1, y + 1), None);
            assert_eq!(world.get(x, y + 2), None);
            assert_eq!(world.get(x, y - 2), None);
            assert_eq!(world.get(x + 2, y), None);
            assert_eq!(world.get(x - 2, y), None);
        }
    }

    #[test]
    fn toggle_clears_cells() {
        for [x, y] in POSITIONS {
            let mut world = LifeWorld::new();
            world.toggle(x, y);
            world.toggle(x, y);
            assert_eq!(world.get(x, y), None);
            assert_eq!(world.get(x + 1, y), None);
            assert_eq!(world.get(x - 1, y), None);
            assert_eq!(world.get(x, y + 1), None);
            assert_eq!(world.get(x, y - 1), None);
            assert_eq!(world.get(x + 1, y + 1), None);
            assert_eq!(world.get(x + 1, y - 1), None);
            assert_eq!(world.get(x - 1, y - 1), None);
            assert_eq!(world.get(x - 1, y + 1), None);
            assert_eq!(world.get(x, y + 2), None);
            assert_eq!(world.get(x, y - 2), None);
            assert_eq!(world.get(x + 2, y), None);
            assert_eq!(world.get(x - 2, y), None);
        }
    }

    #[test]
    fn toggle_clears_adjacent_cells() {
        for [x, y] in POSITIONS {
            for [dx, dy] in [[0, 1], [1, 1], [1, 0]] {
                let mut world = LifeWorld::new();
                world.toggle(x, y);
                world.toggle(x + dx, y + dy);
                world.toggle(x, y);
                world.toggle(x + dx, y + dy);
                assert_eq!(world.get(x + 1, y), None);
                assert_eq!(world.get(x - 1, y), None);
                assert_eq!(world.get(x, y + 1), None);
                assert_eq!(world.get(x, y - 1), None);
                assert_eq!(world.get(x + 1, y + 1), None);
                assert_eq!(world.get(x + 1, y - 1), None);
                assert_eq!(world.get(x - 1, y - 1), None);
                assert_eq!(world.get(x - 1, y + 1), None);
                assert_eq!(world.get(x + 2, y), None);
                assert_eq!(world.get(x + 2, y + 1), None);
                assert_eq!(world.get(x + 2, y + 2), None);
                assert_eq!(world.get(x + 1, y + 2), None);
                assert_eq!(world.get(x, y + 2), None);
            }
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
        for n in 0..=8 {
            let mut world = LifeWorld::new();
            world.raise(0, 0);
            for [x, y] in &POSITIONS[0..n] {
                world.raise(*x, *y);
            }
            world.evolve();
            match n {
                2 | 3 => assert_eq!(world.alive(0, 0), true),
                _ => assert_eq!(world.alive(0, 0), false),
            }
        }
    }

    #[test]
    fn dead_cell_with_n_living_neighbors() {
        for n in 0..=8 {
            let mut world = LifeWorld::new();
            for [x, y] in &POSITIONS[0..n] {
                world.raise(*x, *y);
            }
            if n == 0 {
                world.lower(0, 0);
            }
            world.evolve();
            match n {
                3 => assert_eq!(world.alive(0, 0), true),
                _ => assert_eq!(world.alive(0, 0), false),
            }
        }
    }
}
