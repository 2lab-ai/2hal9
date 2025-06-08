# 2HAL9 Export Guide

## 🎬 Overview

The 2HAL9 MVP supports exporting demo recordings to various visual formats for sharing, documentation, and presentations.

## 📊 Export Formats

### 1. Animated SVG
- **Format**: Self-contained SVG with CSS animations
- **Features**: 
  - Animated signal flow between neurons
  - Timed status messages
  - Browser-viewable
  - No external dependencies
- **Use Case**: Documentation, web embedding

### 2. Frame Sequence
- **Format**: Series of SVG files (one per frame)
- **Features**:
  - Customizable frame rate (FPS)
  - Suitable for GIF/video conversion
  - Frame-by-frame visualization
- **Use Case**: Creating GIFs, video editing

### 3. GIF Conversion Script
- **Format**: Bash script for automated conversion
- **Features**:
  - Uses ImageMagick or FFmpeg
  - Configurable output settings
  - Batch processing support
- **Use Case**: Quick GIF generation

## 🚀 Quick Start

### Export Everything at Once
```bash
./mvp/export-demo.sh
# Select recording
# Choose option 4 (All formats)
```

### Individual Export Commands

#### Animated SVG
```bash
cargo run -p hal9_mvp -- --export-svg=mvp/recordings/demo_task_management_20250108_123456.json
# Creates: demo_task_management_20250108_123456.svg
```

#### Frame Sequence
```bash
cargo run -p hal9_mvp -- --export-frames=mvp/recordings/demo_task_management_20250108_123456.json
# Creates: demo_task_management_20250108_123456_frames/
#   - frame_00000.svg
#   - frame_00001.svg
#   - ...
```

#### GIF Script
```bash
cargo run -p hal9_mvp -- --export-gif-script=mvp/recordings/demo_task_management_20250108_123456.json
# Creates: demo_task_management_20250108_123456.gif.sh
```

## 🎨 Creating GIFs

### Option 1: Using ImageMagick

```bash
# Install ImageMagick if not present
brew install imagemagick  # macOS
apt-get install imagemagick  # Ubuntu

# Convert frames to GIF
convert -delay 3 -loop 0 frames/*.svg demo.gif

# With optimization
convert -delay 3 -loop 0 -layers optimize frames/*.svg demo_optimized.gif
```

### Option 2: Using FFmpeg

```bash
# Install FFmpeg if not present
brew install ffmpeg  # macOS
apt-get install ffmpeg  # Ubuntu

# First convert SVG to PNG
for svg in frames/*.svg; do
    convert "$svg" "${svg%.svg}.png"
done

# Create GIF
ffmpeg -framerate 30 -pattern_type glob -i 'frames/*.png' \
       -vf "scale=800:600:flags=lanczos" \
       demo.gif
```

### Option 3: Using the Generated Script

```bash
# Run the auto-generated script
bash demo_task_management_20250108_123456.gif.sh
```

## 📐 SVG Animation Details

The exported SVG includes:
- **Neuron visualization**: L4 (Strategic), L3 (Design), L2 (Implementation)
- **Signal animations**: Timed paths showing message flow
- **Status updates**: Synchronized text messages
- **Duration**: Matches original recording timing

### SVG Structure
```xml
<svg width="800" height="600">
  <!-- Background -->
  <rect fill="#1a1a1a"/>
  
  <!-- Neurons with pulsing animation -->
  <circle cx="400" cy="150" r="40" fill="#6366f1">
    <animate attributeName="r" values="40;45;40" dur="2s"/>
  </circle>
  
  <!-- Signal flow animations -->
  <line stroke="#4ade80" opacity="0">
    <animate attributeName="opacity" values="0;1;1;0" begin="1.5s"/>
  </line>
</svg>
```

## 🔧 Customization

### Frame Rate Settings
- **Low (10 FPS)**: Smaller file size, choppy animation
- **Medium (30 FPS)**: Good balance, smooth animation
- **High (60 FPS)**: Very smooth, larger files

### Output Optimization
```bash
# Reduce GIF size
gifsicle -O3 --colors 128 input.gif > output.gif

# Convert to WebP (smaller, better quality)
gif2webp -q 80 input.gif -o output.webp
```

## 📝 Tips & Tricks

1. **Best Recording Length**: 5-15 seconds for GIFs
2. **Optimal Resolution**: 800x600 (can be scaled)
3. **Color Reduction**: Use 128-256 colors for smaller files
4. **Loop Settings**: Use `-loop 0` for infinite loop

## 🎯 Use Cases

### Documentation
```markdown
![2HAL9 Demo](demo.gif)
*Hierarchical AI orchestration in action*
```

### Presentations
- Embed SVG directly in slides
- Use GIF for universal compatibility
- Frame sequence for step-by-step walkthrough

### Social Media
- Twitter: GIF under 15MB
- GitHub: Embed in README
- Blog posts: SVG for interactivity

## 🐛 Troubleshooting

### SVG Not Animating
- Ensure browser supports SVG animations
- Try opening in Chrome/Firefox
- Check Content-Security-Policy settings

### GIF Too Large
```bash
# Reduce dimensions
convert -resize 50% input.gif output.gif

# Reduce colors
convert +dither -colors 64 input.gif output.gif

# Optimize with gifsicle
gifsicle -O3 --lossy=80 input.gif > output.gif
```

### Missing Dependencies
```bash
# Check installations
which convert  # ImageMagick
which ffmpeg   # FFmpeg
which gifsicle # Gifsicle

# Install all at once (macOS)
brew install imagemagick ffmpeg gifsicle
```

## 🚀 Advanced Usage

### Batch Export
```bash
# Export all recordings
for recording in mvp/recordings/*.json; do
    cargo run -p hal9_mvp -- --export-svg="$recording"
done
```

### Custom Frame Rate
```bash
# 60 FPS for ultra-smooth animation
echo "60" | cargo run -p hal9_mvp -- --export-frames=recording.json
```

### Watermarking
```bash
# Add watermark to GIF
convert demo.gif -gravity southeast \
        -stroke '#000C' -strokewidth 2 -annotate +10+10 '2HAL9' \
        -stroke none -fill white -annotate +10+10 '2HAL9' \
        watermarked.gif
```

## 📚 Further Reading

- [SVG Animation Guide](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Animation)
- [ImageMagick Documentation](https://imagemagick.org/script/command-line-options.php)
- [FFmpeg GIF Guide](https://engineering.giphy.com/how-to-make-gifs-with-ffmpeg/)
- [Web Animation Best Practices](https://web.dev/animations/)

---

**Happy Exporting! 🎬**