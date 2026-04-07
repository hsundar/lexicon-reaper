use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::dict::definitions::DefinitionCache;
use crate::dict::Dictionary;
use crate::event::Event;
use crate::game::board::{Board, BOARD_COLS, BOARD_ROWS};
use crate::game::combat::{resolve_enemy_turn, resolve_player_word};
use crate::game::equipment::Potion;
use crate::game::player::Player;
use crate::game::shop::{self, ShopItem};
use crate::game::state::*;
use crate::game::stats::RunStats;
use crate::generation::stage::{self, Biome};
use crate::save;

pub struct ShopScreenState {
    pub items: Vec<ShopItem>,
    pub selected_index: usize,
    pub message: Option<String>,
}

pub struct App {
    pub player: Player,
    pub combat: Option<CombatState>,
    pub shop_state: Option<ShopScreenState>,
    pub stage_number: u32,
    pub dictionary: Dictionary,
    pub rng: ChaCha8Rng,
    pub message_log: Vec<LogEntry>,
    pub menu_selection: usize,
    pub inventory_selection: usize,
    pub biome: Biome,
    pub has_save: bool,
    pub stats: RunStats,
    pub show_help: bool,
    pub show_stats: bool,
    pub definitions: DefinitionCache,
    screen: Screen,
    previous_screen: Screen,
    pub should_quit: bool,
}

impl App {
    pub fn new(dictionary: Dictionary) -> Self {
        let has_save = save::has_save();
        Self {
            player: Player::new(),
            combat: None,
            shop_state: None,
            stage_number: 1,
            dictionary,
            rng: ChaCha8Rng::from_entropy(),
            message_log: Vec::new(),
            menu_selection: 0,
            inventory_selection: 0,
            biome: Biome::Forest,
            has_save,
            stats: RunStats::default(),
            show_help: false,
            show_stats: false,
            definitions: DefinitionCache::new(),
            screen: Screen::Title,
            previous_screen: Screen::Title,
            should_quit: false,
        }
    }

    pub fn current_screen(&self) -> Screen {
        self.screen
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Tick => self.handle_tick(),
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        // Toggle help overlay
        if key.code == KeyCode::Char('?') || key.code == KeyCode::F(1) {
            if self.screen == Screen::Combat || self.screen == Screen::Shop {
                self.show_help = !self.show_help;
                return;
            }
        }

        // Toggle stats overlay
        if key.code == KeyCode::Tab && self.screen == Screen::Combat {
            self.show_stats = !self.show_stats;
            return;
        }

        // Dismiss overlays
        if self.show_help || self.show_stats {
            self.show_help = false;
            self.show_stats = false;
            return;
        }

        match self.screen {
            Screen::Title => self.handle_title_key(key),
            Screen::Combat => self.handle_combat_key(key),
            Screen::Map => self.handle_map_key(key),
            Screen::Shop => self.handle_shop_key(key),
            Screen::Inventory => self.handle_inventory_key(key),
            Screen::GameOver => self.handle_game_over_key(key),
        }
    }

    fn handle_tick(&mut self) {
        // Check for deferred definitions
        self.check_pending_definitions();

        if self.screen != Screen::Combat {
            return;
        }

        let combat = match &mut self.combat {
            Some(c) => c,
            None => return,
        };

        match &mut combat.phase {
            CombatPhase::PlayerAttackAnim {
                ticks_remaining,
                damage: _,
                is_crit: _,
                word: _,
            } => {
                if *ticks_remaining > 0 {
                    *ticks_remaining -= 1;
                } else {
                    combat.phase = CombatPhase::EnemyTurn {
                        ticks_remaining: 8,
                    };
                }
            }
            CombatPhase::EnemyTurn { ticks_remaining } => {
                if *ticks_remaining > 0 {
                    *ticks_remaining -= 1;
                } else {
                    // Resolve enemy attack and transition to hit animation
                    let enemy = &mut combat.enemies[combat.current_enemy_idx];
                    let result =
                        resolve_enemy_turn(&mut combat.board, &mut self.player, enemy, &mut self.rng);

                    if result.dodged {
                        self.stats.record_dodge();
                        self.message_log.push(LogEntry {
                            text: "You dodged the attack!".to_string(),
                            color: LogColor::Info,
                        });
                    } else if result.damage_dealt > 0 {
                        self.stats.record_damage_taken(result.damage_dealt);
                        self.message_log.push(LogEntry {
                            text: format!(
                                "{} attacks for {} damage!",
                                combat.current_enemy().name,
                                result.damage_dealt
                            ),
                            color: LogColor::EnemyDamage,
                        });
                    }

                    for msg in &result.abilities_used {
                        self.message_log.push(LogEntry {
                            text: format!("{}: {}", combat.current_enemy().name, msg),
                            color: LogColor::Warning,
                        });
                    }

                    // Show hit/dodge reaction animation
                    combat.phase = CombatPhase::EnemyHitAnim {
                        damage: result.damage_dealt,
                        dodged: result.dodged,
                        ticks_remaining: 6,
                    };
                }
            }
            CombatPhase::EnemyHitAnim { ticks_remaining, .. } => {
                if *ticks_remaining > 0 {
                    *ticks_remaining -= 1;
                } else {
                    if !self.player.is_alive() {
                        combat.phase = CombatPhase::Defeat;
                        self.message_log.push(LogEntry {
                            text: "You have been defeated...".to_string(),
                            color: LogColor::EnemyDamage,
                        });
                    } else {
                        combat.phase = CombatPhase::PlayerInput;
                        combat.turn_count += 1;
                    }
                }
            }
            CombatPhase::EnemyDefeated { ticks_remaining } => {
                if *ticks_remaining > 0 {
                    *ticks_remaining -= 1;
                } else {
                    let was_boss = combat.current_enemy().is_boss;
                    self.stats.record_enemy_defeated(was_boss);

                    if combat.has_next_enemy() {
                        // Walking transition to next enemy
                        self.message_log.push(LogEntry {
                            text: "Moving to next encounter...".to_string(),
                            color: LogColor::Info,
                        });
                        combat.phase = CombatPhase::Walking {
                            ticks_remaining: 20,
                        };
                    } else {
                        let gems = crate::generation::loot::stage_gem_reward(
                            self.stage_number,
                            was_boss,
                            &self.player,
                        );
                        combat.phase = CombatPhase::StageVictory {
                            gems_earned: gems,
                            ticks_remaining: 20,
                        };
                    }
                }
            }
            CombatPhase::Walking { ticks_remaining } => {
                if *ticks_remaining > 0 {
                    *ticks_remaining -= 1;
                } else {
                    // Advance to next enemy
                    combat.advance_enemy();
                    self.message_log.push(LogEntry {
                        text: format!(
                            "A new foe appears: {}!",
                            combat.current_enemy().name
                        ),
                        color: LogColor::Warning,
                    });
                    combat.phase = CombatPhase::PlayerInput;
                }
            }
            CombatPhase::StageVictory {
                gems_earned,
                ticks_remaining,
            } => {
                if *ticks_remaining > 0 {
                    *ticks_remaining -= 1;
                } else {
                    let gems = *gems_earned;
                    self.player.gems += gems;
                    self.stats.gems_earned += gems;
                    self.message_log.push(LogEntry {
                        text: format!("Stage {} complete! Earned {} gems!", self.stage_number, gems),
                        color: LogColor::Info,
                    });
                    let _ = save::save_game(&self.player, self.stage_number + 1, &self.stats);
                    self.enter_map();
                }
            }
            CombatPhase::Defeat => {
                let _ = save::delete_save();
                self.screen = Screen::GameOver;
            }
            _ => {}
        }
    }

    // --- Title Screen ---
    fn handle_title_key(&mut self, key: KeyEvent) {
        let max_option = if self.has_save { 2 } else { 1 };
        match key.code {
            KeyCode::Up => {
                if self.menu_selection > 0 {
                    self.menu_selection -= 1;
                }
            }
            KeyCode::Down => {
                if self.menu_selection < max_option {
                    self.menu_selection += 1;
                }
            }
            KeyCode::Enter => {
                if self.has_save {
                    match self.menu_selection {
                        0 => self.continue_game(),
                        1 => self.new_game(),
                        2 => self.should_quit = true,
                        _ => {}
                    }
                } else {
                    match self.menu_selection {
                        0 => self.new_game(),
                        1 => self.should_quit = true,
                        _ => {}
                    }
                }
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    // --- Combat ---
    fn handle_combat_key(&mut self, key: KeyEvent) {
        let combat = match &mut self.combat {
            Some(c) => c,
            None => return,
        };

        if !matches!(combat.phase, CombatPhase::PlayerInput) {
            return;
        }

        match key.code {
            KeyCode::Up => {
                if combat.cursor.0 > 0 {
                    combat.cursor.0 -= 1;
                }
            }
            KeyCode::Down => {
                if combat.cursor.0 < BOARD_ROWS - 1 {
                    combat.cursor.0 += 1;
                }
            }
            KeyCode::Left => {
                if combat.cursor.1 > 0 {
                    combat.cursor.1 -= 1;
                }
            }
            KeyCode::Right => {
                if combat.cursor.1 < BOARD_COLS - 1 {
                    combat.cursor.1 += 1;
                }
            }
            KeyCode::Char(' ') => {
                let (row, col) = combat.cursor;
                combat.board.toggle_select(row, col);
            }
            KeyCode::Backspace => {
                // Undo last tile selection
                combat.board.undo_last_select();
            }
            KeyCode::Enter => {
                self.try_submit_word();
            }
            KeyCode::Esc => {
                combat.board.clear_selection();
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                combat.board.shuffle(&mut self.rng);
                self.stats.tiles_shuffled += 1;
                self.message_log.push(LogEntry {
                    text: "Board shuffled! (turn used)".to_string(),
                    color: LogColor::Info,
                });
                combat.phase = CombatPhase::EnemyTurn {
                    ticks_remaining: 8,
                };
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.use_potion();
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn use_potion(&mut self) {
        if !self.player.has_potions() {
            self.message_log.push(LogEntry {
                text: "No potions available!".to_string(),
                color: LogColor::Warning,
            });
            return;
        }

        // Check if it's a purity potion to also cleanse board
        let is_purity = matches!(self.player.potions.first(), Some(Potion::Purity));
        let is_lifesteal = matches!(self.player.potions.first(), Some(Potion::Lifesteal));

        if let Some(msg) = self.player.use_potion(0) {
            self.stats.potions_used += 1;
            self.message_log.push(LogEntry {
                text: msg,
                color: LogColor::Heal,
            });

            if is_purity {
                if let Some(combat) = &mut self.combat {
                    combat.board.cleanse_effects();
                    self.message_log.push(LogEntry {
                        text: "All negative tile effects cleansed!".to_string(),
                        color: LogColor::Heal,
                    });
                }
            }

            if is_lifesteal {
                if let Some(combat) = &mut self.combat {
                    combat.lifesteal_active = true;
                }
            }
        }
    }

    // --- Map ---
    fn handle_map_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                // Go to shop
                self.enter_shop();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Skip shop, go directly to next stage
                self.stage_number += 1;
                self.start_combat();
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    // --- Shop ---
    fn handle_shop_key(&mut self, key: KeyEvent) {
        let shop = match &mut self.shop_state {
            Some(s) => s,
            None => return,
        };

        match key.code {
            KeyCode::Up => {
                if shop.selected_index > 0 {
                    shop.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if shop.selected_index + 1 < shop.items.len() {
                    shop.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                let item = shop.items[shop.selected_index].clone();
                let cost = crate::game::shop::cost_for_item(&item, &self.player);
                if let Some(msg) = shop::buy_item(&item, &mut self.player) {
                    self.stats.gems_spent += cost;
                    shop.message = Some(msg);
                    shop.items = shop::generate_shop_items(self.stage_number, &self.player);
                    if shop.selected_index >= shop.items.len() {
                        shop.selected_index = shop.items.len().saturating_sub(1);
                    }
                } else {
                    shop.message = Some("Can't buy that!".to_string());
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.stage_number += 1;
                self.start_combat();
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.previous_screen = Screen::Shop;
                self.screen = Screen::Inventory;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    // --- Inventory ---
    fn handle_inventory_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.inventory_selection > 0 {
                    self.inventory_selection -= 1;
                }
            }
            KeyCode::Down => {
                let max = self.player.books.len().saturating_sub(1);
                if self.inventory_selection < max {
                    self.inventory_selection += 1;
                }
            }
            KeyCode::Delete | KeyCode::Backspace => {
                if !self.player.books.is_empty()
                    && self.inventory_selection < self.player.books.len()
                {
                    self.player.books.remove(self.inventory_selection);
                    if self.inventory_selection > 0
                        && self.inventory_selection >= self.player.books.len()
                    {
                        self.inventory_selection -= 1;
                    }
                }
            }
            KeyCode::Esc => {
                self.screen = self.previous_screen;
            }
            _ => {}
        }
    }

    // --- Game Over ---
    fn handle_game_over_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.screen = Screen::Title;
                self.menu_selection = 0;
                self.has_save = save::has_save();
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    // --- Word Submission ---
    fn try_submit_word(&mut self) {
        let combat = match &mut self.combat {
            Some(c) => c,
            None => return,
        };

        let word = combat.board.selected_word();

        if word.len() < 3 {
            self.message_log.push(LogEntry {
                text: "Words must be at least 3 letters!".to_string(),
                color: LogColor::Warning,
            });
            return;
        }

        if !self.dictionary.is_word(&word) {
            self.message_log.push(LogEntry {
                text: format!("\"{}\" is not a valid word!", word),
                color: LogColor::Warning,
            });
            return;
        }

        if let Some(constraint) = &combat.current_enemy().word_constraint {
            if !constraint.check(&word) {
                self.message_log.push(LogEntry {
                    text: format!(
                        "\"{}\" doesn't satisfy: {}",
                        word,
                        constraint.description()
                    ),
                    color: LogColor::Warning,
                });
                return;
            }
        }

        // Consume lifesteal potion flag
        let lifesteal_active = combat.lifesteal_active;
        combat.lifesteal_active = false;

        let enemy = &mut combat.enemies[combat.current_enemy_idx];
        let result = resolve_player_word(
            &mut combat.board,
            &mut self.player,
            enemy,
            &mut self.rng,
            lifesteal_active,
        );

        // Record stats
        self.stats
            .record_word(&result.word, result.damage_dealt, result.is_crit);

        // Request definition in background
        self.definitions.request(&result.word);

        // Log results
        let crit_text = if result.is_crit {
            " CRITICAL HIT!"
        } else {
            ""
        };
        self.message_log.push(LogEntry {
            text: format!(
                "\"{}\" deals {} damage!{}",
                result.word, result.damage_dealt, crit_text
            ),
            color: if result.is_crit {
                LogColor::Crit
            } else {
                LogColor::PlayerDamage
            },
        });

        // Show definition if available (from a previous lookup of the same word)
        if let Some(Some(def)) = self.definitions.get(&result.word) {
            self.message_log.push(LogEntry {
                text: format!("  \"{}\"", def),
                color: LogColor::Normal,
            });
        }

        if result.venom_bonus > 0 {
            self.message_log.push(LogEntry {
                text: format!("Venom Scythe bonus: +{} from poison tiles!", result.venom_bonus),
                color: LogColor::PlayerDamage,
            });
        }

        if result.self_damage > 0 {
            self.stats.record_damage_taken(result.self_damage);
            self.message_log.push(LogEntry {
                text: format!("Tile effects deal {} self-damage!", result.self_damage),
                color: LogColor::EnemyDamage,
            });
        }

        if result.lifesteal > 0 {
            self.message_log.push(LogEntry {
                text: format!("Lifesteal heals {} HP!", result.lifesteal),
                color: LogColor::Heal,
            });
        }

        if result.potion_lifesteal > 0 {
            self.message_log.push(LogEntry {
                text: format!("Lifesteal potion heals {} HP!", result.potion_lifesteal),
                color: LogColor::Heal,
            });
        }

        if result.purity_cleansed {
            self.message_log.push(LogEntry {
                text: "Purity book cleansed a tile!".to_string(),
                color: LogColor::Heal,
            });
        }

        if result.bonus_turn {
            self.message_log.push(LogEntry {
                text: "Speed book grants a bonus turn!".to_string(),
                color: LogColor::Info,
            });
        }

        if result.enemy_defeated {
            self.message_log.push(LogEntry {
                text: format!("{} defeated!", combat.current_enemy().name),
                color: LogColor::Info,
            });
            combat.phase = CombatPhase::EnemyDefeated {
                ticks_remaining: 15,
            };
        } else if result.bonus_turn {
            // Bonus turn: skip enemy turn, go straight back to player input
            combat.phase = CombatPhase::PlayerInput;
            combat.turn_count += 1;
        } else {
            combat.phase = CombatPhase::PlayerAttackAnim {
                damage: result.damage_dealt,
                is_crit: result.is_crit,
                word: result.word,
                ticks_remaining: 10,
            };
        }
    }

    /// Check if any pending word definitions have arrived from the background thread.
    /// If so, log them in the combat log.
    fn check_pending_definitions(&mut self) {
        if let Some(last_word) = self.stats.word_history.last().cloned() {
            if let Some(Some(def)) = self.definitions.get(&last_word) {
                // Check if we already logged this definition (avoid duplicates)
                let already_logged = self.message_log.iter().rev().take(10).any(|entry| {
                    entry.text.contains(&def)
                });
                if !already_logged {
                    self.message_log.push(LogEntry {
                        text: format!("  {}", def),
                        color: LogColor::Normal,
                    });
                }
            }
        }
    }

    // --- State Transitions ---
    fn new_game(&mut self) {
        self.player = Player::new();
        self.stage_number = 1;
        self.message_log.clear();
        self.stats = RunStats::default();
        let _ = save::delete_save();
        self.start_combat();
    }

    fn continue_game(&mut self) {
        match save::load_game() {
            Ok(Some(data)) => {
                self.player = data.player;
                self.stage_number = data.stage_number;
                self.rng = ChaCha8Rng::seed_from_u64(data.rng_seed);
                self.stats = data.stats;
                self.message_log.clear();
                self.message_log.push(LogEntry {
                    text: "Game loaded!".to_string(),
                    color: LogColor::Info,
                });
                self.start_combat();
            }
            Ok(None) => {
                self.new_game();
            }
            Err(e) => {
                self.message_log.push(LogEntry {
                    text: format!("Load failed: {}", e),
                    color: LogColor::Warning,
                });
                self.new_game();
            }
        }
    }

    pub fn start_combat(&mut self) {
        let stage_config = stage::generate_stage(&mut self.rng, self.stage_number);
        self.biome = stage_config.biome;

        let board = Board::new(&mut self.rng);

        let enemy_count = stage_config.enemies.len();
        let first_enemy_name = stage_config.enemies[0].name.clone();

        self.message_log.push(LogEntry {
            text: format!(
                "--- Stage {} | {} ---",
                self.stage_number,
                self.biome.name()
            ),
            color: LogColor::Info,
        });

        if enemy_count > 1 {
            self.message_log.push(LogEntry {
                text: format!("{} enemies approach! First up: {}!", enemy_count, first_enemy_name),
                color: LogColor::Warning,
            });
        } else {
            self.message_log.push(LogEntry {
                text: format!("A wild {} appears!", first_enemy_name),
                color: LogColor::Warning,
            });
        }

        if stage_config.enemies[0].is_boss {
            self.message_log.push(LogEntry {
                text: "BOSS BATTLE!".to_string(),
                color: LogColor::Crit,
            });
        }

        self.combat = Some(CombatState {
            board,
            enemies: stage_config.enemies,
            current_enemy_idx: 0,
            turn_count: 0,
            cursor: (0, 0),
            phase: CombatPhase::PlayerInput,
            lifesteal_active: false,
        });

        self.screen = Screen::Combat;
    }

    fn enter_map(&mut self) {
        // Heal a small amount between stages
        let stage_heal = 10 + self.stage_number as i32;
        self.player.heal(stage_heal);

        self.combat = None;
        self.screen = Screen::Map;
    }

    fn enter_shop(&mut self) {
        let items = shop::generate_shop_items(self.stage_number, &self.player);
        self.shop_state = Some(ShopScreenState {
            items,
            selected_index: 0,
            message: None,
        });
        self.screen = Screen::Shop;
    }
}
