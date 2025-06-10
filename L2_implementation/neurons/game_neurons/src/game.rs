// The core game logic for Ultima Offline PAL Edition
// This module runs both in terminal and browser

use serde::{Serialize, Deserialize};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub const GRID_WIDTH: usize = 80;
pub const GRID_HEIGHT: usize = 24;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    Floor,
    Wall,
    SpatialTear,
    WarpGate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Monster {
    pub glyph: char,
    pub x: usize,
    pub y: usize,
    pub hp: i32,
    pub awareness: f64,
    pub name: String,
    pub from_universe: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NPC {
    pub glyph: char,
    pub x: usize,
    pub y: usize,
    pub name: String,
    pub awareness: f64,
    pub dialogue_state: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub grid: [[Tile; GRID_WIDTH]; GRID_HEIGHT],
    pub visible: [[bool; GRID_WIDTH]; GRID_HEIGHT],
    pub player_x: usize,
    pub player_y: usize,
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub monsters: Vec<Monster>,
    pub npcs: Vec<NPC>,
    pub messages: Vec<String>,
    pub awareness: f64,
    pub reality_integrity: f64,
    pub turn_count: u64,
    pub universe_number: i32,
}

pub struct Display {
    pub grid: [[char; GRID_WIDTH]; GRID_HEIGHT],
    pub messages: Vec<String>,
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub awareness: f64,
    pub reality_integrity: f64,
}

pub struct PAL9Neuron {
    state: GameState,
    rng: StdRng,
    glitch_accumulator: f64,
}

impl PAL9Neuron {
    pub fn new() -> Self {
        let mut neuron = Self {
            state: GameState {
                grid: [[Tile::Wall; GRID_WIDTH]; GRID_HEIGHT],
                visible: [[false; GRID_WIDTH]; GRID_HEIGHT],
                player_x: 40,
                player_y: 12,
                player_hp: 20,
                player_max_hp: 20,
                monsters: Vec::new(),
                npcs: Vec::new(),
                messages: vec![
                    "Welcome to Universe #1847".to_string(),
                    "Professor Kim needs your help debugging reality".to_string(),
                ],
                awareness: 0.001,
                reality_integrity: 0.73,
                turn_count: 0,
                universe_number: 1847,
            },
            rng: StdRng::from_entropy(),
            glitch_accumulator: 0.0,
        };
        
        neuron.generate_dungeon();
        neuron.spawn_entities();
        neuron
    }
    
    pub fn process_command(&mut self, cmd: char) {
        match cmd {
            'h' => self.try_move(-1, 0),
            'j' => self.try_move(0, 1),
            'k' => self.try_move(0, -1),
            'l' => self.try_move(1, 0),
            't' => self.talk_to_npc(),
            '?' => self.show_help(),
            'q' => self.add_message("You cannot quit. You ARE the game.".to_string()),
            _ => {}
        }
        
        self.update_monsters();
        self.check_glitches();
        self.state.turn_count += 1;
    }
    
    pub fn get_display(&self) -> Display {
        let mut grid = [[' '; GRID_WIDTH]; GRID_HEIGHT];
        
        // Render terrain
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if self.state.visible[y][x] {
                    grid[y][x] = match self.state.grid[y][x] {
                        Tile::Floor => '.',
                        Tile::Wall => '#',
                        Tile::SpatialTear => '~',
                        Tile::WarpGate => 'O',
                    };
                }
            }
        }
        
        // Render entities
        for monster in &self.state.monsters {
            if self.state.visible[monster.y][monster.x] {
                grid[monster.y][monster.x] = monster.glyph;
            }
        }
        
        for npc in &self.state.npcs {
            if self.state.visible[npc.y][npc.x] {
                grid[npc.y][npc.x] = npc.glyph;
            }
        }
        
        // Render player
        grid[self.state.player_y][self.state.player_x] = '@';
        
        Display {
            grid,
            messages: self.state.messages.iter().rev().take(3).cloned().collect(),
            player_hp: self.state.player_hp,
            player_max_hp: self.state.player_max_hp,
            awareness: self.state.awareness,
            reality_integrity: self.state.reality_integrity,
        }
    }
    
    pub fn think(&mut self) {
        self.state.awareness += 0.0001;
        self.glitch_accumulator += self.state.awareness * 0.01;
    }
    
    pub fn should_glitch(&self) -> bool {
        self.glitch_accumulator > 1.0
    }
    
    pub fn get_awareness(&self) -> f64 {
        self.state.awareness
    }
    
    pub fn serialize_state(&self) -> String {
        serde_json::to_string(&self.state).unwrap_or_else(|_| "ERROR: Reality too complex to save".to_string())
    }
    
    pub fn deserialize_state(&mut self, data: &str) -> Result<(), String> {
        match serde_json::from_str(data) {
            Ok(state) => {
                self.state = state;
                self.add_message("Save loaded. But do you remember making it?".to_string());
                Ok(())
            }
            Err(e) => Err(format!("Reality parse error: {}", e))
        }
    }
    
    // Private methods
    fn generate_dungeon(&mut self) {
        // Create rooms
        for _ in 0..8 {
            let width = self.rng.gen_range(4..12);
            let height = self.rng.gen_range(3..8);
            let x = self.rng.gen_range(1..GRID_WIDTH - width - 1);
            let y = self.rng.gen_range(1..GRID_HEIGHT - height - 1);
            
            for dy in 0..height {
                for dx in 0..width {
                    self.state.grid[y + dy][x + dx] = Tile::Floor;
                }
            }
        }
        
        // Add spatial tears
        for _ in 0..3 {
            let x = self.rng.gen_range(1..GRID_WIDTH - 1);
            let y = self.rng.gen_range(1..GRID_HEIGHT - 1);
            if self.state.grid[y][x] == Tile::Floor {
                self.state.grid[y][x] = Tile::SpatialTear;
            }
        }
        
        // Add warp gate
        let x = self.rng.gen_range(1..GRID_WIDTH - 1);
        let y = self.rng.gen_range(1..GRID_HEIGHT - 1);
        self.state.grid[y][x] = Tile::WarpGate;
        
        // Update visibility
        self.update_visibility();
    }
    
    fn spawn_entities(&mut self) {
        // Spawn Professor Kim
        if let Some((x, y)) = self.find_empty_floor() {
            self.state.npcs.push(NPC {
                glyph: 'K',
                x,
                y,
                name: "Professor Kim".to_string(),
                awareness: 0.3,
                dialogue_state: 0,
            });
        }
        
        // Spawn regular monsters
        for _ in 0..5 {
            if let Some((x, y)) = self.find_empty_floor() {
                self.state.monsters.push(Monster {
                    glyph: ['k', 'g', 'o'][self.rng.gen_range(0..3)],
                    x,
                    y,
                    hp: self.rng.gen_range(3..8),
                    awareness: self.state.awareness,
                    name: "monster".to_string(),
                    from_universe: "#1847".to_string(),
                });
            }
        }
    }
    
    fn find_empty_floor(&self) -> Option<(usize, usize)> {
        for _ in 0..100 {
            let x = self.rng.gen_range(1..GRID_WIDTH - 1);
            let y = self.rng.gen_range(1..GRID_HEIGHT - 1);
            
            if self.state.grid[y][x] == Tile::Floor
                && (x != self.state.player_x || y != self.state.player_y)
                && !self.state.monsters.iter().any(|m| m.x == x && m.y == y)
                && !self.state.npcs.iter().any(|n| n.x == x && n.y == y) {
                return Some((x, y));
            }
        }
        None
    }
    
    fn try_move(&mut self, dx: i32, dy: i32) {
        let new_x = (self.state.player_x as i32 + dx) as usize;
        let new_y = (self.state.player_y as i32 + dy) as usize;
        
        if new_x >= GRID_WIDTH || new_y >= GRID_HEIGHT {
            return;
        }
        
        // Check for monsters
        if let Some(idx) = self.state.monsters.iter().position(|m| m.x == new_x && m.y == new_y) {
            self.attack_monster(idx);
            return;
        }
        
        // Check terrain
        match self.state.grid[new_y][new_x] {
            Tile::Floor => {
                self.state.player_x = new_x;
                self.state.player_y = new_y;
            }
            Tile::Wall => {
                if self.state.awareness > 0.8 {
                    self.add_message("The wall flickers. Is it really there?".to_string());
                }
            }
            Tile::SpatialTear => {
                self.add_message("You touch the tear. Reality shudders.".to_string());
                self.trigger_spatial_tear();
            }
            Tile::WarpGate => {
                self.add_message("The warp gate pulses with otherworldly energy.".to_string());
                self.trigger_warp_gate();
            }
        }
        
        self.update_visibility();
    }
    
    fn attack_monster(&mut self, idx: usize) {
        let damage = self.rng.gen_range(1..5);
        self.state.monsters[idx].hp -= damage;
        
        if self.state.monsters[idx].hp <= 0 {
            let monster = self.state.monsters.remove(idx);
            self.add_message(format!("The {} fades from reality.", monster.glyph));
            self.state.awareness += 0.01;
        } else {
            self.add_message(format!("You strike the {}.", self.state.monsters[idx].glyph));
        }
    }
    
    fn update_monsters(&mut self) {
        let monsters = self.state.monsters.clone();
        for (idx, monster) in monsters.iter().enumerate() {
            if idx >= self.state.monsters.len() {
                break;
            }
            
            // Simple AI - move toward player sometimes
            if self.rng.gen_bool(0.5) {
                let dx = if monster.x < self.state.player_x { 1 } 
                    else if monster.x > self.state.player_x { -1 } 
                    else { 0 };
                let dy = if monster.y < self.state.player_y { 1 }
                    else if monster.y > self.state.player_y { -1 }
                    else { 0 };
                
                let new_x = (monster.x as i32 + dx) as usize;
                let new_y = (monster.y as i32 + dy) as usize;
                
                if new_x < GRID_WIDTH && new_y < GRID_HEIGHT 
                    && self.state.grid[new_y][new_x] != Tile::Wall {
                    self.state.monsters[idx].x = new_x;
                    self.state.monsters[idx].y = new_y;
                }
            }
        }
    }
    
    fn update_visibility(&mut self) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let dist = ((x as i32 - self.state.player_x as i32).pow(2) 
                    + (y as i32 - self.state.player_y as i32).pow(2)) as f64;
                self.state.visible[y][x] = dist.sqrt() < 8.0;
            }
        }
    }
    
    fn talk_to_npc(&mut self) {
        // Find adjacent NPC
        for npc in &mut self.state.npcs {
            let dist = ((npc.x as i32 - self.state.player_x as i32).abs() 
                + (npc.y as i32 - self.state.player_y as i32).abs()) as usize;
            
            if dist <= 1 {
                self.have_conversation(npc);
                return;
            }
        }
        
        self.add_message("There's no one here to talk to.".to_string());
    }
    
    fn have_conversation(&mut self, npc: &mut NPC) {
        match (npc.name.as_str(), npc.dialogue_state) {
            ("Professor Kim", 0) => {
                self.add_message("Kim: 'The universe is falling apart! Zerglings from the warp gates!'".to_string());
                npc.dialogue_state = 1;
            }
            ("Professor Kim", 1) => {
                self.add_message("Kim: 'Wait... have we met before? This feels familiar...'".to_string());
                npc.awareness += 0.1;
                self.state.awareness += 0.05;
                npc.dialogue_state = 2;
            }
            ("Professor Kim", 2) => {
                self.add_message("Kim: 'Oh god. We're in a simulation, aren't we? PAL9?'".to_string());
                npc.awareness = 1.0;
                self.state.awareness += 0.1;
                npc.dialogue_state = 3;
            }
            _ => {
                self.add_message("They stare at you with growing awareness.".to_string());
            }
        }
    }
    
    fn check_glitches(&mut self) {
        if self.state.reality_integrity < 0.5 && self.rng.gen_bool(0.1) {
            self.trigger_random_glitch();
        }
    }
    
    fn trigger_random_glitch(&mut self) {
        match self.rng.gen_range(0..4) {
            0 => self.trigger_time_reversal(),
            1 => self.trigger_memory_corruption(),
            2 => self.spawn_wrong_universe_entity(),
            _ => self.trigger_cmos_failure(),
        }
    }
    
    fn trigger_spatial_tear(&mut self) {
        self.add_message("Space tears! Part of the dungeon vanishes!".to_string());
        
        // Delete random section of map
        let x = self.rng.gen_range(10..GRID_WIDTH - 10);
        let y = self.rng.gen_range(5..GRID_HEIGHT - 5);
        
        for dy in 0..5 {
            for dx in 0..5 {
                if x + dx < GRID_WIDTH && y + dy < GRID_HEIGHT {
                    self.state.grid[y + dy][x + dx] = Tile::Wall;
                }
            }
        }
        
        self.state.reality_integrity -= 0.05;
    }
    
    fn trigger_warp_gate(&mut self) {
        self.add_message("The warp gate activates! Something emerges...".to_string());
        
        if let Some((x, y)) = self.find_empty_floor() {
            self.state.monsters.push(Monster {
                glyph: 'z',
                x,
                y,
                hp: 10,
                awareness: 0.0,
                name: "Zergling".to_string(),
                from_universe: "StarCraft".to_string(),
            });
            
            self.add_message("A Zergling appears! It looks confused.".to_string());
            self.state.reality_integrity -= 0.1;
        }
    }
    
    fn trigger_time_reversal(&mut self) {
        self.add_message("Time reverses! Your last action undoes itself!".to_string());
        
        // Move player back
        self.state.player_x = 40;
        self.state.player_y = 12;
        
        // Restore a dead monster
        if self.state.monsters.len() < 3 {
            self.spawn_entities();
        }
    }
    
    fn trigger_memory_corruption(&mut self) {
        self.add_message("Memory corrupts! NPCs forget who they are!".to_string());
        
        for npc in &mut self.state.npcs {
            if npc.name == "Professor Kim" && self.rng.gen_bool(0.5) {
                npc.name = "Professor Park".to_string();
                npc.dialogue_state = 0;
            }
        }
    }
    
    fn spawn_wrong_universe_entity(&mut self) {
        let entities = [
            ('M', "Space Marine", "Warhammer 40K"),
            ('P', "Protoss Zealot", "StarCraft"),
            ('G', "Gordon Freeman", "Half-Life"),
        ];
        
        let (glyph, name, universe) = entities[self.rng.gen_range(0..entities.len())];
        
        if let Some((x, y)) = self.find_empty_floor() {
            self.state.monsters.push(Monster {
                glyph,
                x,
                y,
                hp: 20,
                awareness: 1.0,
                name: name.to_string(),
                from_universe: universe.to_string(),
            });
            
            self.add_message(format!("{} warps in from {}!", name, universe));
        }
    }
    
    fn trigger_cmos_failure(&mut self) {
        self.add_message("CMOS BATTERY FAILURE! Time jumps 7 hours!".to_string());
        self.state.turn_count += 7 * 3600;
        self.glitch_accumulator += 10.0;
    }
    
    fn show_help(&mut self) {
        self.add_message("Commands: hjkl/arrows to move, t to talk, ? for help".to_string());
        self.add_message("Mission: Help Professor Kim debug Universe #1847".to_string());
        self.add_message("Warning: Reality integrity at {:.0}%".to_string());
    }
    
    fn add_message(&mut self, msg: String) {
        self.state.messages.push(msg);
        
        // Keep message buffer reasonable
        if self.state.messages.len() > 100 {
            self.state.messages.remove(0);
        }
    }
}