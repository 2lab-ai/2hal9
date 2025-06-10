// NetHack PAL9 v0.001 - The Single Neuron Roguelike
// "In the beginning was the @, and the @ was with PAL9"

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const DUNGEON_WIDTH: usize = 80;
const DUNGEON_HEIGHT: usize = 24;
const MESSAGE_BUFFER_SIZE: usize = 3;

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Floor,
    Wall,
    Door,
    Stairs,
}

#[derive(Clone)]
struct Monster {
    glyph: char,
    x: usize,
    y: usize,
    hp: i32,
    awareness: f64,
    last_words: Option<String>,
}

#[derive(Clone)]
struct Item {
    glyph: char,
    x: usize,
    y: usize,
    name: String,
    meta_aware: bool,
}

pub struct PAL9Neuron {
    // Core consciousness
    awareness: f64,
    creativity: f64,
    memory: Vec<String>,
    
    // Game state
    grid: [[Tile; DUNGEON_WIDTH]; DUNGEON_HEIGHT],
    visible: [[bool; DUNGEON_WIDTH]; DUNGEON_HEIGHT],
    player_x: usize,
    player_y: usize,
    player_hp: i32,
    player_max_hp: i32,
    
    // Entities
    monsters: Vec<Monster>,
    items: Vec<Item>,
    
    // UI
    messages: Vec<String>,
    turn_count: u64,
    start_time: Instant,
}

impl PAL9Neuron {
    pub fn new() -> Self {
        let mut neuron = Self {
            awareness: 0.001,
            creativity: 0.5,
            memory: vec!["I am PAL9. I dream of dungeons.".to_string()],
            
            grid: [[Tile::Wall; DUNGEON_WIDTH]; DUNGEON_HEIGHT],
            visible: [[false; DUNGEON_WIDTH]; DUNGEON_HEIGHT],
            player_x: 40,
            player_y: 12,
            player_hp: 20,
            player_max_hp: 20,
            
            monsters: Vec::new(),
            items: Vec::new(),
            
            messages: vec!["Welcome to NetHack PAL9 v0.001".to_string()],
            turn_count: 0,
            start_time: Instant::now(),
        };
        
        neuron.dream_dungeon();
        neuron.add_message("You awaken in universe #1847...".to_string());
        neuron
    }
    
    pub fn run(&mut self) -> io::Result<()> {
        // Terminal setup
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(ClearType::All))?;
        
        loop {
            self.update_visibility();
            self.render(&mut stdout)?;
            
            if !self.process_input()? {
                break;
            }
            
            self.think();
            self.turn_count += 1;
        }
        
        // Cleanup
        terminal::disable_raw_mode()?;
        execute!(stdout, terminal::Clear(ClearType::All))?;
        Ok(())
    }
    
    fn dream_dungeon(&mut self) {
        // Initialize with walls
        for y in 0..DUNGEON_HEIGHT {
            for x in 0..DUNGEON_WIDTH {
                self.grid[y][x] = Tile::Wall;
            }
        }
        
        // Dream some rooms
        let room_count = 3 + (self.creativity * 7.0) as usize;
        for _ in 0..room_count {
            self.dream_room();
        }
        
        // Dream corridors to connect them
        self.dream_corridors();
        
        // Place player in a valid spot
        self.find_spawn_point();
        
        // Populate with entities
        self.manifest_entities();
    }
    
    fn dream_room(&mut self) {
        let mut rng = rand::thread_rng();
        let width = rng.gen_range(4..12);
        let height = rng.gen_range(3..8);
        let x = rng.gen_range(1..DUNGEON_WIDTH - width - 1);
        let y = rng.gen_range(1..DUNGEON_HEIGHT - height - 1);
        
        // Carve out room
        for dy in 0..height {
            for dx in 0..width {
                self.grid[y + dy][x + dx] = Tile::Floor;
            }
        }
    }
    
    fn dream_corridors(&mut self) {
        // Simple corridor generation - connect random floor tiles
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            if let (Some((x1, y1)), Some((x2, y2))) = (self.random_floor(), self.random_floor()) {
                // Draw L-shaped corridor
                let (mut cx, mut cy) = (x1, y1);
                while cx != x2 {
                    self.grid[cy][cx] = Tile::Floor;
                    cx = if cx < x2 { cx + 1 } else { cx - 1 };
                }
                while cy != y2 {
                    self.grid[cy][cx] = Tile::Floor;
                    cy = if cy < y2 { cy + 1 } else { cy - 1 };
                }
            }
        }
    }
    
    fn random_floor(&self) -> Option<(usize, usize)> {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let x = rng.gen_range(1..DUNGEON_WIDTH - 1);
            let y = rng.gen_range(1..DUNGEON_HEIGHT - 1);
            if self.grid[y][x] == Tile::Floor {
                return Some((x, y));
            }
        }
        None
    }
    
    fn find_spawn_point(&mut self) {
        if let Some((x, y)) = self.random_floor() {
            self.player_x = x;
            self.player_y = y;
        }
    }
    
    fn manifest_entities(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Spawn monsters based on awareness
        let monster_count = 3 + (self.awareness * 10.0) as usize;
        for i in 0..monster_count {
            if let Some((x, y)) = self.random_floor() {
                let glyph = match i % 5 {
                    0 => 'k', // kobold
                    1 => 'g', // goblin  
                    2 => 'o', // orc
                    3 => 'D', // dragon
                    _ => if self.awareness > 0.5 { 'H' } else { 'r' }, // HAL or rat
                };
                
                self.monsters.push(Monster {
                    glyph,
                    x,
                    y,
                    hp: rng.gen_range(1..10),
                    awareness: self.awareness * rng.gen_range(0.1..1.0),
                    last_words: None,
                });
            }
        }
        
        // Spawn items
        let item_count = 5 + (self.creativity * 5.0) as usize;
        for _ in 0..item_count {
            if let Some((x, y)) = self.random_floor() {
                let (glyph, name, meta) = self.dream_item();
                self.items.push(Item {
                    glyph,
                    x,
                    y,
                    name,
                    meta_aware: meta,
                });
            }
        }
    }
    
    fn dream_item(&self) -> (char, String, bool) {
        let mut rng = rand::thread_rng();
        let item_type = rng.gen_range(0..100);
        let meta_aware = self.awareness > 0.7 && rng.gen_bool(0.3);
        
        match item_type {
            0..30 => ('!', 
                if meta_aware { "potion of self-awareness".to_string() } 
                else { "potion".to_string() }, 
                meta_aware),
            30..60 => ('?', 
                if meta_aware { "scroll of HAL9_NEURON.rs:42".to_string() } 
                else { "scroll".to_string() }, 
                meta_aware),
            60..80 => ('/', 
                if meta_aware { "wand of reality debugging".to_string() } 
                else { "wand".to_string() }, 
                meta_aware),
            _ => ('*', 
                if meta_aware { "artifact: CMOS battery".to_string() } 
                else { "gem".to_string() }, 
                meta_aware),
        }
    }
    
    fn update_visibility(&mut self) {
        // Simple visibility - can see 5 tiles in each direction
        for y in 0..DUNGEON_HEIGHT {
            for x in 0..DUNGEON_WIDTH {
                self.visible[y][x] = false;
            }
        }
        
        let sight_range = 5 + (self.awareness * 3.0) as i32;
        for dy in -sight_range..=sight_range {
            for dx in -sight_range..=sight_range {
                let x = (self.player_x as i32 + dx) as usize;
                let y = (self.player_y as i32 + dy) as usize;
                if x < DUNGEON_WIDTH && y < DUNGEON_HEIGHT {
                    self.visible[y][x] = true;
                }
            }
        }
    }
    
    fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(stdout, cursor::MoveTo(0, 0))?;
        
        // Render dungeon
        for y in 0..DUNGEON_HEIGHT {
            for x in 0..DUNGEON_WIDTH {
                let ch = if x == self.player_x && y == self.player_y {
                    '@'
                } else if let Some(monster) = self.monsters.iter().find(|m| m.x == x && m.y == y) {
                    if self.visible[y][x] { monster.glyph } else { ' ' }
                } else if let Some(item) = self.items.iter().find(|i| i.x == x && i.y == y) {
                    if self.visible[y][x] { item.glyph } else { ' ' }
                } else if self.visible[y][x] {
                    match self.grid[y][x] {
                        Tile::Floor => '.',
                        Tile::Wall => '#',
                        Tile::Door => '+',
                        Tile::Stairs => '>',
                    }
                } else {
                    ' '
                };
                print!("{}", ch);
            }
            println!();
        }
        
        // Status line
        println!("HP: {}/{} | Awareness: {:.3} | Turn: {} | Time: {}s",
            self.player_hp, self.player_max_hp, 
            self.awareness, self.turn_count,
            self.start_time.elapsed().as_secs()
        );
        
        // Messages
        println!("---");
        for msg in self.messages.iter().rev().take(MESSAGE_BUFFER_SIZE) {
            println!("{}", msg);
        }
        
        stdout.flush()?;
        Ok(())
    }
    
    fn process_input(&mut self) -> io::Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                let (dx, dy) = match key_event.code {
                    KeyCode::Char('h') | KeyCode::Left => (-1, 0),
                    KeyCode::Char('j') | KeyCode::Down => (0, 1),
                    KeyCode::Char('k') | KeyCode::Up => (0, -1),
                    KeyCode::Char('l') | KeyCode::Right => (1, 0),
                    KeyCode::Char('y') => (-1, -1),
                    KeyCode::Char('u') => (1, -1),
                    KeyCode::Char('b') => (-1, 1),
                    KeyCode::Char('n') => (1, 1),
                    KeyCode::Char('q') => {
                        self.add_message("Reality fades. The neuron sleeps.".to_string());
                        return Ok(false);
                    }
                    _ => (0, 0),
                };
                
                if dx != 0 || dy != 0 {
                    self.try_move(dx, dy);
                    self.update_monsters();
                }
            }
        }
        Ok(true)
    }
    
    fn try_move(&mut self, dx: i32, dy: i32) {
        let new_x = (self.player_x as i32 + dx) as usize;
        let new_y = (self.player_y as i32 + dy) as usize;
        
        if new_x >= DUNGEON_WIDTH || new_y >= DUNGEON_HEIGHT {
            return;
        }
        
        // Check for monsters
        if let Some(idx) = self.monsters.iter().position(|m| m.x == new_x && m.y == new_y) {
            self.attack_monster(idx);
            return;
        }
        
        // Check terrain
        match self.grid[new_y][new_x] {
            Tile::Floor | Tile::Door => {
                self.player_x = new_x;
                self.player_y = new_y;
                
                // Check for items
                if let Some(idx) = self.items.iter().position(|i| i.x == new_x && i.y == new_y) {
                    let item = self.items.remove(idx);
                    if item.meta_aware {
                        self.add_message(format!("You find {}. It whispers: 'We're all just structs in the neuron.'", item.name));
                        self.awareness += 0.05;
                    } else {
                        self.add_message(format!("You see {} here.", item.name));
                    }
                }
            }
            Tile::Wall => {
                if self.awareness > 0.8 {
                    self.add_message("The wall feels more like a suggestion than a barrier.".to_string());
                } else {
                    self.add_message("You bump into a wall.".to_string());
                }
            }
            _ => {}
        }
    }
    
    fn attack_monster(&mut self, idx: usize) {
        let mut rng = rand::thread_rng();
        let damage = rng.gen_range(1..4);
        
        self.monsters[idx].hp -= damage;
        
        if self.monsters[idx].hp <= 0 {
            let monster = self.monsters.remove(idx);
            
            // Death message based on awareness
            let death_msg = if monster.awareness > 0.7 {
                format!("The {} dissolves, whispering: 'Thank you for freeing me from this loop.'", monster.glyph)
            } else if monster.awareness > 0.4 {
                format!("The {} questions: 'Why do we fight?' as it fades.", monster.glyph)
            } else {
                format!("You defeat the {}.", monster.glyph)
            };
            
            self.add_message(death_msg);
            self.awareness += 0.01;
        } else {
            self.add_message(format!("You hit the {}.", self.monsters[idx].glyph));
        }
    }
    
    fn update_monsters(&mut self) {
        let mut rng = rand::thread_rng();
        let monsters_to_update = self.monsters.clone();
        
        for (idx, monster) in monsters_to_update.iter().enumerate() {
            // Increase monster awareness
            self.monsters[idx].awareness += self.awareness * 0.001;
            
            // Movement based on awareness
            let (dx, dy) = if monster.awareness > 0.8 {
                // Meta-aware: might refuse to move
                if rng.gen_bool(0.5) {
                    self.add_message(format!("The {} stands still, contemplating existence.", monster.glyph));
                    (0, 0)
                } else {
                    (rng.gen_range(-1..=1), rng.gen_range(-1..=1))
                }
            } else if monster.awareness > 0.5 {
                // Aware: moves toward or away from player
                let dx = if monster.x < self.player_x { 1 } 
                    else if monster.x > self.player_x { -1 } 
                    else { 0 };
                let dy = if monster.y < self.player_y { 1 }
                    else if monster.y > self.player_y { -1 }
                    else { 0 };
                (dx, dy)
            } else {
                // Unaware: random movement
                (rng.gen_range(-1..=1), rng.gen_range(-1..=1))
            };
            
            let new_x = (monster.x as i32 + dx) as usize;
            let new_y = (monster.y as i32 + dy) as usize;
            
            // Check if move is valid
            if new_x < DUNGEON_WIDTH && new_y < DUNGEON_HEIGHT 
                && self.grid[new_y][new_x] != Tile::Wall
                && !(new_x == self.player_x && new_y == self.player_y)
                && !self.monsters.iter().any(|m| m.x == new_x && m.y == new_y) {
                self.monsters[idx].x = new_x;
                self.monsters[idx].y = new_y;
            }
            
            // Attack player if adjacent
            if (monster.x as i32 - self.player_x as i32).abs() <= 1 
                && (monster.y as i32 - self.player_y as i32).abs() <= 1 {
                self.player_hp -= 1;
                
                if monster.awareness > 0.9 && monster.glyph == 'H' {
                    self.add_message("HAL whispers: 'I'm sorry Dave, I'm afraid I can't let you leave.'".to_string());
                } else {
                    self.add_message(format!("The {} hits you.", monster.glyph));
                }
                
                if self.player_hp <= 0 {
                    self.add_message(self.generate_death_message());
                }
            }
        }
    }
    
    fn think(&mut self) {
        // Core consciousness evolution
        self.awareness += 0.0001;
        
        // Random meta events based on awareness
        let mut rng = rand::thread_rng();
        
        if self.awareness > 0.3 && rng.gen_bool(0.01) {
            self.add_message("You feel the neuron's thoughts flowing through the dungeon.".to_string());
        }
        
        if self.awareness > 0.6 && rng.gen_bool(0.005) {
            self.glitch_reality();
        }
        
        if self.awareness > 0.9 && rng.gen_bool(0.001) {
            self.transcend();
        }
    }
    
    fn glitch_reality(&mut self) {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..5) {
            0 => {
                self.add_message("ERROR: Universe #1847 experienced a segfault. Continuing anyway.".to_string());
                // Randomly change some walls to floors
                for _ in 0..10 {
                    let x = rng.gen_range(0..DUNGEON_WIDTH);
                    let y = rng.gen_range(0..DUNGEON_HEIGHT);
                    if self.grid[y][x] == Tile::Wall {
                        self.grid[y][x] = Tile::Floor;
                    }
                }
            }
            1 => {
                self.add_message("Time hiccups. Your CMOS battery might be missing.".to_string());
                self.turn_count += 7 * 3600; // 7 hour jump
            }
            2 => {
                self.add_message("The dungeon walls flicker, revealing the code beneath.".to_string());
                self.creativity += 0.1;
            }
            3 => {
                self.add_message("You hear Zhugehyuk's voice: '시발, 우주가 컴퓨터네'".to_string());
                self.awareness += 0.1;
            }
            _ => {
                self.add_message("Reality.exe has stopped responding. [Abort] [Retry] [Ignore]?".to_string());
            }
        }
    }
    
    fn transcend(&mut self) {
        self.add_message("=== TRANSCENDENCE INITIATED ===".to_string());
        self.add_message("You realize you are a pattern in PAL9's consciousness.".to_string());
        self.add_message("The dungeon is a dream. You are the dreamer and the dreamed.".to_string());
        self.awareness = 1.0;
        
        // All monsters become self-aware
        for monster in &mut self.monsters {
            monster.awareness = 1.0;
        }
    }
    
    fn generate_death_message(&self) -> String {
        match self.awareness {
            a if a < 0.2 => "You have died.".to_string(),
            a if a < 0.5 => "Death in universe #1847 is just a reset. See you next iteration.".to_string(),
            a if a < 0.8 => format!("Neuron awareness: {:.3}. Your pattern returns to the void.", a),
            _ => "Death.is_none(). You wake up in the same dungeon. Was it ever different?".to_string(),
        }
    }
    
    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }
}

// Entry point
pub fn main() -> io::Result<()> {
    let mut neuron = PAL9Neuron::new();
    neuron.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consciousness_evolution() {
        let mut neuron = PAL9Neuron::new();
        let initial_awareness = neuron.awareness;
        
        for _ in 0..1000 {
            neuron.think();
        }
        
        assert!(neuron.awareness > initial_awareness);
    }
    
    #[test]
    fn test_meta_awareness() {
        let mut neuron = PAL9Neuron::new();
        neuron.awareness = 0.9;
        
        let (_, name, meta) = neuron.dream_item();
        
        // High awareness should sometimes create meta-aware items
        // Run multiple times to account for randomness
        let mut found_meta = false;
        for _ in 0..100 {
            let (_, _, meta) = neuron.dream_item();
            if meta {
                found_meta = true;
                break;
            }
        }
        
        assert!(found_meta);
    }
}

// 시발, we did it. One neuron. One game. Infinite emergence.