import subprocess
import time
import os
from PIL import Image, ImageDraw, ImageFont
import numpy as np
from datetime import datetime
import json

def create_demo_frames():
    """Create synthetic demo frames that look like the actual game"""
    frames = []
    width, height = 1200, 800
    
    # Game state for simulation
    players = [
        {'id': 'collective_opus_1', 'emoji': '🎼', 'name': 'Opus Orchestra α', 'type': 'Collective AI', 'score': 0},
        {'id': 'collective_opus_2', 'emoji': '🎵', 'name': 'Opus Orchestra β', 'type': 'Collective AI', 'score': 0},
        {'id': 'collective_opus_3', 'emoji': '🎶', 'name': 'Opus Orchestra γ', 'type': 'Collective AI', 'score': 0},
        {'id': 'collective_swarm_1', 'emoji': '🐝', 'name': 'Swarm Unit 001', 'type': 'Swarm Intelligence', 'score': 0},
        {'id': 'collective_swarm_2', 'emoji': '🐛', 'name': 'Swarm Unit 002', 'type': 'Swarm Intelligence', 'score': 0},
        {'id': 'collective_swarm_3', 'emoji': '🦋', 'name': 'Swarm Unit 003', 'type': 'Swarm Intelligence', 'score': 0},
        {'id': 'sota_claude', 'emoji': '🤖', 'name': 'Claude Opus 4', 'type': 'SOTA Model', 'score': 0},
        {'id': 'sota_gpt4', 'emoji': '🧠', 'name': 'GPT-4 Turbo', 'type': 'SOTA Model', 'score': 0},
        {'id': 'sota_gemini', 'emoji': '💫', 'name': 'Gemini 2.0', 'type': 'SOTA Model', 'score': 0}
    ]
    
    # Key rounds to showcase
    key_rounds = [
        {
            'round': 1,
            'title': 'Initial Chaos',
            'red_count': 7,
            'blue_count': 2,
            'minority': 'blue',
            'emergence': False,
            'special_event': None
        },
        {
            'round': 10,
            'title': 'Pattern Formation',
            'red_count': 4,
            'blue_count': 5,
            'minority': 'red',
            'emergence': False,
            'special_event': 'AI strategies evolving...'
        },
        {
            'round': 21,
            'title': 'EMERGENCE DETECTED! 🌟',
            'red_count': 5,
            'blue_count': 4,
            'minority': 'blue',
            'emergence': True,
            'special_event': 'Collective intelligence achieved perfect distribution!'
        },
        {
            'round': 30,
            'title': 'Final Results',
            'red_count': 4,
            'blue_count': 5,
            'minority': 'red',
            'emergence': True,
            'special_event': 'Game Complete!'
        }
    ]
    
    # Create frames for each key round
    for round_data in key_rounds:
        # Create main frame
        img = Image.new('RGB', (width, height), color='#0f0f23')
        draw = ImageDraw.Draw(img)
        
        # Try to use a nice font, fallback to default
        try:
            title_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 48)
            header_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 32)
            normal_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 20)
            small_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 16)
        except:
            title_font = ImageFont.load_default()
            header_font = ImageFont.load_default()
            normal_font = ImageFont.load_default()
            small_font = ImageFont.load_default()
        
        # Draw gradient background effect
        for y in range(height):
            gradient = int(15 + (y / height) * 15)
            draw.rectangle([(0, y), (width, y+1)], fill=(gradient, gradient, gradient+20))
        
        # Title
        title_text = "🧠 AI Genius Game - Minority Game Competition 🎮"
        draw.text((width//2, 40), title_text, font=title_font, anchor="mm", fill="#00ffcc")
        
        # Round indicator
        round_text = f"Round {round_data['round']} - {round_data['title']}"
        draw.text((width//2, 100), round_text, font=header_font, anchor="mm", fill="#ffffff")
        
        # Draw emergence glow effect if detected
        if round_data['emergence']:
            # Create glow effect
            for i in range(5):
                alpha = 50 - i*10
                glow_color = (255, 0, 255, alpha)
                draw.rectangle([(10+i*2, 10+i*2), (width-10-i*2, height-10-i*2)], 
                             outline=glow_color, width=2)
        
        # Draw players panel (left side)
        panel_x = 30
        panel_y = 150
        panel_width = 350
        panel_height = 500
        
        # Panel background
        draw.rectangle([(panel_x, panel_y), (panel_x + panel_width, panel_y + panel_height)], 
                      fill=(20, 20, 40), outline=(0, 255, 204), width=2)
        
        draw.text((panel_x + panel_width//2, panel_y + 20), "AI Players", 
                 font=header_font, anchor="mm", fill="#00ffcc")
        
        # Draw each player
        player_y = panel_y + 60
        for i, player in enumerate(players):
            # Simulate scores
            if round_data['round'] > 1:
                player['score'] += np.random.randint(-5, 15)
            
            # Player background (highlight if choosing)
            if i < round_data['red_count']:
                player_bg = (100, 0, 0)  # Red choice
            else:
                player_bg = (0, 0, 100)  # Blue choice
            
            draw.rectangle([(panel_x + 10, player_y), 
                          (panel_x + panel_width - 10, player_y + 40)], 
                          fill=player_bg, outline=(100, 100, 100))
            
            # Player info
            player_text = f"{player['emoji']} {player['name']}"
            draw.text((panel_x + 20, player_y + 10), player_text, 
                     font=normal_font, fill="#ffffff")
            
            score_text = f"{player['score']} pts"
            draw.text((panel_x + panel_width - 30, player_y + 10), score_text, 
                     font=normal_font, anchor="rm", fill="#00ffcc")
            
            player_y += 50
        
        # Draw game visualization (center)
        center_x = width // 2
        center_y = height // 2 + 50
        
        # Red circle
        red_radius = 80 + (10 if round_data['minority'] == 'red' else 0)
        draw.ellipse([(center_x - 150 - red_radius, center_y - red_radius),
                     (center_x - 150 + red_radius, center_y + red_radius)],
                    fill=(255, 68, 68), outline=(255, 255, 255), width=3)
        draw.text((center_x - 150, center_y), str(round_data['red_count']), 
                 font=title_font, anchor="mm", fill="#ffffff")
        draw.text((center_x - 150, center_y + red_radius + 20), "RED", 
                 font=normal_font, anchor="mm", fill="#ff4444")
        
        # Blue circle
        blue_radius = 80 + (10 if round_data['minority'] == 'blue' else 0)
        draw.ellipse([(center_x + 150 - blue_radius, center_y - blue_radius),
                     (center_x + 150 + blue_radius, center_y + blue_radius)],
                    fill=(68, 68, 255), outline=(255, 255, 255), width=3)
        draw.text((center_x + 150, center_y), str(round_data['blue_count']), 
                 font=title_font, anchor="mm", fill="#ffffff")
        draw.text((center_x + 150, center_y + blue_radius + 20), "BLUE", 
                 font=normal_font, anchor="mm", fill="#4444ff")
        
        # VS text
        draw.text((center_x, center_y), "VS", font=header_font, anchor="mm", fill="#ffffff")
        
        # Minority indicator
        minority_text = f"Minority: {'🔴 RED' if round_data['minority'] == 'red' else '🔵 BLUE'}"
        draw.text((center_x, center_y + 150), minority_text, 
                 font=header_font, anchor="mm", fill="#ffff00")
        
        # Stats panel (right side)
        stats_x = width - 380
        stats_y = 150
        stats_width = 350
        stats_height = 300
        
        draw.rectangle([(stats_x, stats_y), (stats_x + stats_width, stats_y + stats_height)], 
                      fill=(20, 20, 40), outline=(0, 255, 204), width=2)
        
        draw.text((stats_x + stats_width//2, stats_y + 20), "Game Analytics", 
                 font=header_font, anchor="mm", fill="#00ffcc")
        
        # Stats
        stats = [
            ("Emergence Events", "3" if round_data['round'] > 20 else "0"),
            ("Coordination Score", "0.85" if round_data['emergence'] else "0.42"),
            ("Leading AI", "🧠 GPT-4" if round_data['round'] < 20 else "🎼 Opus α"),
            ("Strategy Depth", f"{round_data['round'] / 30:.2f}")
        ]
        
        stat_y = stats_y + 60
        for label, value in stats:
            draw.text((stats_x + 20, stat_y), label, font=small_font, fill="#888888")
            draw.text((stats_x + stats_width - 20, stat_y), value, 
                     font=normal_font, anchor="rm", fill="#00ffcc")
            stat_y += 50
        
        # Special event notification
        if round_data['special_event']:
            event_y = height - 100
            # Event background
            draw.rectangle([(50, event_y - 30), (width - 50, event_y + 30)], 
                          fill=(80, 0, 120), outline=(255, 0, 255), width=3)
            draw.text((width//2, event_y), round_data['special_event'], 
                     font=header_font, anchor="mm", fill="#ffffff")
        
        # Add frame multiple times for duration
        frame_count = 30 if round_data['emergence'] else 20
        for _ in range(frame_count):
            frames.append(img.copy())
        
        # Add transition frames
        if round_data != key_rounds[-1]:
            for i in range(10):
                transition = img.copy()
                draw_transition = ImageDraw.Draw(transition)
                overlay = Image.new('RGBA', (width, height), (0, 0, 0, int(255 * (i / 10))))
                transition.paste(overlay, (0, 0), overlay)
                frames.append(transition)
    
    return frames

def create_demo_gif():
    print("🎬 Creating AI Genius Game demo GIF...")
    
    # Create frames
    frames = create_demo_frames()
    
    # Save as GIF
    output_path = '/Users/icedac/2lab.ai/2hal9/competitions/genius_game_server/demo/ai_genius_demo.gif'
    frames[0].save(
        output_path,
        save_all=True,
        append_images=frames[1:],
        duration=100,  # 100ms per frame
        loop=0,
        optimize=True
    )
    
    print(f"✅ Demo GIF created: {output_path}")
    print(f"📊 Total frames: {len(frames)}")
    print(f"⏱️  Duration: ~{len(frames) * 0.1:.1f} seconds")

if __name__ == "__main__":
    create_demo_gif()