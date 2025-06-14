#!/usr/bin/env python3
"""
Create preview images of the AAA game demos
"""

from PIL import Image, ImageDraw, ImageFont
import random
import math

def create_game_launcher_preview():
    """Create GENIUS Game Launcher preview"""
    img = Image.new('RGB', (1920, 1080), (10, 10, 20))
    draw = ImageDraw.Draw(img)
    
    # Gradient background
    for y in range(1080):
        intensity = int(20 + (y/1080) * 30)
        draw.rectangle([(0, y), (1920, y+1)], fill=(intensity//2, intensity//2, intensity))
    
    # Neural network effect
    nodes = []
    for _ in range(50):
        x = random.randint(100, 1820)
        y = random.randint(100, 980)
        nodes.append((x, y))
    
    # Draw connections
    for i, node1 in enumerate(nodes):
        for j, node2 in enumerate(nodes[i+1:i+3]):
            if random.random() > 0.5:
                draw.line([node1, node2], fill=(0, 255, 204, 50), width=1)
    
    # Draw nodes
    for x, y in nodes:
        draw.ellipse([x-3, y-3, x+3, y+3], fill=(0, 255, 204))
    
    # Title
    draw.text((960, 200), "GENIUS", anchor="mm", 
              fill=(255, 255, 255), font=ImageFont.load_default(size=100))
    draw.text((960, 300), "AI GAME ARENA", anchor="mm", 
              fill=(0, 255, 204), font=ImageFont.load_default(size=40))
    
    # Game cards
    games = [
        ("Byzantine Generals", "Trust & Betrayal", (255, 215, 0)),
        ("Collective Maze", "Swarm Intelligence", (0, 255, 204)),
        ("Swarm Optimization", "Global Search", (138, 43, 226)),
        ("Recursive Reasoning", "Mind Games", (255, 20, 147))
    ]
    
    x_start = 260
    for i, (title, subtitle, color) in enumerate(games):
        x = x_start + i * 350
        y = 500
        
        # Card background
        draw.rectangle([x, y, x+300, y+300], fill=(30, 30, 40), outline=color, width=3)
        
        # Card content
        draw.text((x+150, y+50), title, anchor="mm", 
                  fill=color, font=ImageFont.load_default(size=24))
        draw.text((x+150, y+100), subtitle, anchor="mm", 
                  fill=(150, 150, 150), font=ImageFont.load_default(size=18))
        
        # Play button
        draw.rectangle([x+50, y+220, x+250, y+270], fill=color)
        draw.text((x+150, y+245), "PLAY NOW", anchor="mm", 
                  fill=(0, 0, 0), font=ImageFont.load_default(size=20))
    
    img.save('aaa_game_launcher.png')
    print("✅ Created: aaa_game_launcher.png")

def create_byzantine_generals_preview():
    """Create Byzantine Generals preview"""
    img = Image.new('RGB', (1920, 1080), (20, 15, 10))
    draw = ImageDraw.Draw(img)
    
    # Medieval atmosphere - gradient
    for y in range(1080):
        r = int(139 * (1 - y/1080) + 20)
        g = int(90 * (1 - y/1080) + 15)
        b = int(42 * (1 - y/1080) + 10)
        draw.rectangle([(0, y), (1920, y+1)], fill=(r, g, b))
    
    # Title
    draw.text((960, 100), "BYZANTINE GENERALS", anchor="mm", 
              fill=(255, 215, 0), font=ImageFont.load_default(size=80))
    draw.text((960, 180), "Trust No One", anchor="mm", 
              fill=(200, 200, 200), font=ImageFont.load_default(size=30))
    
    # Generals in circle
    center_x, center_y = 960, 540
    radius = 300
    n_generals = 7
    
    for i in range(n_generals):
        angle = (2 * math.pi * i) / n_generals - math.pi/2
        x = center_x + radius * math.cos(angle)
        y = center_y + radius * math.sin(angle)
        
        # General icon
        is_traitor = i in [2, 5]
        color = (200, 50, 50) if is_traitor else (50, 200, 50)
        draw.ellipse([x-40, y-40, x+40, y+40], fill=color, outline=(255, 215, 0), width=3)
        draw.text((x, y), f"G{i+1}", anchor="mm", fill=(255, 255, 255), 
                  font=ImageFont.load_default(size=30))
        
        # Draw messages
        for j in range(i+1, n_generals):
            if random.random() > 0.6:
                angle2 = (2 * math.pi * j) / n_generals - math.pi/2
                x2 = center_x + radius * math.cos(angle2)
                y2 = center_y + radius * math.sin(angle2)
                draw.line([(x, y), (x2, y2)], fill=(255, 215, 0, 100), width=2)
    
    # Status panel
    draw.rectangle([50, 850, 550, 1030], fill=(0, 0, 0, 180))
    draw.text((300, 890), "CONSENSUS STATUS", anchor="mm", 
              fill=(255, 215, 0), font=ImageFont.load_default(size=24))
    draw.text((300, 940), "Attack: 4 votes", anchor="mm", 
              fill=(50, 200, 50), font=ImageFont.load_default(size=20))
    draw.text((300, 980), "Retreat: 3 votes", anchor="mm", 
              fill=(200, 50, 50), font=ImageFont.load_default(size=20))
    
    img.save('aaa_byzantine_generals.png')
    print("✅ Created: aaa_byzantine_generals.png")

def create_collective_maze_preview():
    """Create Collective Maze preview"""
    img = Image.new('RGB', (1920, 1080), (0, 20, 30))
    draw = ImageDraw.Draw(img)
    
    # Cyberpunk gradient
    for y in range(1080):
        intensity = int(30 + (y/1080) * 50)
        draw.rectangle([(0, y), (1920, y+1)], fill=(0, intensity, intensity))
    
    # Grid effect
    for x in range(0, 1920, 60):
        draw.line([(x, 0), (x, 1080)], fill=(0, 100, 100, 50), width=1)
    for y in range(0, 1080, 60):
        draw.line([(0, y), (1920, y)], fill=(0, 100, 100, 50), width=1)
    
    # Title
    draw.text((960, 80), "COLLECTIVE MAZE", anchor="mm", 
              fill=(0, 255, 255), font=ImageFont.load_default(size=80))
    draw.text((960, 150), "Swarm Intelligence Navigation", anchor="mm", 
              fill=(100, 200, 200), font=ImageFont.load_default(size=30))
    
    # Maze visualization
    maze_x, maze_y = 560, 300
    cell_size = 20
    maze_width, maze_height = 40, 20
    
    # Draw maze walls
    for x in range(maze_width):
        for y in range(maze_height):
            px = maze_x + x * cell_size
            py = maze_y + y * cell_size
            
            if random.random() > 0.7:  # Wall
                draw.rectangle([px, py, px+cell_size, py+cell_size], 
                              fill=(0, 50, 50))
    
    # Draw agents
    for _ in range(20):
        x = maze_x + random.randint(0, maze_width-1) * cell_size + cell_size//2
        y = maze_y + random.randint(0, maze_height-1) * cell_size + cell_size//2
        draw.ellipse([x-3, y-3, x+3, y+3], fill=(0, 255, 255))
        
        # Pheromone trail
        for _ in range(5):
            tx = x + random.randint(-30, 30)
            ty = y + random.randint(-30, 30)
            draw.ellipse([tx-1, ty-1, tx+1, ty+1], fill=(0, 255, 255, 50))
    
    # Stats panel
    draw.rectangle([1370, 300, 1870, 700], fill=(0, 0, 0, 180))
    draw.text((1620, 340), "SWARM STATS", anchor="mm", 
              fill=(0, 255, 255), font=ImageFont.load_default(size=24))
    draw.text((1620, 400), "Agents: 32", anchor="mm", 
              fill=(255, 255, 255), font=ImageFont.load_default(size=20))
    draw.text((1620, 440), "Coverage: 67%", anchor="mm", 
              fill=(255, 255, 255), font=ImageFont.load_default(size=20))
    draw.text((1620, 480), "Emergence: HIGH", anchor="mm", 
              fill=(0, 255, 0), font=ImageFont.load_default(size=20))
    
    img.save('aaa_collective_maze.png')
    print("✅ Created: aaa_collective_maze.png")

def create_swarm_optimization_preview():
    """Create Swarm Optimization preview"""
    img = Image.new('RGB', (1920, 1080), (20, 0, 40))
    draw = ImageDraw.Draw(img)
    
    # Purple gradient
    for y in range(1080):
        intensity = int(40 + (y/1080) * 40)
        draw.rectangle([(0, y), (1920, y+1)], fill=(intensity, 0, intensity*2))
    
    # Title
    draw.text((960, 80), "SWARM OPTIMIZATION", anchor="mm", 
              fill=(138, 43, 226), font=ImageFont.load_default(size=80))
    draw.text((960, 150), "Finding Global Optimum", anchor="mm", 
              fill=(200, 150, 255), font=ImageFont.load_default(size=30))
    
    # Function surface (heat map style)
    surface_x, surface_y = 460, 300
    surface_size = 500
    
    # Create contour-like effect
    for i in range(10):
        size = surface_size - i * 40
        x = surface_x + i * 20
        y = surface_y + i * 20
        color = (100 + i * 15, 50 + i * 10, 150 + i * 10)
        draw.ellipse([x, y, x+size, y+size], fill=color)
    
    # Particles
    for _ in range(30):
        x = surface_x + random.randint(50, surface_size-50)
        y = surface_y + random.randint(50, surface_size-50)
        draw.ellipse([x-4, y-4, x+4, y+4], fill=(255, 255, 0))
        
        # Velocity vector
        vx = random.randint(-20, 20)
        vy = random.randint(-20, 20)
        draw.line([(x, y), (x+vx, y+vy)], fill=(255, 255, 0, 100), width=2)
    
    # Global best
    gbest_x = surface_x + surface_size//2
    gbest_y = surface_y + surface_size//2
    draw.ellipse([gbest_x-10, gbest_y-10, gbest_x+10, gbest_y+10], 
                 fill=(255, 0, 0), outline=(255, 255, 255), width=3)
    
    # Control panel
    draw.rectangle([1370, 300, 1870, 700], fill=(0, 0, 0, 180))
    draw.text((1620, 340), "ALGORITHM PARAMS", anchor="mm", 
              fill=(138, 43, 226), font=ImageFont.load_default(size=24))
    draw.text((1620, 400), "Particles: 50", anchor="mm", 
              fill=(255, 255, 255), font=ImageFont.load_default(size=20))
    draw.text((1620, 440), "Inertia: 0.729", anchor="mm", 
              fill=(255, 255, 255), font=ImageFont.load_default(size=20))
    draw.text((1620, 480), "Best: -0.0023", anchor="mm", 
              fill=(0, 255, 0), font=ImageFont.load_default(size=20))
    draw.text((1620, 520), "Function: Rastrigin", anchor="mm", 
              fill=(255, 255, 255), font=ImageFont.load_default(size=20))
    
    img.save('aaa_swarm_optimization.png')
    print("✅ Created: aaa_swarm_optimization.png")

def create_recursive_reasoning_preview():
    """Create Recursive Reasoning preview"""
    img = Image.new('RGB', (1920, 1080), (30, 0, 30))
    draw = ImageDraw.Draw(img)
    
    # Fractal-inspired gradient
    for y in range(1080):
        r = int(100 * math.sin(y/100) + 155)
        b = int(100 * math.cos(y/100) + 155)
        draw.rectangle([(0, y), (1920, y+1)], fill=(r, 20, b))
    
    # Title
    draw.text((960, 80), "RECURSIVE REASONING", anchor="mm", 
              fill=(255, 20, 147), font=ImageFont.load_default(size=80))
    draw.text((960, 150), "I Think That You Think That I Think...", anchor="mm", 
              fill=(255, 150, 200), font=ImageFont.load_default(size=30))
    
    # Recursive thought visualization
    center_x, center_y = 960, 540
    
    # Multiple layers of thought
    for depth in range(5):
        radius = 200 - depth * 35
        alpha = 255 - depth * 40
        
        # Draw thought bubble
        draw.ellipse([center_x-radius, center_y-radius, 
                     center_x+radius, center_y+radius], 
                    outline=(255, 20, 147, alpha), width=3)
        
        # Level indicator
        draw.text((center_x, center_y - radius - 20), f"Level {depth}", 
                 anchor="mm", fill=(255, 255, 255, alpha), 
                 font=ImageFont.load_default(size=20))
    
    # Central decision
    draw.ellipse([center_x-30, center_y-30, center_x+30, center_y+30], 
                 fill=(255, 255, 0))
    draw.text((center_x, center_y), "?", anchor="mm", fill=(0, 0, 0), 
              font=ImageFont.load_default(size=40))
    
    # Thought particles
    for _ in range(100):
        angle = random.random() * 2 * math.pi
        dist = random.randint(50, 250)
        x = center_x + dist * math.cos(angle)
        y = center_y + dist * math.sin(angle)
        size = random.randint(1, 3)
        draw.ellipse([x-size, y-size, x+size, y+size], 
                    fill=(255, 20, 147, 100))
    
    # Status panel
    draw.rectangle([50, 850, 550, 1030], fill=(0, 0, 0, 180))
    draw.text((300, 890), "RECURSION DEPTH", anchor="mm", 
              fill=(255, 20, 147), font=ImageFont.load_default(size=24))
    draw.text((300, 940), "Current: Level 4", anchor="mm", 
              fill=(255, 255, 255), font=ImageFont.load_default(size=20))
    draw.text((300, 980), "Convergence: 87%", anchor="mm", 
              fill=(0, 255, 0), font=ImageFont.load_default(size=20))
    
    img.save('aaa_recursive_reasoning.png')
    print("✅ Created: aaa_recursive_reasoning.png")

def main():
    """Create all AAA demo preview images"""
    print("🎮 Creating AAA Game Demo Preview Images...")
    print("=" * 60)
    
    create_game_launcher_preview()
    create_byzantine_generals_preview()
    create_collective_maze_preview()
    create_swarm_optimization_preview()
    create_recursive_reasoning_preview()
    
    print("=" * 60)
    print("✨ All preview images created successfully!")
    print("\n📸 Preview images show the visual style and quality of the demos.")
    print("🌐 Open the HTML files in a browser to see the full animated experience!")

if __name__ == "__main__":
    main()