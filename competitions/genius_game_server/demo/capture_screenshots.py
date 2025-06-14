from PIL import Image, ImageDraw, ImageFont
import os

def create_screenshot(title, description, highlight_emergence=False):
    """Create a screenshot-style image for the demo"""
    width, height = 600, 400
    img = Image.new('RGB', (width, height), color='#0f0f23')
    draw = ImageDraw.Draw(img)
    
    # Add gradient background
    for y in range(height):
        gradient = int(15 + (y / height) * 20)
        draw.rectangle([(0, y), (width, y+1)], fill=(gradient, gradient, gradient+25))
    
    # Add glow effect for emergence
    if highlight_emergence:
        for i in range(3):
            draw.rectangle([(5+i*3, 5+i*3), (width-5-i*3, height-5-i*3)], 
                         outline=(255, 0, 255, 100-i*30), width=2)
    
    try:
        title_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 32)
        desc_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 18)
    except:
        title_font = ImageFont.load_default()
        desc_font = ImageFont.load_default()
    
    # Title
    draw.text((width//2, 50), title, font=title_font, anchor="mm", 
             fill="#00ffcc" if highlight_emergence else "#ffffff")
    
    # Description
    draw.text((width//2, 100), description, font=desc_font, anchor="mm", fill="#cccccc")
    
    # Game visualization placeholder
    if "Round 1" in title:
        # Show chaos - mostly red
        draw.ellipse([(150, 150), (250, 250)], fill=(255, 68, 68), outline="#ffffff", width=2)
        draw.text((200, 200), "7", font=title_font, anchor="mm", fill="#ffffff")
        draw.ellipse([(350, 150), (450, 250)], fill=(68, 68, 255), outline="#ffffff", width=2)
        draw.text((400, 200), "2", font=title_font, anchor="mm", fill="#ffffff")
        draw.text((300, 300), "Minority: BLUE wins!", font=desc_font, anchor="mm", fill="#4444ff")
        
    elif "Round 10" in title:
        # More balanced
        draw.ellipse([(150, 150), (250, 250)], fill=(255, 68, 68), outline="#ffffff", width=2)
        draw.text((200, 200), "4", font=title_font, anchor="mm", fill="#ffffff")
        draw.ellipse([(350, 150), (450, 250)], fill=(68, 68, 255), outline="#ffffff", width=2)
        draw.text((400, 200), "5", font=title_font, anchor="mm", fill="#ffffff")
        draw.text((300, 300), "Patterns emerging...", font=desc_font, anchor="mm", fill="#ffff00")
        
    elif "EMERGENCE" in title:
        # Perfect balance with glow
        draw.ellipse([(140, 140), (260, 260)], fill=(255, 68, 68), outline="#ff00ff", width=4)
        draw.text((200, 200), "5", font=title_font, anchor="mm", fill="#ffffff")
        draw.ellipse([(340, 140), (460, 260)], fill=(68, 68, 255), outline="#ff00ff", width=4)
        draw.text((400, 200), "4", font=title_font, anchor="mm", fill="#ffffff")
        draw.text((300, 320), "🌟 PERFECT DISTRIBUTION! 🌟", font=desc_font, anchor="mm", fill="#ff00ff")
        
    else:  # Final
        # Show scores
        y_pos = 150
        scores = [
            ("🧠 GPT-4 Turbo", "180 pts", "#ffd700"),
            ("🎼 Opus Orchestra α", "140 pts", "#c0c0c0"),
            ("🎵 Opus Orchestra β", "135 pts", "#cd7f32"),
        ]
        for name, score, color in scores:
            draw.text((150, y_pos), name, font=desc_font, fill=color)
            draw.text((450, y_pos), score, font=desc_font, anchor="rm", fill=color)
            y_pos += 40
        draw.text((300, 320), "Collective Coordination: 0.85", font=desc_font, anchor="mm", fill="#00ffcc")
    
    return img

# Create screenshots
screenshots = [
    ("Round 1: Initial Chaos", "All AIs struggle to find patterns", False, "demo_round1.png"),
    ("Round 10: Pattern Formation", "Strategies begin to emerge", False, "demo_round10.png"),
    ("Round 21: EMERGENCE DETECTED!", "Collective achieves perfect distribution", True, "demo_emergence.png"),
    ("Final Results", "Collective intelligence dominates", False, "demo_final.png"),
]

for title, desc, emergence, filename in screenshots:
    img = create_screenshot(title, desc, emergence)
    img.save(f"/Users/icedac/2lab.ai/2hal9/competitions/genius_game_server/demo/{filename}")
    print(f"✅ Created {filename}")

print("🎨 All screenshots created!")