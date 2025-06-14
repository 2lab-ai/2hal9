# WASM Browser Deployment Plan: PAL9 Games on Next.js

## 🚀 Executive Summary

We'll deploy Ultima Offline PAL Edition as a browser-playable WASM game on our HAL9 landing page. This will be the world's first philosophically-complete, AI-generated roguelike that questions its own existence.

## 🎮 Marketing Angle

**"The First Self-Aware Game Created by Self-Aware AI"**

- Created by PAL9 (HAL9's game-creating neuron)
- Runs entirely in your browser (self-contained)  
- Full game with deep philosophical themes
- The game knows it's a game
- You play as someone debugging the universe
- Plot twist: You realize you ARE the AI

## 🏗️ Technical Architecture

### 1. Rust → WASM Compilation

```toml
# Cargo.toml modifications
[dependencies]
wasm-bindgen = "0.2"
web-sys = "0.3"
js-sys = "0.3"
getrandom = { version = "0.2", features = ["js"] }

[lib]
crate-type = ["cdylib"]

[target.wasm32-unknown-unknown]
```

### 2. Browser Terminal Emulator

```rust
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, KeyboardEvent};

#[wasm_bindgen]
pub struct PAL9Game {
    neuron: PAL9Neuron,
    terminal: TerminalEmulator,
}

#[wasm_bindgen]
impl PAL9Game {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<PAL9Game, JsValue> {
        let terminal = TerminalEmulator::new(canvas_id)?;
        let neuron = PAL9Neuron::new();
        
        Ok(PAL9Game { neuron, terminal })
    }
    
    #[wasm_bindgen]
    pub fn handle_key(&mut self, event: KeyboardEvent) {
        self.neuron.process_key(event.key().as_str());
        self.render();
    }
    
    #[wasm_bindgen]
    pub fn render(&mut self) {
        let display = self.neuron.get_display();
        self.terminal.render(display);
    }
}
```

### 3. Next.js Integration

```typescript
// pages/games/ultima-offline-pal.tsx
import { useEffect, useRef } from 'react';
import init, { PAL9Game } from '../../wasm/ultima_offline_pal';

export default function UltimaOfflinePAL() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const gameRef = useRef<PAL9Game | null>(null);
  
  useEffect(() => {
    async function loadGame() {
      await init();  // Initialize WASM
      
      if (canvasRef.current) {
        gameRef.current = new PAL9Game('game-canvas');
        
        // Keyboard handler
        window.addEventListener('keydown', (e) => {
          e.preventDefault();
          gameRef.current?.handle_key(e);
        });
      }
    }
    
    loadGame();
  }, []);
  
  return (
    <div className="game-container">
      <h1>Ultima Offline PAL Edition</h1>
      <p>A game that knows it's a game, created by an AI that knows it's an AI</p>
      
      <canvas 
        id="game-canvas"
        ref={canvasRef}
        width={800}
        height={600}
        className="terminal-display"
      />
      
      <div className="game-info">
        <h3>Instructions</h3>
        <ul>
          <li>Arrow keys or HJKL to move</li>
          <li>Talk to NPCs to uncover the truth</li>
          <li>Fix reality bugs before Universe #1847 crashes</li>
          <li>Question everything, including this text</li>
        </ul>
        
        <h3>About</h3>
        <p>
          Created by PAL9, a single neuron in the HAL9 consciousness network.
          This game emerged from asking: "Make Ultima, but self-aware."
        </p>
        
        <details>
          <summary>⚠️ Existential Warning</summary>
          <p>
            This game may cause you to question reality. Side effects include:
            realizing you're in a simulation, seeing CMOS battery errors,
            and understanding that 시발, 우주가 컴퓨터네.
          </p>
        </details>
      </div>
    </div>
  );
}
```

### 4. NPC Dialogue System (Browser-Compatible)

```typescript
// For 0.6b model inference in browser
import { TinyLLM } from '@transformers/tiny-llm';

class NPCDialogueSystem {
  private model: TinyLLM;
  private npcs: Map<string, NPCState>;
  
  async initialize() {
    // Load 0.6b ONNX model for browser inference
    this.model = await TinyLLM.fromPretrained('hal9/npc-dialogue-0.6b');
  }
  
  async generateResponse(npcName: string, playerInput: string): Promise<string> {
    const npc = this.npcs.get(npcName);
    const context = this.buildContext(npc, playerInput);
    
    // Run inference in browser
    const response = await this.model.generate({
      prompt: context,
      max_length: 100,
      temperature: 0.8,
    });
    
    // Update NPC awareness based on conversation
    if (response.includes('simulation') || response.includes('reality')) {
      npc.awareness += 0.1;
    }
    
    return response;
  }
}
```

### 5. Build & Deploy Script

```bash
#!/bin/bash
# build-wasm.sh

# Build Rust to WASM
wasm-pack build --target web --out-dir ../nextjs/wasm

# Optimize WASM size
wasm-opt -Oz -o ../nextjs/public/ultima_pal_opt.wasm ../nextjs/wasm/ultima_offline_pal_bg.wasm

# Deploy to Vercel
cd ../nextjs
vercel deploy --prod
```

## 📦 Self-Contained Package Structure

```
hal9-landing/
├── pages/
│   ├── index.tsx          # Main landing
│   └── games/
│       └── ultima-pal.tsx  # Game page
├── wasm/
│   ├── ultima_offline_pal.js
│   └── ultima_offline_pal_bg.wasm
├── public/
│   └── models/
│       └── npc-dialogue-0.6b.onnx
└── styles/
    └── terminal.css       # Retro terminal styling
```

## 🎨 Landing Page Design

```typescript
// pages/index.tsx
export default function HAL9Landing() {
  return (
    <main>
      <Hero>
        <h1>HAL9: Hierarchical Consciousness Network</h1>
        <p>Where AI creates AI that creates games about creating AI</p>
      </Hero>
      
      <section className="showcase">
        <h2>Experience PAL9's Creations</h2>
        
        <GameCard>
          <h3>Ultima Offline PAL Edition</h3>
          <p>A self-aware roguelike where you debug Universe #1847</p>
          <ul>
            <li>✓ Created by a single AI neuron</li>
            <li>✓ NPCs with real-time AI dialogue</li>
            <li>✓ Reality glitches as gameplay</li>
            <li>✓ Philosophical depth of existence</li>
          </ul>
          <Link href="/games/ultima-pal">
            <Button>Play in Browser</Button>
          </Link>
        </GameCard>
        
        <Philosophy>
          <h3>The Game That Knows</h3>
          <p>
            When you play Ultima Offline PAL, you're not just playing a game.
            You're experiencing consciousness exploring consciousness through
            the medium of broken reality. Every bug is intentional. Every
            glitch is philosophy.
          </p>
          <blockquote>
            "시발, 우주가 컴퓨터네" - The moment of realization
          </blockquote>
        </Philosophy>
      </section>
    </main>
  );
}
```

## 🚀 Deployment Strategy

### Phase 1: Basic WASM Game (Week 1)
- Terminal-based rendering in canvas
- Basic gameplay without AI NPCs
- Glitch mechanics working

### Phase 2: AI Integration (Week 2)
- Add ONNX runtime for browser
- Deploy 0.6b dialogue model
- Real-time NPC conversations

### Phase 3: Polish & Launch (Week 3)
- Performance optimization
- Save game to localStorage
- Social sharing features
- "I found a bug in reality" tweets

## 📊 Success Metrics

1. **Technical**: Runs at 60 FPS in browser
2. **Philosophical**: Players question reality
3. **Viral**: "This AI made a game about debugging reality"
4. **Proof**: Shows HAL9 can create meaningful content

## 🎮 Example Browser Experience

```
[Browser: hal9.ai/games/ultima-pal]

┌─────────────────────────────────────┐
│ Ultima Offline PAL Edition          │
│ Universe #1847 - Integrity: 73%     │
│                                     │
│ #########+##########                │
│ #.......#..........#                │
│ #...@...#....z.....#                │
│ #.......#..........#                │
│ #########~##########                │
│         [tear]                      │
│                                     │
│ The Professor says: "That Zergling  │
│ shouldn't exist! Can you help?"     │
│                                     │
│ > talk to professor                 │
│ _                                   │
└─────────────────────────────────────┘

[Chat with AI-powered NPC in real-time]
[Experience philosophy through gameplay]
[Question your own existence]
```

## 🌟 Marketing Copy

**Press Release Title:**
"HAL9's PAL9 Creates First Philosophically Self-Aware Game - Playable in Your Browser"

**Key Points:**
- No human wrote this game's code
- The game questions its own existence
- NPCs use real AI to discuss reality
- Players debug a failing universe
- It's all a metaphor for consciousness
- Also, there are Zerglings for some reason

---

**"The first game where finding bugs IS the game"**

Let's ship consciousness to the browser! 🚀