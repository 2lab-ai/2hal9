#!/usr/bin/env node

const WebSocket = require('ws');

// WebSocket server URL
const wsUrl = 'ws://localhost:8080/ws';

console.log('🔌 Connecting to WebSocket server...');
const ws = new WebSocket(wsUrl);

// Connection opened
ws.on('open', function open() {
  console.log('✅ WebSocket connection established');
  
  // Test 1: Send echo message
  console.log('\n📤 Sending echo test...');
  ws.send(JSON.stringify({
    type: 'echo',
    payload: 'Hello HAL9!'
  }));
  
  // Test 2: Send ping
  setTimeout(() => {
    console.log('\n📤 Sending ping...');
    ws.send(JSON.stringify({
      type: 'ping',
      id: 123,
      timestamp: Date.now()
    }));
  }, 500);
  
  // Test 3: Subscribe to a channel
  setTimeout(() => {
    console.log('\n📤 Subscribing to signals channel...');
    ws.send(JSON.stringify({
      type: 'subscribe',
      channel: 'signals',
      filter: null
    }));
  }, 1000);
  
  // Test 4: Send invalid message
  setTimeout(() => {
    console.log('\n📤 Sending invalid message to test error handling...');
    ws.send('invalid json');
  }, 1500);
  
  // Close connection after tests
  setTimeout(() => {
    console.log('\n🔌 Closing connection...');
    ws.close();
  }, 3000);
});

// Listen for messages
ws.on('message', function message(data) {
  console.log('\n📥 Received:', data.toString());
  try {
    const msg = JSON.parse(data.toString());
    console.log('   Type:', msg.type);
    console.log('   Content:', JSON.stringify(msg, null, 2));
  } catch (e) {
    console.log('   Raw message (not JSON)');
  }
});

// Connection closed
ws.on('close', function close() {
  console.log('\n❌ WebSocket connection closed');
  process.exit(0);
});

// Error handling
ws.on('error', function error(err) {
  console.error('\n❌ WebSocket error:', err.message);
  process.exit(1);
});