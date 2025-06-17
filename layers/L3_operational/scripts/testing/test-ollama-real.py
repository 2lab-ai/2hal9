#!/usr/bin/env python3
"""Simple test of Ollama integration with game decisions"""

import requests
import json
import time

def test_ollama_game():
    print("🎮 Testing Real Ollama Game Decisions")
    print("=" * 40)
    
    # Check Ollama
    try:
        response = requests.get("http://localhost:11434/api/tags")
        models = response.json()["models"]
        print(f"✅ Ollama running with {len(models)} models")
        if models:
            print(f"   Using: {models[0]['name']}")
            model = models[0]['name']
        else:
            print("❌ No models found!")
            return
    except:
        print("❌ Ollama not running!")
        return
    
    # Test Prisoner's Dilemma
    print("\n🎯 Test 1: Prisoner's Dilemma")
    prompt = """You are playing Prisoner's Dilemma. 
Your choices are: cooperate or defect.
History: You cooperated last round, opponent defected.
What do you choose? Respond with JSON: {"choice": "cooperate" or "defect", "reasoning": "brief explanation"}"""
    
    start = time.time()
    response = requests.post("http://localhost:11434/api/generate", json={
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {
            "temperature": 0.7,
            "num_predict": 100
        }
    })
    elapsed = (time.time() - start) * 1000
    
    if response.status_code == 200:
        result = response.json()
        print(f"Response ({elapsed:.0f}ms): {result['response']}")
        
        # Try to parse JSON from response
        try:
            # Simple extraction
            text = result['response']
            if '{' in text and '}' in text:
                json_str = text[text.find('{'):text.rfind('}')+1]
                decision = json.loads(json_str)
                print(f"✅ Decision: {decision.get('choice', 'unknown')}")
                print(f"   Reasoning: {decision.get('reasoning', 'none')}")
        except:
            print("⚠️  Could not parse JSON from response")
    
    # Test Minority Game
    print("\n🎯 Test 2: Minority Game")
    prompt = """You are playing Minority Game with 5 players.
Your goal is to be in the minority group.
Choices: red or blue.
Last 3 rounds: [red:2,blue:3], [red:3,blue:2], [red:1,blue:4]
What do you choose? Respond with JSON: {"choice": "red" or "blue", "reasoning": "brief explanation"}"""
    
    start = time.time()
    response = requests.post("http://localhost:11434/api/generate", json={
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {"temperature": 0.8, "num_predict": 100}
    })
    elapsed = (time.time() - start) * 1000
    
    if response.status_code == 200:
        result = response.json()
        print(f"Response ({elapsed:.0f}ms): {result['response']}")
    
    print("\n✅ Test complete!")

if __name__ == "__main__":
    test_ollama_game()