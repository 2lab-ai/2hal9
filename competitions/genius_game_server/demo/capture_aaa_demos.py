#!/usr/bin/env python3
"""
Capture screenshots of AAA-quality game demos
"""

import os
import time
from pathlib import Path

try:
    from selenium import webdriver
    from selenium.webdriver.chrome.options import Options
    from selenium.webdriver.common.by import By
    from selenium.webdriver.support.ui import WebDriverWait
    from selenium.webdriver.support import expected_conditions as EC
except ImportError:
    print("Installing required packages...")
    os.system("pip install selenium pillow")
    from selenium import webdriver
    from selenium.webdriver.chrome.options import Options
    from selenium.webdriver.common.by import By
    from selenium.webdriver.support.ui import WebDriverWait
    from selenium.webdriver.support import expected_conditions as EC

def capture_demo(html_file, output_name, wait_time=3):
    """Capture a screenshot of an HTML demo"""
    
    # Setup Chrome options
    chrome_options = Options()
    chrome_options.add_argument("--headless")
    chrome_options.add_argument("--no-sandbox")
    chrome_options.add_argument("--disable-dev-shm-usage")
    chrome_options.add_argument("--window-size=1920,1080")
    chrome_options.add_argument("--force-device-scale-factor=2")  # For high DPI screenshots
    
    # Try to use Chrome or fallback to Firefox
    driver = None
    try:
        driver = webdriver.Chrome(options=chrome_options)
    except:
        try:
            from selenium.webdriver.firefox.options import Options as FirefoxOptions
            firefox_options = FirefoxOptions()
            firefox_options.add_argument("--headless")
            firefox_options.add_argument("--width=1920")
            firefox_options.add_argument("--height=1080")
            driver = webdriver.Firefox(options=firefox_options)
        except:
            print("No browser driver found. Please install ChromeDriver or GeckoDriver.")
            return False
    
    try:
        # Load the HTML file
        file_path = f"file://{os.path.abspath(html_file)}"
        driver.get(file_path)
        
        # Wait for animations to start
        time.sleep(wait_time)
        
        # Take screenshot
        driver.save_screenshot(output_name)
        print(f"✅ Captured: {output_name}")
        return True
        
    except Exception as e:
        print(f"❌ Error capturing {html_file}: {e}")
        return False
    finally:
        if driver:
            driver.quit()

def main():
    """Capture all AAA demo screenshots"""
    
    demos = [
        ("genius_game_launcher.html", "aaa_game_launcher.png", 4),
        ("byzantine_generals.html", "aaa_byzantine_generals.png", 5),
        ("collective_maze.html", "aaa_collective_maze.png", 5),
        ("swarm_optimization.html", "aaa_swarm_optimization.png", 5),
        ("recursive_reasoning.html", "aaa_recursive_reasoning.png", 5),
    ]
    
    print("🎮 Capturing AAA Game Demo Screenshots...")
    print("=" * 60)
    
    success_count = 0
    for html_file, output_name, wait_time in demos:
        if os.path.exists(html_file):
            if capture_demo(html_file, output_name, wait_time):
                success_count += 1
        else:
            print(f"⚠️  File not found: {html_file}")
    
    print("=" * 60)
    print(f"✨ Captured {success_count}/{len(demos)} screenshots")
    
    # Alternative: Create static preview images if browser capture fails
    if success_count == 0:
        print("\n🎨 Creating preview images using PIL...")
        create_preview_images()

def create_preview_images():
    """Create preview images using PIL as fallback"""
    try:
        from PIL import Image, ImageDraw, ImageFont
        import random
        
        demos = [
            ("GENIUS Game Launcher", "aaa_game_launcher.png", (20, 20, 40)),
            ("Byzantine Generals", "aaa_byzantine_generals.png", (139, 90, 42)),
            ("Collective Maze", "aaa_collective_maze.png", (0, 100, 100)),
            ("Swarm Optimization", "aaa_swarm_optimization.png", (75, 0, 130)),
            ("Recursive Reasoning", "aaa_recursive_reasoning.png", (255, 20, 147)),
        ]
        
        for title, filename, base_color in demos:
            # Create a 1920x1080 image
            img = Image.new('RGB', (1920, 1080), (10, 10, 20))
            draw = ImageDraw.Draw(img)
            
            # Add gradient background
            for y in range(1080):
                color = (
                    int(base_color[0] * (1 - y/1080)),
                    int(base_color[1] * (1 - y/1080)),
                    int(base_color[2] * (1 - y/1080))
                )
                draw.rectangle([(0, y), (1920, y+1)], fill=color)
            
            # Add some particle effects
            for _ in range(100):
                x = random.randint(0, 1920)
                y = random.randint(0, 1080)
                size = random.randint(1, 3)
                brightness = random.randint(100, 255)
                draw.ellipse([x-size, y-size, x+size, y+size], 
                           fill=(brightness, brightness, brightness))
            
            # Add title
            try:
                font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 80)
            except:
                font = ImageFont.load_default()
            
            text_bbox = draw.textbbox((0, 0), title, font=font)
            text_width = text_bbox[2] - text_bbox[0]
            text_height = text_bbox[3] - text_bbox[1]
            x = (1920 - text_width) // 2
            y = (1080 - text_height) // 2
            
            # Add glow effect
            for offset in range(3, 0, -1):
                alpha = 80 // offset
                draw.text((x, y), title, font=font, 
                         fill=(base_color[0]//2, base_color[1]//2, base_color[2]//2, alpha))
            
            draw.text((x, y), title, font=font, fill=(255, 255, 255))
            
            # Add "AAA Demo Preview" subtitle
            subtitle = "AAA DEMO PREVIEW"
            try:
                subfont = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 30)
            except:
                subfont = font
            
            sub_bbox = draw.textbbox((0, 0), subtitle, font=subfont)
            sub_width = sub_bbox[2] - sub_bbox[0]
            draw.text(((1920 - sub_width) // 2, y + text_height + 20), 
                     subtitle, font=subfont, fill=(150, 150, 150))
            
            img.save(filename)
            print(f"✅ Created preview: {filename}")
            
    except ImportError:
        print("❌ PIL not available. Cannot create preview images.")

if __name__ == "__main__":
    main()