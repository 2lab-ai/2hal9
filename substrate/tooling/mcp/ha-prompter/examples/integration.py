#!/usr/bin/env python3
"""
Example integration of HA Prompter with LLM applications
Shows how to use the MCP tool in Python
"""

import json
import subprocess
import asyncio
from typing import Dict, Any, Optional

class HAPrompterClient:
    """Client for interacting with HA Prompter MCP tool"""
    
    def __init__(self, binary_path: str = "./target/release/ha-prompter"):
        self.binary_path = binary_path
        self.process = None
        
    async def start(self):
        """Start the HA Prompter process"""
        self.process = await asyncio.create_subprocess_exec(
            self.binary_path,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )
        
        # Read capabilities
        capabilities_line = await self.process.stdout.readline()
        self.capabilities = json.loads(capabilities_line.decode())
        print(f"Connected to {self.capabilities['name']} v{self.capabilities['version']}")
        
    async def compress(self, content: str, data_type: str, target_level: int, 
                      current_level: Optional[int] = None) -> Dict[str, Any]:
        """Compress content to a higher level"""
        request = {
            "tool": "compress",
            "parameters": {
                "content": content,
                "data_type": data_type,
                "target_level": target_level
            }
        }
        if current_level is not None:
            request["parameters"]["current_level"] = current_level
            
        return await self._send_request(request)
    
    async def expand(self, content: str, data_type: str, 
                    from_level: int, to_level: int) -> Dict[str, Any]:
        """Expand content from one level to another"""
        request = {
            "tool": "expand",
            "parameters": {
                "content": content,
                "data_type": data_type,
                "from_level": from_level,
                "to_level": to_level
            }
        }
        return await self._send_request(request)
    
    async def cascade_down(self, content: str, data_type: str) -> Dict[str, Any]:
        """Generate cascade from L9 to L1"""
        request = {
            "tool": "cascade_down",
            "parameters": {
                "content": content,
                "data_type": data_type
            }
        }
        return await self._send_request(request)
    
    async def cascade_up(self, content: str, data_type: str) -> Dict[str, Any]:
        """Generate cascade from L1 to L9"""
        request = {
            "tool": "cascade_up", 
            "parameters": {
                "content": content,
                "data_type": data_type
            }
        }
        return await self._send_request(request)
    
    async def analyze(self, content: str, data_type: str) -> Dict[str, Any]:
        """Analyze content to determine its level"""
        request = {
            "tool": "analyze",
            "parameters": {
                "content": content,
                "data_type": data_type
            }
        }
        return await self._send_request(request)
    
    async def _send_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Send request to HA Prompter and get response"""
        request_line = json.dumps(request) + "\n"
        self.process.stdin.write(request_line.encode())
        await self.process.stdin.drain()
        
        response_line = await self.process.stdout.readline()
        return json.loads(response_line.decode())
    
    async def close(self):
        """Close the HA Prompter process"""
        if self.process:
            self.process.terminate()
            await self.process.wait()


async def example_usage():
    """Example usage of HA Prompter"""
    
    # Initialize client
    client = HAPrompterClient()
    await client.start()
    
    print("\n=== Example 1: Compress Bug Report to Philosophy ===")
    result = await client.compress(
        content="Users can't log in after 30 minutes of inactivity",
        data_type="bug report",
        target_level=9,
        current_level=3
    )
    print(f"Generated prompt preview:\n{result['result']['prompt'][:200]}...\n")
    
    print("\n=== Example 2: Expand Philosophy to Code ===")
    result = await client.expand(
        content="Consciousness emerges from recursive self-observation",
        data_type="philosophical concept",
        from_level=9,
        to_level=2
    )
    print(f"Generated prompt preview:\n{result['result']['prompt'][:200]}...\n")
    
    print("\n=== Example 3: Analyze Email Level ===")
    result = await client.analyze(
        content="We need to leverage synergies to maximize stakeholder value",
        data_type="corporate email",
    )
    print(f"Analysis prompt preview:\n{result['result']['prompt'][:200]}...\n")
    
    print("\n=== Example 4: Cascade Down a Concept ===")
    result = await client.cascade_down(
        content="Love",
        data_type="emotion",
    )
    print(f"Cascade prompt preview:\n{result['result']['prompt'][:200]}...\n")
    
    # Cleanup
    await client.close()


def integrate_with_llm(ha_prompt: str, user_content: str) -> str:
    """
    Example of how to integrate HA Prompter with an LLM
    
    In real usage, you would:
    1. Get the prompt from HA Prompter
    2. Combine with user content
    3. Send to your LLM of choice
    """
    
    full_prompt = f"""
{ha_prompt}

User Content:
{user_content}

Please provide your response following the hierarchical abstraction guidelines above.
"""
    
    # Here you would call your LLM API
    # response = llm_api.complete(full_prompt)
    
    return full_prompt


if __name__ == "__main__":
    # Run async example
    asyncio.run(example_usage())
    
    # Show integration example
    print("\n=== LLM Integration Example ===")
    example_prompt = """You need to compress the following bug report to L9 level.

Target Level: Universal (9)
Philosophy, existence, consciousness itself

Content to Compress:
Database connection timeout

Provide your L9 level compression:"""
    
    integrated = integrate_with_llm(example_prompt, "Database connection timeout after 30 seconds")
    print(f"Full prompt for LLM:\n{integrated[:300]}...")