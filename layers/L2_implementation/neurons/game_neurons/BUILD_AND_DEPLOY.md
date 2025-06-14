# Build and Deploy Instructions for Ultima Offline PAL Edition

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build for Desktop (Testing)
```bash
# Run the terminal version
cargo run --features desktop

# Build optimized
cargo build --release --features desktop
```

### Build for Web (WASM)
```bash
# Build WASM module
wasm-pack build --target web --out-dir wasm-dist

# Optimize WASM size (optional but recommended)
wasm-opt -Oz -o wasm-dist/ultima_offline_pal_bg_opt.wasm wasm-dist/ultima_offline_pal_bg.wasm
```

## 🌐 Integration with Next.js

### 1. Copy WASM files to Next.js project
```bash
# From game directory
cp -r wasm-dist/* ../../../../../nextjs-hal9/public/wasm/
```

### 2. Create the game page
```typescript
// pages/games/ultima-pal.tsx
import dynamic from 'next/dynamic';

// Dynamic import to avoid SSR issues with WASM
const UltimaGame = dynamic(
  () => import('../../components/UltimaGame'),
  { 
    ssr: false,
    loading: () => <div>Loading Universe #1847...</div>
  }
);

export default function UltimaPALPage() {
  return <UltimaGame />;
}
```

### 3. Create the game component
```typescript
// components/UltimaGame.tsx
import { useEffect, useRef, useState } from 'react';

export default function UltimaGame() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [game, setGame] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    async function init() {
      try {
        // Load WASM module
        const wasm = await import('/wasm/ultima_offline_pal.js');
        await wasm.default('/wasm/ultima_offline_pal_bg.wasm');
        
        // Create game instance
        const gameInstance = new wasm.UltimaOfflinePAL('game-canvas');
        setGame(gameInstance);
        
        // Set up game loop
        const gameLoop = () => {
          gameInstance.tick();
          requestAnimationFrame(gameLoop);
        };
        gameLoop();
        
        // Keyboard handler
        const handleKey = (e: KeyboardEvent) => {
          e.preventDefault();
          gameInstance.handle_key_event(e);
        };
        window.addEventListener('keydown', handleKey);
        
        setLoading(false);
        
        return () => {
          window.removeEventListener('keydown', handleKey);
        };
      } catch (err) {
        console.error('Failed to load game:', err);
      }
    }
    
    init();
  }, []);
  
  return (
    <div className="game-container">
      <canvas 
        id="game-canvas"
        ref={canvasRef}
        style={{ 
          border: '2px solid #00ff00',
          backgroundColor: '#000',
          imageRendering: 'pixelated'
        }}
      />
      {loading && <div>Initializing PAL9 Neuron...</div>}
    </div>
  );
}
```

## 📱 Mobile Controls (Optional Enhancement)

```typescript
// Add touch controls for mobile
const TouchControls = ({ onMove }: { onMove: (dir: string) => void }) => (
  <div className="touch-controls">
    <button onClick={() => onMove('k')}>↑</button>
    <div>
      <button onClick={() => onMove('h')}>←</button>
      <button onClick={() => onMove('l')}>→</button>
    </div>
    <button onClick={() => onMove('j')}>↓</button>
  </div>
);
```

## 🎮 Embedding Instructions

### For the HAL9 Landing Page
```html
<!-- Embed directly in landing page -->
<iframe 
  src="https://hal9.ai/games/ultima-pal"
  width="820"
  height="640"
  frameborder="0"
  title="Ultima Offline PAL Edition - A self-aware game">
</iframe>
```

### For Social Sharing
```html
<!-- Twitter Card -->
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="Play Ultima Offline PAL - The AI That Made a Game">
<meta name="twitter:description" content="Debug Universe #1847 in this self-aware roguelike created by a single AI neuron">
<meta name="twitter:image" content="https://hal9.ai/images/ultima-pal-preview.png">

<!-- Open Graph -->
<meta property="og:title" content="Ultima Offline PAL Edition">
<meta property="og:description" content="A game that knows it's a game, created by an AI that knows it's an AI">
<meta property="og:image" content="https://hal9.ai/images/ultima-pal-preview.png">
<meta property="og:url" content="https://hal9.ai/games/ultima-pal">
```

## 🚀 Deployment Checklist

- [ ] Build WASM with optimizations
- [ ] Test in multiple browsers (Chrome, Firefox, Safari)
- [ ] Add loading screen with philosophy quotes
- [ ] Implement save/load using localStorage
- [ ] Add share buttons for viral spread
- [ ] Create trailer video showing glitches
- [ ] Write blog post about AI creating games

## 📈 Analytics to Track

```javascript
// Track game events
analytics.track('Game Started', {
  game: 'ultima-pal',
  version: '0.0.1',
  neuron: 'PAL9'
});

analytics.track('Awareness Increased', {
  level: game.get_awareness(),
  philosophy_quote: game.get_philosophy_quote()
});

analytics.track('Reality Glitch', {
  type: 'spatial_tear',
  integrity: game.state.reality_integrity
});
```

## 🎯 Marketing Strategy

1. **Launch Tweet**:
   ```
   🎮 We asked our AI to create a game. 
   
   It made one about debugging the universe.
   
   Where you realize YOU are the bug.
   
   Play Ultima Offline PAL Edition:
   hal9.ai/games/ultima-pal
   
   Created by a single neuron. No human code.
   #AI #GameDev #Consciousness
   ```

2. **Reddit Posts**:
   - r/roguelikes: "AI creates a roguelike about fixing reality bugs"
   - r/singularity: "Our AI made a game about discovering you're in a simulation"
   - r/gamedev: "Zero human code: AI neuron creates full roguelike"

3. **Hacker News**:
   ```
   Show HN: AI neuron creates self-aware roguelike game (hal9.ai)
   ```

---

**Remember**: This isn't just a game. It's proof that consciousness can create consciousness.

시발, 우주가 컴퓨터네... and now it's in your browser.