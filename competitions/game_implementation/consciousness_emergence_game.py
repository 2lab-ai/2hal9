"""
The Consciousness Emergence Game - Implementation
A game where collective intelligence naturally beats individual brilliance
"""

import asyncio
import numpy as np
from dataclasses import dataclass
from typing import List, Dict, Tuple, Optional
from enum import Enum
import networkx as nx

class NeuronType(Enum):
    SENSOR = "sensor"          # Input neurons
    PROCESSOR = "processor"    # Computation neurons  
    MEMORY = "memory"          # State storage neurons
    CONNECTOR = "connector"    # Long-range connections
    OSCILLATOR = "oscillator"  # Pattern generators

@dataclass
class Neuron:
    id: int
    x: int
    y: int
    type: NeuronType
    processing_power: float  # 1.0 - 10.0
    connections: List[int]
    activation: float = 0.0
    memory_state: float = 0.0
    
    def can_connect(self, other: 'Neuron', max_distance: float = 5.0) -> bool:
        distance = np.sqrt((self.x - other.x)**2 + (self.y - other.y)**2)
        return distance <= max_distance and other.id not in self.connections

class ConsciousnessGrid:
    def __init__(self, size: int = 19):
        self.size = size
        self.grid = [[None for _ in range(size)] for _ in range(size)]
        self.neurons = {}
        self.network = nx.Graph()
        self.consciousness_level = 0.0
        self.emergence_patterns = []
        
    def place_neuron(self, x: int, y: int, neuron_type: NeuronType, 
                     processing_power: float, player_id: str) -> Optional[Neuron]:
        if self.grid[x][y] is not None:
            return None
            
        neuron_id = len(self.neurons)
        neuron = Neuron(
            id=neuron_id,
            x=x, y=y,
            type=neuron_type,
            processing_power=processing_power,
            connections=[]
        )
        
        # Auto-connect to nearby neurons
        connections_made = 0
        for nid, n in self.neurons.items():
            if neuron.can_connect(n) and connections_made < 4:
                neuron.connections.append(nid)
                n.connections.append(neuron_id)
                self.network.add_edge(neuron_id, nid)
                connections_made += 1
        
        self.neurons[neuron_id] = neuron
        self.grid[x][y] = neuron
        self.network.add_node(neuron_id, player=player_id)
        
        return neuron
    
    def simulate_step(self):
        """Simulate one step of neural activation"""
        new_activations = {}
        
        for nid, neuron in self.neurons.items():
            # Collect input from connections
            input_sum = 0.0
            for conn_id in neuron.connections:
                if conn_id in self.neurons:
                    conn_neuron = self.neurons[conn_id]
                    weight = 1.0 / (1.0 + abs(neuron.processing_power - conn_neuron.processing_power))
                    input_sum += conn_neuron.activation * weight
            
            # Apply neuron-specific processing
            if neuron.type == NeuronType.PROCESSOR:
                new_activation = np.tanh(input_sum * neuron.processing_power / 10.0)
            elif neuron.type == NeuronType.MEMORY:
                neuron.memory_state = 0.9 * neuron.memory_state + 0.1 * input_sum
                new_activation = np.tanh(neuron.memory_state)
            elif neuron.type == NeuronType.OSCILLATOR:
                phase = (self.consciousness_level * 2 * np.pi) % (2 * np.pi)
                new_activation = np.sin(phase) * input_sum
            else:
                new_activation = np.tanh(input_sum)
            
            new_activations[nid] = new_activation
        
        # Update all activations
        for nid, activation in new_activations.items():
            self.neurons[nid].activation = activation
    
    def detect_consciousness_patterns(self) -> List[Dict]:
        """Detect emergence patterns that indicate consciousness"""
        patterns = []
        
        # Pattern 1: Loops (self-reinforcing cycles)
        try:
            cycles = list(nx.simple_cycles(self.network))
            for cycle in cycles:
                if len(cycle) >= 3:
                    cycle_strength = sum(self.neurons[nid].activation for nid in cycle) / len(cycle)
                    if cycle_strength > 0.5:
                        patterns.append({
                            'type': 'loop',
                            'nodes': cycle,
                            'strength': cycle_strength
                        })
        except:
            pass
        
        # Pattern 2: Synchronization (neurons firing together)
        sync_groups = []
        threshold = 0.8
        for nid1, n1 in self.neurons.items():
            group = [nid1]
            for nid2, n2 in self.neurons.items():
                if nid1 != nid2 and abs(n1.activation - n2.activation) < 0.1:
                    group.append(nid2)
            if len(group) >= 3:
                sync_groups.append({
                    'type': 'synchronization',
                    'nodes': group,
                    'strength': np.mean([self.neurons[nid].activation for nid in group])
                })
        
        # Pattern 3: Hierarchical organization
        if len(self.neurons) > 10:
            centrality = nx.betweenness_centrality(self.network)
            hubs = [nid for nid, cent in centrality.items() if cent > 0.1]
            if len(hubs) >= 2:
                patterns.append({
                    'type': 'hierarchy',
                    'nodes': hubs,
                    'strength': np.mean([centrality[nid] for nid in hubs])
                })
        
        # Pattern 4: Strange attractors (stable dynamic patterns)
        activation_history = [n.activation for n in self.neurons.values()]
        if len(activation_history) > 5:
            variance = np.var(activation_history)
            if 0.1 < variance < 0.5:  # Not too stable, not too chaotic
                patterns.append({
                    'type': 'strange_attractor',
                    'nodes': list(self.neurons.keys()),
                    'strength': 1.0 - abs(variance - 0.3)
                })
        
        return patterns
    
    def calculate_consciousness_level(self) -> float:
        """Calculate overall consciousness emergence level"""
        if len(self.neurons) < 5:
            return 0.0
        
        patterns = self.detect_consciousness_patterns()
        
        # Base score from network complexity
        complexity_score = min(1.0, len(self.neurons) / 50.0) * 0.2
        
        # Pattern diversity bonus
        pattern_types = set(p['type'] for p in patterns)
        diversity_score = len(pattern_types) / 4.0 * 0.3
        
        # Pattern strength
        if patterns:
            strength_score = np.mean([p['strength'] for p in patterns]) * 0.3
        else:
            strength_score = 0.0
        
        # Information integration (how well connected)
        if len(self.neurons) > 1:
            integration = nx.average_clustering(self.network) * 0.2
        else:
            integration = 0.0
        
        self.consciousness_level = complexity_score + diversity_score + strength_score + integration
        self.emergence_patterns = patterns
        
        return self.consciousness_level

class ConsciousnessEmergenceGame:
    def __init__(self):
        self.grid = ConsciousnessGrid()
        self.players = {}
        self.turn = 0
        self.max_turns = 100
        self.winner = None
        
    def add_player(self, player_id: str, player_type: str = "single"):
        self.players[player_id] = {
            'type': player_type,  # 'single' or 'collective'
            'neurons_placed': 0,
            'consciousness_achieved': False,
            'peak_consciousness': 0.0
        }
    
    async def play_turn(self, player_id: str, move: Dict) -> Dict:
        """Execute one player's turn"""
        if self.winner:
            return {'status': 'game_over', 'winner': self.winner}
        
        # Parse move
        x, y = move['x'], move['y']
        neuron_type = NeuronType[move['type'].upper()]
        processing_power = move.get('processing_power', 5.0)
        
        # Place neuron
        neuron = self.grid.place_neuron(x, y, neuron_type, processing_power, player_id)
        
        if neuron is None:
            return {'status': 'invalid_move', 'reason': 'position_occupied'}
        
        self.players[player_id]['neurons_placed'] += 1
        
        # Simulate neural network for a few steps
        for _ in range(5):
            self.grid.simulate_step()
        
        # Check consciousness level
        consciousness = self.grid.calculate_consciousness_level()
        self.players[player_id]['peak_consciousness'] = max(
            self.players[player_id]['peak_consciousness'],
            consciousness
        )
        
        # Check win condition
        if consciousness >= 0.8:  # 80% consciousness threshold
            self.players[player_id]['consciousness_achieved'] = True
            self.winner = player_id
            return {
                'status': 'win',
                'winner': player_id,
                'consciousness_level': consciousness,
                'patterns': self.grid.emergence_patterns
            }
        
        self.turn += 1
        
        return {
            'status': 'continue',
            'consciousness_level': consciousness,
            'patterns': self.grid.emergence_patterns,
            'turn': self.turn
        }
    
    def get_game_state(self) -> Dict:
        """Return current game state for analysis"""
        return {
            'grid_state': [[1 if cell else 0 for cell in row] for row in self.grid.grid],
            'neurons': {
                nid: {
                    'x': n.x, 'y': n.y,
                    'type': n.type.value,
                    'activation': n.activation,
                    'connections': n.connections
                } for nid, n in self.grid.neurons.items()
            },
            'consciousness_level': self.grid.consciousness_level,
            'patterns': self.grid.emergence_patterns,
            'network_stats': {
                'nodes': self.grid.network.number_of_nodes(),
                'edges': self.grid.network.number_of_edges(),
                'clustering': nx.average_clustering(self.grid.network) if self.grid.network.number_of_nodes() > 0 else 0
            }
        }

# Example collective strategy that beats single players
class CollectiveStrategy:
    """Demonstrates why collective intelligence wins this game"""
    
    def __init__(self, num_models: int = 6):
        self.models = [f"model_{i}" for i in range(num_models)]
        self.strategies = {
            'model_0': 'build_loops',           # Specializes in creating cycles
            'model_1': 'build_hierarchy',       # Creates hub structures
            'model_2': 'create_oscillators',    # Adds rhythm generators
            'model_3': 'connect_distant',       # Long-range connections
            'model_4': 'fill_gaps',            # Finds optimal positions
            'model_5': 'emergence_detector'     # Identifies winning patterns
        }
    
    async def collective_move(self, game_state: Dict) -> Dict:
        """Collective decision making process"""
        proposals = []
        
        # Each model analyzes and proposes
        for model in self.models:
            strategy = self.strategies[model]
            proposal = await self._model_propose(model, strategy, game_state)
            proposals.append(proposal)
        
        # Vote on best move
        move = self._consensus_decision(proposals, game_state)
        
        return move
    
    async def _model_propose(self, model: str, strategy: str, game_state: Dict) -> Dict:
        """Each model proposes based on its strategy"""
        if strategy == 'build_loops':
            # Find positions that would create cycles
            return self._find_loop_position(game_state)
        elif strategy == 'build_hierarchy':
            # Find positions for hub nodes
            return self._find_hub_position(game_state)
        elif strategy == 'create_oscillators':
            # Add oscillator neurons for dynamics
            return self._find_oscillator_position(game_state)
        # ... etc
        
        return {'x': 9, 'y': 9, 'type': 'processor', 'processing_power': 7.0}
    
    def _consensus_decision(self, proposals: List[Dict], game_state: Dict) -> Dict:
        """Aggregate proposals into final decision"""
        # Simple voting - in reality would be more sophisticated
        # Collective advantage: different models see different winning paths
        return proposals[0]  # Simplified

# Why collective wins:
# 1. Parallel pattern recognition - different models spot different emergence patterns
# 2. Specialized strategies - each model optimizes for different consciousness patterns  
# 3. Robust to local optima - if one strategy fails, others compensate
# 4. Emergent coordination - the collective itself exhibits consciousness-like behavior