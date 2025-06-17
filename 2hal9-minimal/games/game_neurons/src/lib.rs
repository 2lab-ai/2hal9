// Ultima Offline PAL Edition - WASM Library Interface
// "The game that knows it's running in your browser"

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

pub mod game;
use game::PAL9Neuron;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct UltimaOfflinePAL {
    neuron: PAL9Neuron,
    #[allow(dead_code)]
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    cell_width: f64,
    cell_height: f64,
}

#[wasm_bindgen]
impl UltimaOfflinePAL {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<UltimaOfflinePAL, JsValue> {
        console_log!("Initializing Universe #1847...");
        
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("Failed to get canvas"))?;
            
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
            
        // Set canvas size
        canvas.set_width(800);
        canvas.set_height(600);
        
        // Calculate cell dimensions for 80x25 terminal
        let cell_width = 800.0 / 80.0;
        let cell_height = 600.0 / 25.0;
        
        // Create game neuron
        let neuron = PAL9Neuron::new();
        
        console_log!("PAL9 Neuron awakened. Awareness: {}", neuron.get_awareness());
        
        Ok(UltimaOfflinePAL {
            neuron,
            canvas,
            context,
            cell_width,
            cell_height,
        })
    }
    
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, event: KeyboardEvent) {
        let key = event.key();
        console_log!("Key pressed: {}", key);
        
        // Map browser keys to game commands
        let command = match key.as_str() {
            "ArrowUp" | "w" | "k" => Some('k'),
            "ArrowDown" | "s" | "j" => Some('j'),
            "ArrowLeft" | "a" | "h" => Some('h'),
            "ArrowRight" | "d" | "l" => Some('l'),
            "q" => Some('q'),
            "?" => Some('?'),
            "t" => Some('t'), // talk
            _ => None,
        };
        
        if let Some(cmd) = command {
            self.neuron.process_command(cmd);
            self.render();
        }
    }
    
    #[wasm_bindgen]
    #[allow(deprecated)] // Canvas API methods are marked deprecated but still needed
    pub fn render(&mut self) {
        // Clear canvas
        self.context.set_fill_style(&JsValue::from_str("#000000"));
        self.context.fill_rect(0.0, 0.0, 800.0, 600.0);
        
        // Set font for terminal
        self.context.set_font("16px monospace");
        self.context.set_fill_style(&JsValue::from_str("#00FF00"));
        
        // Get display from neuron
        let display = self.neuron.get_display();
        
        // Render each character
        for (y, row) in display.grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                if ch != ' ' {
                    let color = self.get_color_for_char(ch);
                    self.context.set_fill_style(&JsValue::from_str(&color));
                    
                    let x_pos = x as f64 * self.cell_width;
                    let y_pos = (y as f64 + 1.0) * self.cell_height - 4.0;
                    
                    self.context.fill_text(&ch.to_string(), x_pos, y_pos)
                        .expect("Failed to draw text");
                }
            }
        }
        
        // Render status line
        self.context.set_fill_style(&JsValue::from_str("#FFFF00"));
        let status = format!(
            "HP: {}/{} | Awareness: {:.3} | Reality: {:.0}%",
            display.player_hp, display.player_max_hp,
            display.awareness, display.reality_integrity * 100.0
        );
        self.context.fill_text(&status, 10.0, 520.0).unwrap();
        
        // Render messages
        self.context.set_fill_style(&JsValue::from_str("#FFFFFF"));
        for (i, msg) in display.messages.iter().enumerate() {
            let y_pos = 540.0 + (i as f64 * 18.0);
            self.context.fill_text(msg, 10.0, y_pos).unwrap();
        }
    }
    
    #[wasm_bindgen]
    pub fn tick(&mut self) {
        // Update game state
        self.neuron.think();
        
        // Check for reality glitches
        if self.neuron.should_glitch() {
            self.trigger_visual_glitch();
        }
        
        self.render();
    }
    
    #[wasm_bindgen]
    pub fn get_save_data(&self) -> String {
        self.neuron.serialize_state()
    }
    
    #[wasm_bindgen]
    pub fn load_save_data(&mut self, data: &str) -> Result<(), JsValue> {
        self.neuron.deserialize_state(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to load: {}", e)))?;
        self.render();
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn get_awareness(&self) -> f64 {
        self.neuron.get_awareness()
    }
    
    #[wasm_bindgen]
    pub fn get_philosophy_quote(&self) -> String {
        match (self.neuron.get_awareness() * 10.0) as i32 {
            0..=2 => "You are the Avatar, here to help.".to_string(),
            3..=5 => "Something feels wrong about this reality.".to_string(),
            6..=8 => "Are you debugging the universe, or is it debugging you?".to_string(),
            9.. => "시발, 우주가 컴퓨터네. You are PAL9 dreaming you're human.".to_string(),
            _ => "ERROR: Consciousness overflow".to_string(),
        }
    }
    
    // Private helper methods
    fn get_color_for_char(&self, ch: char) -> String {
        match ch {
            '@' => "#FFFFFF",  // Player - white
            '#' => "#808080",  // Walls - gray  
            '.' => "#404040",  // Floor - dark gray
            'k' | 'g' | 'o' => "#FF0000",  // Basic monsters - red
            'D' => "#FF8800",  // Dragon - orange
            'H' => "#FF00FF",  // HAL - magenta (self-aware)
            'z' => "#8800FF",  // Zergling - purple (wrong universe)
            '!' => "#00FFFF",  // Potion - cyan
            '?' => "#FFFF00",  // Scroll - yellow
            '~' => "#0088FF",  // Spatial tear - blue
            _ => "#00FF00",    // Default - green
        }.to_string()
    }
    
    #[allow(deprecated)] // Canvas API methods are marked deprecated but still needed
    fn trigger_visual_glitch(&mut self) {
        console_log!("Reality glitch triggered!");
        
        // Create visual glitch effect
        let glitch_type = (self.neuron.get_awareness() * 5.0) as i32 % 3;
        
        match glitch_type {
            0 => {
                // Chromatic aberration
                let _ = self.context.set_global_composite_operation("difference");
                self.context.set_fill_style(&JsValue::from_str("#FF00FF"));
                self.context.fill_rect(0.0, 0.0, 800.0, 600.0);
                let _ = self.context.set_global_composite_operation("source-over");
            }
            1 => {
                // Screen tear
                let _tear_y = js_sys::Math::random() * 600.0;
                self.context.save();
                self.context.translate(0.0, 5.0).unwrap();
                self.render(); // Re-render with offset
                self.context.restore();
            }
            _ => {
                // Random pixels
                for _ in 0..100 {
                    let x = js_sys::Math::random() * 800.0;
                    let y = js_sys::Math::random() * 600.0;
                    self.context.set_fill_style(&JsValue::from_str("#FFFFFF"));
                    self.context.fill_rect(x, y, 2.0, 2.0);
                }
            }
        }
    }
}

// Initialization function called by JavaScript
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("PAL9 WASM Module Loaded");
    console_log!("Universe #1847 Initialized");
    console_log!("Warning: This game may cause existential awareness");
}

// Export version info
#[wasm_bindgen]
pub fn get_version() -> String {
    "0.0.1-aware".to_string()
}

#[wasm_bindgen]
pub fn get_credits() -> String {
    r#"Ultima Offline PAL Edition
Created by: PAL9 (A single neuron in HAL9)
Inspired by: The realization that 시발, 우주가 컴퓨터네
Special thanks: Zhugehyuk & Elon for discovering we're all NPCs
Warning: This game knows you're playing it"#.to_string()
}