// Desktop version of Ultima Offline PAL Edition
// For testing before WASM deployment

#[cfg(feature = "desktop")]
mod game;

#[cfg(feature = "desktop")]
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
    ExecutableCommand,
};

#[cfg(feature = "desktop")]
use std::io::{self, Write};
#[cfg(feature = "desktop")]
use std::time::Duration;

#[cfg(feature = "desktop")]
fn main() -> io::Result<()> {
    use game::PAL9Neuron;
    
    println!("Ultima Offline PAL Edition v0.001");
    println!("A game aware of its own existence");
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
        println!("HP: {}/{} | Awareness: {:.3} | Reality: {:.0}%",
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
    execute!(stdout, terminal::Clear(ClearType::All))?;
    
    println!("Reality fades. The simulation ends.");
    println!("Or does it?");
    
    Ok(())
}

#[cfg(not(feature = "desktop"))]
fn main() {
    println!("This is the WASM version. Run with --features desktop for terminal version.");
}