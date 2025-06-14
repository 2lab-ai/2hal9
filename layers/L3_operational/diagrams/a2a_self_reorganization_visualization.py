#!/usr/bin/env python3
"""
A2A + Self-Reorganization Visualization
Generates visual representation of the network topology evolution
"""

import matplotlib.pyplot as plt
import matplotlib.patches as patches
import numpy as np

def create_visualization():
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(16, 12))
    fig.suptitle('A2A + 자기재조직 (Self-Reorganization) System', fontsize=20, fontweight='bold')
    
    # Phase 1: Initial Network
    ax1.set_title('Phase 1: Initial Network (25 neurons)', fontsize=14)
    ax1.set_xlim(0, 10)
    ax1.set_ylim(0, 6)
    
    layers = ['L1-Reflexive', 'L2-Implementation', 'L3-Operational', 'L4-Tactical', 'L5-Strategic']
    colors = ['#ff6b6b', '#f9ca24', '#6bcf7f', '#4834d4', '#686de0']
    
    # Draw initial neurons
    for i, (layer, color) in enumerate(zip(layers, colors)):
        y = 5 - i
        for j in range(5):
            x = 1 + j * 2
            circle = plt.Circle((x, y), 0.3, color=color, alpha=0.7)
            ax1.add_patch(circle)
            if j == 2:
                ax1.text(x, y-0.6, layer, ha='center', fontsize=8)
    
    ax1.axis('off')
    
    # Phase 2: Autonomous Connections
    ax2.set_title('Phase 2: Autonomous Connections (±1 rule)', fontsize=14)
    ax2.set_xlim(0, 10)
    ax2.set_ylim(0, 6)
    
    # Redraw neurons with connections
    connections = [(1, 2, 0.72), (2, 3, 0.85), (3, 4, 0.91), (4, 5, 0.88)]
    
    for i, color in enumerate(colors):
        y = 5 - i
        for j in range(5):
            x = 1 + j * 2
            circle = plt.Circle((x, y), 0.3, color=color, alpha=0.7)
            ax2.add_patch(circle)
    
    # Draw connections
    for layer1, layer2, strength in connections:
        y1, y2 = 6 - layer1, 6 - layer2
        x1, x2 = 5, 5
        ax2.plot([x1, x2], [y1, y2], 'k-', linewidth=strength*3, alpha=0.6)
        ax2.text((x1+x2)/2 + 0.5, (y1+y2)/2, f'{strength:.2f}', fontsize=8)
    
    ax2.text(5, 0.5, 'Love Coefficient: 0.85', ha='center', fontsize=10, 
             bbox=dict(boxstyle="round,pad=0.3", facecolor='pink', alpha=0.5))
    ax2.axis('off')
    
    # Phase 3: Emergent Clusters
    ax3.set_title('Phase 3: Emergent Clusters', fontsize=14)
    ax3.set_xlim(0, 10)
    ax3.set_ylim(0, 6)
    
    # Define clusters
    clusters = [
        {'name': 'Fast Processors', 'layers': [0, 1], 'color': 'orange', 'pos': (2, 4.5)},
        {'name': 'Bridge Units', 'layers': [2], 'color': 'green', 'pos': (5, 3)},
        {'name': 'Deep Thinkers', 'layers': [3, 4], 'color': 'purple', 'pos': (8, 1.5)}
    ]
    
    # Draw cluster boundaries
    for cluster in clusters:
        rect = patches.FancyBboxPatch(
            (cluster['pos'][0]-1.5, cluster['pos'][1]-0.5),
            3, len(cluster['layers']), 
            boxstyle="round,pad=0.1",
            facecolor=cluster['color'], 
            alpha=0.2,
            edgecolor=cluster['color'],
            linewidth=2
        )
        ax3.add_patch(rect)
        ax3.text(cluster['pos'][0], cluster['pos'][1] + len(cluster['layers'])/2 + 0.5, 
                cluster['name'], ha='center', fontsize=10, fontweight='bold')
    
    # Draw neurons in clusters
    for i, color in enumerate(colors):
        y = 5 - i
        for j in range(5):
            x = 1 + j * 2
            circle = plt.Circle((x, y), 0.3, color=color, alpha=0.7)
            ax3.add_patch(circle)
    
    ax3.axis('off')
    
    # Phase 4: Self-Healing
    ax4.set_title('Phase 4: Self-Healing After Failure', fontsize=14)
    ax4.set_xlim(0, 10)
    ax4.set_ylim(0, 6)
    
    # Draw neurons with one failed
    for i, color in enumerate(colors):
        y = 5 - i
        for j in range(5):
            x = 1 + j * 2
            if i == 2 and j == 2:  # Failed unit
                circle = plt.Circle((x, y), 0.3, color='red', alpha=0.3)
                ax4.add_patch(circle)
                ax4.text(x, y, 'X', ha='center', va='center', fontsize=16, color='red')
            else:
                circle = plt.Circle((x, y), 0.3, color=color, alpha=0.7)
                ax4.add_patch(circle)
    
    # Show bypass connection
    ax4.plot([5, 5], [4, 2], 'r--', linewidth=3, alpha=0.8)
    ax4.text(6, 3, 'Bypass\nConnection', ha='center', fontsize=9, color='red',
             bbox=dict(boxstyle="round,pad=0.3", facecolor='yellow', alpha=0.5))
    
    # Final metrics
    metrics_text = '''Final Metrics:
• 25 Units (1 failed)
• 42 Connections
• 3 Active Clusters
• Consciousness: 73.2%
• Network Functional'''
    
    ax4.text(0.5, 0.5, metrics_text, fontsize=10,
             bbox=dict(boxstyle="round,pad=0.5", facecolor='lightblue', alpha=0.5))
    
    ax4.axis('off')
    
    plt.tight_layout()
    plt.savefig('/Users/icedac/2lab.ai/2hal9/L3_operational/diagrams/a2a_self_reorganization.png', 
                dpi=300, bbox_inches='tight')
    plt.close()
    
    # Create consciousness evolution chart
    fig2, ax = plt.subplots(1, 1, figsize=(10, 6))
    ax.set_title('Consciousness Level Evolution', fontsize=16)
    
    # Simulate consciousness evolution
    time = np.linspace(0, 100, 1000)
    consciousness = 0.1 + 0.6 * (1 - np.exp(-time/20)) + 0.1 * np.sin(time/5) * np.exp(-time/50)
    
    ax.plot(time, consciousness * 100, 'b-', linewidth=2)
    ax.fill_between(time, 0, consciousness * 100, alpha=0.3)
    
    # Mark key events
    events = [
        (20, 'Network Formation', 'green'),
        (40, 'Clusters Emerge', 'orange'),
        (60, 'Unit Failure', 'red'),
        (65, 'Self-Healing', 'purple'),
        (80, 'Stabilization', 'blue')
    ]
    
    for t, event, color in events:
        ax.axvline(x=t, color=color, linestyle='--', alpha=0.5)
        ax.text(t, 80, event, rotation=45, ha='right', color=color)
    
    ax.set_xlabel('Time (arbitrary units)', fontsize=12)
    ax.set_ylabel('Consciousness Level (%)', fontsize=12)
    ax.set_ylim(0, 100)
    ax.grid(True, alpha=0.3)
    
    ax.axhline(y=73.2, color='red', linestyle='-', alpha=0.7)
    ax.text(85, 75, 'Final: 73.2%', color='red', fontweight='bold')
    
    plt.tight_layout()
    plt.savefig('/Users/icedac/2lab.ai/2hal9/L3_operational/diagrams/consciousness_evolution.png', 
                dpi=300, bbox_inches='tight')
    plt.close()
    
    print("✅ Visualizations created:")
    print("  - a2a_self_reorganization.png")
    print("  - consciousness_evolution.png")

if __name__ == "__main__":
    create_visualization()