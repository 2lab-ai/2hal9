#!/usr/bin/env python3
"""
HAL9 WebSocket Connection Test Script
"""

import asyncio
import json
import sys
import time
import websockets
from datetime import datetime

# Configuration
WS_URL = "ws://localhost:8080/ws"
TEST_DURATION = 10  # seconds

class WebSocketTester:
    def __init__(self):
        self.messages_sent = 0
        self.messages_received = 0
        self.errors = 0
        self.start_time = None
        self.latencies = []
        
    async def test_connection(self):
        """Test basic WebSocket connection"""
        print(f"🔌 Connecting to {WS_URL}...")
        
        try:
            async with websockets.connect(WS_URL) as websocket:
                print("✅ Connected successfully!")
                return True
        except Exception as e:
            print(f"❌ Connection failed: {e}")
            return False
    
    async def test_echo(self):
        """Test echo functionality"""
        print("\n📤 Testing echo...")
        
        try:
            async with websockets.connect(WS_URL) as websocket:
                # Send test message
                test_msg = {
                    "type": "echo",
                    "payload": "Hello HAL9!",
                    "timestamp": datetime.now().isoformat()
                }
                
                await websocket.send(json.dumps(test_msg))
                self.messages_sent += 1
                
                # Wait for response
                response = await asyncio.wait_for(websocket.recv(), timeout=5.0)
                self.messages_received += 1
                
                print(f"📥 Received: {response}")
                return True
                
        except asyncio.TimeoutError:
            print("❌ Timeout waiting for response")
            self.errors += 1
            return False
        except Exception as e:
            print(f"❌ Echo test failed: {e}")
            self.errors += 1
            return False
    
    async def test_signal_subscription(self):
        """Test signal subscription"""
        print("\n📡 Testing signal subscription...")
        
        try:
            async with websockets.connect(WS_URL) as websocket:
                # Subscribe to signals
                subscribe_msg = {
                    "type": "subscribe",
                    "channel": "signals",
                    "filter": {
                        "layers": ["L2", "L3", "L4"]
                    }
                }
                
                await websocket.send(json.dumps(subscribe_msg))
                self.messages_sent += 1
                
                # Wait for subscription confirmation
                response = await asyncio.wait_for(websocket.recv(), timeout=5.0)
                self.messages_received += 1
                
                data = json.loads(response)
                if data.get("type") == "subscribed":
                    print("✅ Successfully subscribed to signals")
                    return True
                else:
                    print(f"⚠️  Unexpected response: {data}")
                    return False
                    
        except asyncio.TimeoutError:
            print("❌ Timeout waiting for subscription confirmation")
            self.errors += 1
            return False
        except Exception as e:
            print(f"❌ Subscription test failed: {e}")
            self.errors += 1
            return False
    
    async def test_performance(self):
        """Test WebSocket performance with multiple messages"""
        print(f"\n⚡ Testing performance (sending messages for {TEST_DURATION}s)...")
        
        try:
            async with websockets.connect(WS_URL) as websocket:
                self.start_time = time.time()
                end_time = self.start_time + TEST_DURATION
                
                # Send messages continuously
                message_id = 0
                send_times = {}
                
                async def send_messages():
                    nonlocal message_id
                    while time.time() < end_time:
                        msg = {
                            "type": "ping",
                            "id": message_id,
                            "timestamp": time.time()
                        }
                        send_times[message_id] = time.time()
                        await websocket.send(json.dumps(msg))
                        self.messages_sent += 1
                        message_id += 1
                        await asyncio.sleep(0.1)  # 10 messages per second
                
                async def receive_messages():
                    while time.time() < end_time:
                        try:
                            response = await asyncio.wait_for(websocket.recv(), timeout=1.0)
                            self.messages_received += 1
                            
                            # Calculate latency
                            data = json.loads(response)
                            if "id" in data and data["id"] in send_times:
                                latency = (time.time() - send_times[data["id"]]) * 1000
                                self.latencies.append(latency)
                        except asyncio.TimeoutError:
                            continue
                        except Exception as e:
                            print(f"⚠️  Receive error: {e}")
                            self.errors += 1
                
                # Run send and receive concurrently
                await asyncio.gather(
                    send_messages(),
                    receive_messages(),
                    return_exceptions=True
                )
                
                return True
                
        except Exception as e:
            print(f"❌ Performance test failed: {e}")
            self.errors += 1
            return False
    
    def print_summary(self):
        """Print test summary"""
        print("\n" + "="*50)
        print("📊 WebSocket Test Summary")
        print("="*50)
        print(f"Messages sent:     {self.messages_sent}")
        print(f"Messages received: {self.messages_received}")
        print(f"Errors:           {self.errors}")
        
        if self.latencies:
            avg_latency = sum(self.latencies) / len(self.latencies)
            min_latency = min(self.latencies)
            max_latency = max(self.latencies)
            print(f"\nLatency Statistics:")
            print(f"  Average: {avg_latency:.2f}ms")
            print(f"  Min:     {min_latency:.2f}ms")
            print(f"  Max:     {max_latency:.2f}ms")
        
        if self.start_time and self.messages_sent > 0:
            duration = time.time() - self.start_time
            rate = self.messages_sent / duration
            print(f"\nMessage rate: {rate:.2f} msg/s")
        
        success_rate = (self.messages_received / self.messages_sent * 100) if self.messages_sent > 0 else 0
        print(f"Success rate: {success_rate:.1f}%")
        
        if self.errors == 0 and success_rate > 95:
            print("\n✅ All tests passed!")
            return 0
        else:
            print("\n❌ Some tests failed")
            return 1

async def main():
    """Run all WebSocket tests"""
    print("🧪 HAL9 WebSocket Test Suite")
    print("="*50)
    
    tester = WebSocketTester()
    
    # Run tests
    tests = [
        ("Connection Test", tester.test_connection),
        ("Echo Test", tester.test_echo),
        ("Signal Subscription", tester.test_signal_subscription),
        ("Performance Test", tester.test_performance),
    ]
    
    for test_name, test_func in tests:
        print(f"\n🔄 Running {test_name}...")
        try:
            await test_func()
        except Exception as e:
            print(f"❌ {test_name} crashed: {e}")
            tester.errors += 1
    
    # Print summary
    return tester.print_summary()

if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)