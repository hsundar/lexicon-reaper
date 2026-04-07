use rand::Rng;
use serde::{Deserialize, Serialize};

use super::tile::{random_letter, random_tile, Tile, TileEffect};

pub const BOARD_ROWS: usize = 3;
pub const BOARD_COLS: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    tiles: Vec<Vec<Tile>>,
    next_select_order: u8,
}

impl Board {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut tiles = Vec::with_capacity(BOARD_ROWS);
        for _ in 0..BOARD_ROWS {
            let mut row = Vec::with_capacity(BOARD_COLS);
            for _ in 0..BOARD_COLS {
                row.push(random_tile(rng));
            }
            tiles.push(row);
        }
        Self {
            tiles,
            next_select_order: 1,
        }
    }

    pub fn tile(&self, row: usize, col: usize) -> &Tile {
        &self.tiles[row][col]
    }

    pub fn tile_mut(&mut self, row: usize, col: usize) -> &mut Tile {
        &mut self.tiles[row][col]
    }

    /// Toggle selection of a tile. Returns true if the tile was toggled.
    pub fn toggle_select(&mut self, row: usize, col: usize) -> bool {
        let tile = &mut self.tiles[row][col];
        if !tile.is_selectable() {
            return false;
        }
        if tile.selected {
            let removed_order = tile.select_order;
            tile.selected = false;
            tile.select_order = None;
            if let Some(removed) = removed_order {
                for r in 0..BOARD_ROWS {
                    for c in 0..BOARD_COLS {
                        if let Some(order) = self.tiles[r][c].select_order {
                            if order > removed {
                                self.tiles[r][c].select_order = Some(order - 1);
                            }
                        }
                    }
                }
                self.next_select_order -= 1;
            }
        } else {
            tile.selected = true;
            tile.select_order = Some(self.next_select_order);
            self.next_select_order += 1;
        }
        true
    }

    /// Undo the last tile selection. Returns true if a tile was deselected.
    pub fn undo_last_select(&mut self) -> bool {
        if self.next_select_order <= 1 {
            return false;
        }
        let last_order = self.next_select_order - 1;
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                if self.tiles[r][c].select_order == Some(last_order) {
                    self.tiles[r][c].selected = false;
                    self.tiles[r][c].select_order = None;
                    self.next_select_order -= 1;
                    return true;
                }
            }
        }
        false
    }

    /// Get the word formed by currently selected tiles, in selection order
    pub fn selected_word(&self) -> String {
        let mut selected: Vec<(u8, char)> = Vec::new();
        for row in &self.tiles {
            for tile in row {
                if tile.selected {
                    if let Some(order) = tile.select_order {
                        selected.push((order, tile.letter));
                    }
                }
            }
        }
        selected.sort_by_key(|(order, _)| *order);
        selected.iter().map(|(_, ch)| ch).collect()
    }

    /// Get selected tiles in order, returning (row, col) pairs
    pub fn selected_positions(&self) -> Vec<(usize, usize)> {
        let mut selected: Vec<(u8, usize, usize)> = Vec::new();
        for (r, row) in self.tiles.iter().enumerate() {
            for (c, tile) in row.iter().enumerate() {
                if tile.selected {
                    if let Some(order) = tile.select_order {
                        selected.push((order, r, c));
                    }
                }
            }
        }
        selected.sort_by_key(|(order, _, _)| *order);
        selected.iter().map(|(_, r, c)| (*r, *c)).collect()
    }

    /// Get the effective values of selected tiles in order
    pub fn selected_tile_values(&self) -> Vec<f64> {
        let positions = self.selected_positions();
        positions
            .iter()
            .map(|(r, c)| self.tiles[*r][*c].effective_value())
            .collect()
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                tile.selected = false;
                tile.select_order = None;
            }
        }
        self.next_select_order = 1;
    }

    /// Consume selected tiles and replace with new random ones.
    /// Has a small chance to spawn crystal tiles.
    pub fn consume_selected<R: Rng>(&mut self, rng: &mut R, crystal_bonus: f64) {
        let crystal_chance = 0.08 + crystal_bonus;
        for row in &mut self.tiles {
            for tile in row {
                if tile.selected {
                    *tile = random_tile(rng);
                    // Chance to spawn a crystal tile
                    if rng.gen_bool(crystal_chance.clamp(0.0, 1.0)) {
                        tile.effect = TileEffect::Crystal;
                        tile.timer = Some(3);
                    }
                }
            }
        }
        self.next_select_order = 1;
    }

    /// Shuffle all tiles (costs a turn)
    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        for row in &mut self.tiles {
            for tile in row {
                *tile = random_tile(rng);
            }
        }
        self.next_select_order = 1;
    }

    /// Cleanse all negative tile effects (Purity potion)
    pub fn cleanse_effects(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                match tile.effect {
                    TileEffect::Normal | TileEffect::Crystal => {}
                    _ => {
                        tile.effect = TileEffect::Normal;
                        tile.timer = None;
                    }
                }
            }
        }
    }

    /// Tick all tile effects (called at end of each turn).
    /// Handles whirlwind letter changes, plague spreading, duplicator copying, timer expiry.
    pub fn tick_effects<R: Rng>(&mut self, rng: &mut R) {
        let mut plague_spread: Vec<(usize, usize)> = Vec::new();
        let mut duplicator_targets: Vec<(usize, usize, char)> = Vec::new();

        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                if let Some(timer) = self.tiles[r][c].timer.as_mut() {
                    if *timer > 0 {
                        *timer -= 1;
                    }
                }

                let timer_expired = self.tiles[r][c].timer == Some(0);

                match self.tiles[r][c].effect {
                    TileEffect::Whirlwind if !timer_expired => {
                        self.tiles[r][c].letter = random_letter(rng);
                    }
                    TileEffect::Plague if !timer_expired => {
                        for (dr, dc) in &[(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                            let nr = r as i32 + dr;
                            let nc = c as i32 + dc;
                            if nr >= 0
                                && nr < BOARD_ROWS as i32
                                && nc >= 0
                                && nc < BOARD_COLS as i32
                            {
                                let nr = nr as usize;
                                let nc = nc as usize;
                                if self.tiles[nr][nc].effect == TileEffect::Normal {
                                    plague_spread.push((nr, nc));
                                }
                            }
                        }
                    }
                    TileEffect::Duplicator if !timer_expired => {
                        let letter = self.tiles[r][c].letter;
                        for (dr, dc) in &[(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                            let nr = r as i32 + dr;
                            let nc = c as i32 + dc;
                            if nr >= 0
                                && nr < BOARD_ROWS as i32
                                && nc >= 0
                                && nc < BOARD_COLS as i32
                            {
                                let nr = nr as usize;
                                let nc = nc as usize;
                                if self.tiles[nr][nc].effect == TileEffect::Normal {
                                    duplicator_targets.push((nr, nc, letter));
                                    break; // Only one adjacent tile per turn
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // Remove expired effects
                if timer_expired {
                    self.tiles[r][c].effect = TileEffect::Normal;
                    self.tiles[r][c].timer = None;
                }
            }
        }

        // Apply plague spread
        for (r, c) in plague_spread {
            if self.tiles[r][c].effect == TileEffect::Normal {
                self.tiles[r][c].effect = TileEffect::Plague;
                self.tiles[r][c].timer = Some(3);
            }
        }

        // Apply duplicator: copy letter to adjacent tile
        for (r, c, letter) in duplicator_targets {
            if self.tiles[r][c].effect == TileEffect::Normal {
                self.tiles[r][c].letter = letter;
            }
        }

        // Purity book: chance to cleanse one random bad tile
        // (called from combat.rs, not here — this is just board logic)
    }

    /// Apply a tile effect to random normal tiles on the board
    pub fn apply_effect_to_random<R: Rng>(
        &mut self,
        rng: &mut R,
        effect: TileEffect,
        count: usize,
        timer: u8,
    ) {
        let mut normals: Vec<(usize, usize)> = Vec::new();
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                if self.tiles[r][c].effect == TileEffect::Normal {
                    normals.push((r, c));
                }
            }
        }

        use rand::seq::SliceRandom;
        normals.shuffle(rng);

        for (r, c) in normals.into_iter().take(count) {
            self.tiles[r][c].effect = effect;
            self.tiles[r][c].timer = Some(timer);
        }
    }

    /// Cleanse one random negative tile effect (Purity book proc)
    pub fn cleanse_one_random<R: Rng>(&mut self, rng: &mut R) -> bool {
        let mut bad_tiles: Vec<(usize, usize)> = Vec::new();
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                match self.tiles[r][c].effect {
                    TileEffect::Normal | TileEffect::Crystal => {}
                    _ => bad_tiles.push((r, c)),
                }
            }
        }
        if bad_tiles.is_empty() {
            return false;
        }
        use rand::seq::SliceRandom;
        if let Some(&(r, c)) = bad_tiles.choose(rng) {
            self.tiles[r][c].effect = TileEffect::Normal;
            self.tiles[r][c].timer = None;
            return true;
        }
        false
    }

    /// Count how many selected tiles have a specific effect
    pub fn count_selected_with_effect(&self, effect: TileEffect) -> usize {
        self.tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter(|t| t.selected && t.effect == effect)
            .count()
    }

    pub fn selection_count(&self) -> usize {
        self.tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter(|t| t.selected)
            .count()
    }
}
