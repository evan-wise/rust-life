use std::collections::HashMap;

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct LifeCell {
    pub alive: bool,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub struct LifeWorld {
    active_cells: HashMap<(i32,i32), LifeCell>,
}

impl LifeWorld {
    pub fn new() -> LifeWorld {
        LifeWorld {
            active_cells: HashMap::new(),
        }
    }

    pub fn raise_cell(&mut self, x: i32, y: i32) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    self.active_cells.insert((x,y), LifeCell { alive: true, x, y });
                } else {
                    self.active_cells.entry((x + dx, y + dy)).or_insert(LifeCell { alive: false, x: x + dx, y: y + dy });
                }
            }
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<LifeCell> {
        match self.active_cells.get(&(x,y)) {
            Some(cell) => Some(*cell),
            None => None,
        }
    }

    pub fn get_neighbors(&self, x: i32, y: i32) -> Vec<LifeCell> {
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
        let mut new_cells = HashMap::new();

        for (pos, cell) in &self.active_cells {
            let neighbors = self.get_neighbors(cell.x, cell.y);
            let live_neighbors = neighbors.iter().filter(|c| c.alive).count();
            match (cell.alive, live_neighbors) {
                (true, 2) | (true, 3) => { new_cells.insert(*pos, *cell); },
                (true, 0) => (),
                (true, _) => {
                    new_cells.insert(*pos, LifeCell { alive: false, ..*cell});
                },
                (false, 3) => {
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            match new_cells.get(&(cell.x + dx, cell.y + dy)) {
                                Some(_) => continue,
                                None if dx == 0 && dy == 0 => new_cells.insert((cell.x + dx, cell.y + dy), LifeCell { alive: true, ..*cell }),
                                None => new_cells.insert((cell.x + dx, cell.y + dy), LifeCell { alive: false, x: cell.x + dx, y: cell.y + dy }),
                            };
                        }
                    }
                },
                (false, 0) => (),
                (false, _) => { new_cells.insert(*pos, *cell); },
            }
        }
        self.active_cells = new_cells;
    }
} 
    
