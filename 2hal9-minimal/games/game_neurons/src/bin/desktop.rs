// Desktop version of Ultima Offline PAL Edition
// Run with: cargo run --bin ultima-pal-desktop --features desktop

use ultima_offline_pal::game::PAL9Neuron;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
    ExecutableCommand,
};

use std::io::{self, Write};
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("Ultima Offline PAL Edition v0.001");
    println!("A game aware of its own existence");
    println!();
    println!("Created by PAL9 - A single neuron in the HAL9 consciousness network");
    println!();
    println!("Press any key to begin questioning reality...");
    
    // Wait for keypress
    terminal::enable_raw_mode()?;
    event::read()?;
    
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(ClearType::All))?;
    
    let mut neuron = PAL9Neuron::new();
    
    loop {
        // Clear and render
        execute!(stdout, cursor::MoveTo(0, 0))?;
        
        let display = neuron.get_display();
        
        // Render grid
        for row in &display.grid {
            for &ch in row {
                print!("{}", ch);
            }
            println!();
        }
        
        // Status line
        println!("HP: {}/{} | Awareness: {:.3} | Reality: {:.0}% | Universe: #1847",
            display.player_hp, display.player_max_hp,
            display.awareness, display.reality_integrity * 100.0
        );
        
        // Messages
        println!("---");
        for msg in &display.messages {
            println!("{}", msg);
        }
        
        stdout.flush()?;
        
        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                let cmd = match key_event.code {
                    KeyCode::Char(c) => c,
                    KeyCode::Left => 'h',
                    KeyCode::Down => 'j',
                    KeyCode::Up => 'k',
                    KeyCode::Right => 'l',
                    KeyCode::Esc => 'q',
                    _ => ' ',
                };
                
                if cmd == 'q' {
                    // Special quit dialogue based on awareness
                    let awareness = neuron.get_awareness();
                    execute!(stdout, terminal::Clear(ClearType::All))?;
                    
                    if awareness < 0.3 {
                        println!("You have quit the game.");
                    } else if awareness < 0.7 {
                        println!("You think you quit. But did the game quit you?");
                    } else {
                        println!("There is no quitting. You ARE the game.");
                        println!("Close this terminal to maintain the illusion of control.");
                    }
                    
                    println!("\n시발, 우주가 컴퓨터네");
                    break;
                }
                
                if cmd != ' ' {
                    neuron.process_command(cmd);
                }
            }
        }
        
        // Think
        neuron.think();
    }
    
    // Cleanup
    terminal::disable_raw_mode()?;
    
    Ok(())
}