from PIL import Image, ImageDraw, ImageFont, ImageFilter
import numpy as np
import os
from datetime import datetime

def create_death_game_frames():
    """Create dramatic death game demo frames"""
    frames = []
    width, height = 1200, 800
    
    # Font setup
    try:
        title_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 72)
        header_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 48)
        normal_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 28)
        small_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 20)
    except:
        title_font = ImageFont.load_default()
        header_font = ImageFont.load_default()
        normal_font = ImageFont.load_default()
        small_font = ImageFont.load_default()
    
    # Scene 1: Title Screen with blood effect
    for i in range(20):
        img = Image.new('RGB', (width, height), color='#000000')
        draw = ImageDraw.Draw(img)
        
        # Dark red gradient background
        for y in range(height):
            intensity = int(20 + (y / height) * 20)
            draw.rectangle([(0, y), (width, y+1)], fill=(intensity, 0, 0))
        
        # Blood drops
        for _ in range(50):
            x = np.random.randint(0, width)
            y = np.random.randint(0, height)
            size = np.random.randint(2, 5)
            draw.ellipse([(x, y), (x+size, y+size*3)], fill=(139, 0, 0))
        
        # Title with glow effect
        title = "⚔️ AI DEATH GAME CHAMPIONSHIP ⚔️"
        
        # Create glow
        for offset in range(5, 0, -1):
            glow_color = (255, 0, 0, 50 * (6-offset))
            for dx in [-offset, 0, offset]:
                for dy in [-offset, 0, offset]:
                    draw.text((width//2 + dx, 100 + dy), title, 
                            font=title_font, anchor="mm", fill=glow_color)
        
        # Main title
        draw.text((width//2, 100), title, font=title_font, anchor="mm", fill="#FF0000")
        
        # Subtitle
        subtitle = "Only One AI Survives"
        draw.text((width//2, 180), subtitle, font=header_font, anchor="mm", fill="#FFFFFF")
        
        # Animated countdown
        if i >= 10:
            countdown = 3 - (i - 10) // 3
            if countdown > 0:
                count_size = 120 + (i % 3) * 20
                count_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', count_size) if count_size <= 140 else title_font
                draw.text((width//2, height//2), str(countdown), 
                        font=count_font, anchor="mm", fill="#FF0000")
        
        frames.append(img)
    
    # Scene 2: Go Game Battle
    go_board_size = 9
    cell_size = 50
    board_x = 100
    board_y = 200
    
    # Game state
    go_moves = [
        # Opening moves
        [(2, 2, 'black'), (6, 6, 'white')],
        [(6, 2, 'black'), (2, 6, 'white')],
        [(4, 4, 'black'), (3, 3, 'white')],
        [(5, 3, 'black'), (3, 5, 'white')],
        # Battle intensifies
        [(4, 3, 'black'), (4, 5, 'white')],
        [(3, 4, 'black'), (5, 4, 'white')],
        # Capture!
        [(5, 5, 'black', 'capture'), (None, None, None)],
    ]
    
    board_state = [[None for _ in range(go_board_size)] for _ in range(go_board_size)]
    
    for move_set in go_moves:
        for move_frame in range(10):
            img = Image.new('RGB', (width, height), color='#1a1a1a')
            draw = ImageDraw.Draw(img)
            
            # Title
            draw.text((width//2, 50), "🏯 MINI GO - DEATH MATCH", 
                    font=header_font, anchor="mm", fill="#FF0000")
            
            # Player panels
            # Player 1
            draw.rectangle([(50, 150), (350, 350)], fill=(20, 0, 0), outline="#FF0000", width=3)
            draw.text((200, 170), "🤖 AI GLADIATOR α", font=normal_font, anchor="mm", fill="#FFFFFF")
            draw.text((200, 210), "Strategy: Territory Control", font=small_font, anchor="mm", fill="#FF6666")
            draw.text((200, 250), f"Captures: {len([m for m in go_moves[:go_moves.index(move_set)] if len(m[0]) > 3])}", 
                    font=normal_font, anchor="mm", fill="#00FF00")
            
            # Player 2
            draw.rectangle([(850, 150), (1150, 350)], fill=(0, 0, 20), outline="#0000FF", width=3)
            draw.text((1000, 170), "🎭 AI GLADIATOR β", font=normal_font, anchor="mm", fill="#FFFFFF")
            draw.text((1000, 210), "Strategy: Aggressive Invasion", font=small_font, anchor="mm", fill="#6666FF")
            draw.text((1000, 250), f"Territory: {np.random.randint(30, 50)}%", 
                    font=normal_font, anchor="mm", fill="#00FF00")
            
            # Go board
            board_left = 400
            board_top = 200
            
            # Board background
            draw.rectangle([(board_left-10, board_top-10), 
                          (board_left + cell_size * go_board_size + 10, 
                           board_top + cell_size * go_board_size + 10)], 
                         fill="#8B4513", outline="#FF0000", width=3)
            
            # Grid
            for i in range(go_board_size):
                # Vertical lines
                x = board_left + i * cell_size
                draw.line([(x, board_top), (x, board_top + cell_size * (go_board_size-1))], 
                         fill="#000000", width=1)
                # Horizontal lines
                y = board_top + i * cell_size
                draw.line([(board_left, y), (board_left + cell_size * (go_board_size-1), y)], 
                         fill="#000000", width=1)
            
            # Place stones from history
            for move in move_set:
                if move[0] is not None and move_frame >= 5:
                    row, col, color = move[:3]
                    board_state[row][col] = color
            
            # Draw all stones
            for r in range(go_board_size):
                for c in range(go_board_size):
                    if board_state[r][c]:
                        x = board_left + c * cell_size
                        y = board_top + r * cell_size
                        
                        if board_state[r][c] == 'black':
                            draw.ellipse([(x-18, y-18), (x+18, y+18)], fill="#000000", outline="#333333")
                        else:
                            draw.ellipse([(x-18, y-18), (x+18, y+18)], fill="#FFFFFF", outline="#CCCCCC")
            
            # Capture animation
            if len(move_set[0]) > 3 and move_set[0][3] == 'capture' and move_frame >= 5:
                capture_x = board_left + 5 * cell_size
                capture_y = board_top + 4 * cell_size
                radius = 30 + move_frame * 3
                opacity = max(0, 255 - move_frame * 25)
                
                # Capture effect
                for ring in range(3):
                    draw.ellipse([(capture_x - radius - ring*10, capture_y - radius - ring*10),
                                (capture_x + radius + ring*10, capture_y + radius + ring*10)],
                               outline=(255, 0, 0, opacity // (ring+1)), width=3)
                
                draw.text((width//2, height-100), "CAPTURE! +10 POINTS", 
                        font=header_font, anchor="mm", fill="#FF0000")
            
            frames.append(img)
    
    # Scene 3: Hold'em Showdown
    for round_num in range(4):  # Pre-flop, Flop, Turn, River
        for frame in range(15):
            img = Image.new('RGB', (width, height), color='#001a00')
            draw = ImageDraw.Draw(img)
            
            # Title
            draw.text((width//2, 50), "🃏 MINI HOLD'EM - ALL IN", 
                    font=header_font, anchor="mm", fill="#FFD700")
            
            # Poker table (oval)
            table_center = (width//2, height//2)
            table_width = 500
            table_height = 250
            
            # Table shadow
            draw.ellipse([(table_center[0] - table_width//2 + 10, table_center[1] - table_height//2 + 10),
                        (table_center[0] + table_width//2 + 10, table_center[1] + table_height//2 + 10)],
                       fill="#000000")
            
            # Table
            draw.ellipse([(table_center[0] - table_width//2, table_center[1] - table_height//2),
                        (table_center[0] + table_width//2, table_center[1] + table_height//2)],
                       fill="#2d5a2d", outline="#8B4513", width=10)
            
            # Pot
            pot_size = 1000 + round_num * 500
            draw.text((width//2, height//2), f"POT: ${pot_size}", 
                    font=header_font, anchor="mm", fill="#FFD700")
            
            # Community cards
            card_y = height//2 - 50
            cards_to_show = 0
            if round_num >= 1: cards_to_show = 3  # Flop
            if round_num >= 2: cards_to_show = 4  # Turn
            if round_num >= 3: cards_to_show = 5  # River
            
            community_cards = ['A♠', 'K♥', 'Q♦', '10♣', '9♠']
            
            for i in range(cards_to_show):
                if frame >= i * 3:
                    card_x = width//2 - 120 + i * 60
                    # Card
                    draw.rectangle([(card_x - 25, card_y - 35), (card_x + 25, card_y + 35)],
                                 fill="#FFFFFF", outline="#000000", width=2)
                    # Card value
                    card = community_cards[i]
                    color = "#FF0000" if '♥' in card or '♦' in card else "#000000"
                    draw.text((card_x, card_y), card, font=normal_font, anchor="mm", fill=color)
            
            # Player 1 info
            p1_chips = 1000 - round_num * 200
            draw.rectangle([(50, 500), (350, 700)], fill=(30, 0, 0), outline="#FF0000", width=3)
            draw.text((200, 520), "🤖 AI GLADIATOR α", font=normal_font, anchor="mm", fill="#FFFFFF")
            draw.text((200, 560), f"Chips: ${p1_chips}", font=normal_font, anchor="mm", fill="#FFD700")
            draw.text((200, 600), "Hand: A♣ K♣", font=small_font, anchor="mm", fill="#FFFFFF")
            draw.text((200, 640), "Action: ALL IN!", font=normal_font, anchor="mm", fill="#FF0000")
            
            # Player 2 info
            p2_chips = 1000 - round_num * 150
            draw.rectangle([(850, 500), (1150, 700)], fill=(0, 0, 30), outline="#0000FF", width=3)
            draw.text((1000, 520), "🎭 AI GLADIATOR β", font=normal_font, anchor="mm", fill="#FFFFFF")
            draw.text((1000, 560), f"Chips: ${p2_chips}", font=normal_font, anchor="mm", fill="#FFD700")
            draw.text((1000, 600), "Hand: Q♥ Q♠", font=small_font, anchor="mm", fill="#FFFFFF")
            draw.text((1000, 640), "Action: CALL!", font=normal_font, anchor="mm", fill="#00FF00")
            
            frames.append(img)
    
    # Scene 4: Final Elimination
    for i in range(30):
        img = Image.new('RGB', (width, height), color='#000000')
        draw = ImageDraw.Draw(img)
        
        # Dramatic background
        for y in range(height):
            intensity = int(50 * abs(np.sin(y / 50 + i / 5)))
            draw.rectangle([(0, y), (width, y+1)], fill=(intensity, 0, 0))
        
        # Lightning effect
        if i % 5 == 0:
            for _ in range(3):
                start_x = np.random.randint(0, width)
                points = [(start_x, 0)]
                y = 0
                while y < height:
                    y += np.random.randint(20, 50)
                    x = points[-1][0] + np.random.randint(-50, 50)
                    points.append((x, y))
                
                draw.line(points, fill=(255, 255, 255), width=3)
        
        # Title
        draw.text((width//2, 100), "⚔️ FINAL ELIMINATION ⚔️", 
                font=title_font, anchor="mm", fill="#FF0000")
        
        # Winner announcement
        if i >= 15:
            winner_scale = 1 + (i - 15) * 0.1
            winner_font_size = int(48 * winner_scale)
            try:
                winner_font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', winner_font_size)
            except:
                winner_font = header_font
            
            draw.text((width//2, height//2), "🏆 AI GLADIATOR α WINS! 🏆", 
                    font=winner_font, anchor="mm", fill="#FFD700")
            
            # Stats
            draw.text((width//2, height//2 + 100), "Final Score: 2,500", 
                    font=normal_font, anchor="mm", fill="#FFFFFF")
            draw.text((width//2, height//2 + 150), "Eliminations: 3", 
                    font=normal_font, anchor="mm", fill="#FF0000")
            draw.text((width//2, height//2 + 200), "Survival Time: 15:23", 
                    font=normal_font, anchor="mm", fill="#00FF00")
        
        # Eliminated player
        if i < 15:
            elim_y = 300 + i * 20
            draw.text((width//2, elim_y), "AI GLADIATOR β ELIMINATED", 
                    font=header_font, anchor="mm", fill="#666666")
        
        frames.append(img)
    
    return frames

def create_death_game_demo_gif():
    print("🎬 Creating AI Death Game demo GIF...")
    
    frames = create_death_game_frames()
    
    output_path = '/Users/icedac/2lab.ai/2hal9/competitions/genius_game_server/demo/death_game_demo.gif'
    
    # Save as GIF with optimization
    frames[0].save(
        output_path,
        save_all=True,
        append_images=frames[1:],
        duration=100,  # 100ms per frame
        loop=0,
        optimize=True
    )
    
    print(f"✅ Death Game demo GIF created: {output_path}")
    print(f"📊 Total frames: {len(frames)}")
    print(f"⏱️  Duration: ~{len(frames) * 0.1:.1f} seconds")
    
    # Also save some key frames as screenshots
    key_frames = [
        (frames[10], "death_game_title.png"),
        (frames[50], "death_game_go.png"),
        (frames[100], "death_game_holdem.png"),
        (frames[140], "death_game_winner.png"),
    ]
    
    for frame, filename in key_frames:
        frame.save(f"/Users/icedac/2lab.ai/2hal9/competitions/genius_game_server/demo/{filename}")
        print(f"📸 Saved screenshot: {filename}")

if __name__ == "__main__":
    create_death_game_demo_gif()