//! 2HAL9 MVP - Simplified hierarchical AI orchestration demo

mod web;
mod recorder;
mod exporter;

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, Mutex, broadcast};
use tracing::debug;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use recorder::{DemoRecorder, DemoPlayer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub from: String,
    pub to: String,
    pub content: String,
    pub layer: String,
    pub timestamp: DateTime<Utc>,
}

/// Deterministic mock neuron for each layer
pub struct MockNeuron {
    id: String,
    layer: String,
}

impl MockNeuron {
    pub fn new(id: &str, layer: &str) -> Self {
        Self {
            id: id.to_string(),
            layer: layer.to_string(),
        }
    }

    /// Process signal with deterministic responses
    pub async fn process(&self, signal: &Signal) -> Vec<Signal> {
        let start_time = std::time::Instant::now();
        
        // Add visual separation
        println!("\n{}", "─".repeat(80).bright_black());
        
        let icon = match self.layer.as_str() {
            "L4" => "🧠",
            "L3" => "💡", 
            "L2" => "⚙️",
            _ => "📋",
        };
        
        println!("{} {} {} processing", 
            icon,
            self.layer.blue().bold(), 
            format!("[{}]", self.id).cyan()
        );
        println!("  📥 Input: {}", signal.content.white().italic());
        
        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let results = match self.layer.as_str() {
            "L4" => {
                // Strategic layer - break down into design tasks
                println!("  {} Breaking down into strategic components...", "🔍".yellow());
                println!("  {} Decomposing into parallel design tasks", "→".green());
                vec![
                    Signal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-2".to_string(),
                        content: format!("Design architecture for: {}", signal.content),
                        layer: "L3".to_string(),
                        timestamp: Utc::now(),
                    },
                    Signal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-3".to_string(),
                        content: format!("Plan user interface for: {}", signal.content),
                        layer: "L3".to_string(),
                        timestamp: Utc::now(),
                    },
                ]
            }
            "L3" => {
                // Design layer - create 2 implementation tasks per L3 neuron
                println!("  {} Analyzing requirements...", "🔬".blue());
                
                // Each L3 neuron generates 2 L2 tasks based on its focus area
                let implementations = if signal.content.contains("architecture") || signal.content.contains("backend") {
                    println!("  {} Backend system design identified", "→".green());
                    println!("  {} Decomposing into 2 implementation tasks", "→".green());
                    
                    if signal.content.contains("TODO") || signal.content.contains("task") {
                        vec![
                            "Implement database schema with id, title, description, completed, created_at fields",
                            "Implement repository pattern with create, read, update, delete methods"
                        ]
                    } else if signal.content.contains("e-commerce") || signal.content.contains("product") {
                        vec![
                            "Implement product schema with SKU, name, price, inventory fields",
                            "Implement product service with CRUD and inventory tracking"
                        ]
                    } else {
                        vec![
                            "Implement message schema with sender, content, timestamp fields",
                            "Implement message storage with Redis pub/sub for delivery"
                        ]
                    }
                } else if signal.content.contains("API") || signal.content.contains("endpoint") {
                    println!("  {} API design identified", "→".green());
                    println!("  {} Decomposing into 2 implementation tasks", "→".green());
                    
                    if signal.content.contains("TODO") || signal.content.contains("task") {
                        vec![
                            "Implement REST endpoints: POST /todos, GET /todos, PUT /todos/:id, DELETE /todos/:id",
                            "Implement validation middleware and error handling for API requests"
                        ]
                    } else if signal.content.contains("e-commerce") || signal.content.contains("product") {
                        vec![
                            "Implement product listing API with pagination and filters",
                            "Implement product search with full-text and faceted search"
                        ]
                    } else {
                        vec![
                            "Implement WebSocket server for real-time connections",
                            "Implement presence tracking and typing indicators"
                        ]
                    }
                } else {
                    println!("  {} Frontend interface design identified", "→".green());
                    println!("  {} Decomposing into 2 implementation tasks", "→".green());
                    
                    if signal.content.contains("task") {
                        vec![
                            "Implement React components for task list, task item, and task form",
                            "Implement state management with Context API for task operations"
                        ]
                    } else if signal.content.contains("e-commerce") {
                        vec![
                            "Implement product grid component with lazy loading",
                            "Implement shopping cart with local storage persistence"
                        ]
                    } else {
                        vec![
                            "Implement chat message list with virtual scrolling",
                            "Implement message input with emoji and file support"
                        ]
                    }
                };
                
                // Generate 2 L2 signals
                implementations.iter().map(|&implementation| {
                    Signal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-4".to_string(),
                        content: implementation.to_string(),
                        layer: "L2".to_string(),
                        timestamp: Utc::now(),
                    }
                }).collect()
            }
            "L2" => {
                // Implementation layer - generate code
                println!("  {} Generating implementation code...", "🔧".green());
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                
                println!("\n{}", "╔══════════════════════ Generated Code ══════════════════════╗".bright_cyan());
                
                if signal.content.contains("real-time") || signal.content.contains("chat") {
                    println!("{}", "║ 💬 Real-time Chat Implementation (WebSocket + Redis)       ║".bright_cyan());
                    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
                    println!("{}", r#"
// websocket-server.js
const WebSocket = require('ws');
const Redis = require('ioredis');
const jwt = require('jsonwebtoken');

const wss = new WebSocket.Server({ port: 8080 });
const redis = new Redis();
const pubClient = new Redis();
const subClient = new Redis();

// Track connected users
const connections = new Map();

// Subscribe to Redis channels for scalability
subClient.subscribe('chat:messages', 'chat:presence');

subClient.on('message', (channel, message) => {
  const data = JSON.parse(message);
  
  // Broadcast to all connected clients in the room
  if (channel === 'chat:messages') {
    broadcastToRoom(data.roomId, {
      type: 'message',
      ...data
    });
  }
});

wss.on('connection', async (ws, req) => {
  const token = new URL(req.url, `http://${req.headers.host}`)
    .searchParams.get('token');
  
  try {
    const user = jwt.verify(token, process.env.JWT_SECRET);
    const connectionId = crypto.randomUUID();
    
    connections.set(connectionId, {
      ws,
      userId: user.id,
      rooms: new Set()
    });
    
    ws.on('message', async (data) => {
      const message = JSON.parse(data);
      
      switch (message.type) {
        case 'join':
          await handleJoinRoom(connectionId, message.roomId);
          break;
          
        case 'message':
          await handleMessage(connectionId, message);
          break;
          
        case 'typing':
          await handleTypingIndicator(connectionId, message);
          break;
      }
    });
    
    ws.on('close', () => {
      handleDisconnect(connectionId);
    });
    
  } catch (error) {
    ws.close(1008, 'Invalid token');
  }
});

async function handleMessage(connectionId, message) {
  const conn = connections.get(connectionId);
  
  const chatMessage = {
    id: crypto.randomUUID(),
    userId: conn.userId,
    roomId: message.roomId,
    content: message.content,
    timestamp: new Date().toISOString()
  };
  
  // Store in database
  await redis.zadd(
    `room:${message.roomId}:messages`,
    Date.now(),
    JSON.stringify(chatMessage)
  );
  
  // Publish for other servers
  pubClient.publish('chat:messages', JSON.stringify(chatMessage));
}

// client.tsx
import { useEffect, useRef, useState } from 'react';

interface Message {
  id: string;
  userId: string;
  content: string;
  timestamp: string;
}

export function ChatRoom({ roomId, token }: { roomId: string; token: string }) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [connected, setConnected] = useState(false);
  const ws = useRef<WebSocket | null>(null);
  
  useEffect(() => {
    ws.current = new WebSocket(`ws://localhost:8080?token=${token}`);
    
    ws.current.onopen = () => {
      setConnected(true);
      ws.current?.send(JSON.stringify({ type: 'join', roomId }));
    };
    
    ws.current.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'message') {
        setMessages(prev => [...prev, data]);
      }
    };
    
    return () => {
      ws.current?.close();
    };
  }, [roomId, token]);
  
  const sendMessage = (content: string) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify({
        type: 'message',
        roomId,
        content
      }));
    }
  };
  
  return (
    <div className="flex flex-col h-screen">
      <MessageList messages={messages} />
      <MessageInput onSend={sendMessage} disabled={!connected} />
    </div>
  );
}
"#.bright_white());
                } else if signal.content.contains("e-commerce") || signal.content.contains("cart") {
                    println!("{}", "║ 🛒 E-commerce Implementation (Next.js + Stripe)            ║".bright_cyan());
                    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
                    println!("{}", r#"
// pages/api/checkout.ts
import { NextApiRequest, NextApiResponse } from 'next';
import Stripe from 'stripe';
import { prisma } from '@/lib/prisma';
import { getServerSession } from 'next-auth';
import { authOptions } from './auth/[...nextauth]';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const session = await getServerSession(req, res, authOptions);
  if (!session) {
    return res.status(401).json({ error: 'Unauthorized' });
  }

  try {
    const { items } = req.body;
    
    // Validate cart items and calculate total
    const lineItems = await Promise.all(
      items.map(async (item: any) => {
        const product = await prisma.product.findUnique({
          where: { id: item.productId }
        });
        
        if (!product) {
          throw new Error(`Product ${item.productId} not found`);
        }
        
        return {
          price_data: {
            currency: 'usd',
            product_data: {
              name: product.name,
              images: [product.imageUrl],
              metadata: { productId: product.id }
            },
            unit_amount: Math.round(product.price * 100)
          },
          quantity: item.quantity
        };
      })
    );

    // Create order record
    const order = await prisma.order.create({
      data: {
        userId: session.user.id,
        status: 'pending',
        items: {
          create: items.map((item: any) => ({
            productId: item.productId,
            quantity: item.quantity,
            price: item.price
          }))
        }
      }
    });

    // Create Stripe checkout session
    const checkoutSession = await stripe.checkout.sessions.create({
      payment_method_types: ['card'],
      line_items: lineItems,
      mode: 'payment',
      success_url: `${process.env.NEXT_PUBLIC_URL}/order/success?session_id={CHECKOUT_SESSION_ID}`,
      cancel_url: `${process.env.NEXT_PUBLIC_URL}/cart`,
      metadata: { orderId: order.id }
    });

    res.json({ url: checkoutSession.url });
  } catch (error) {
    console.error('Checkout error:', error);
    res.status(500).json({ error: 'Failed to create checkout session' });
  }
}

// components/ProductGrid.tsx
import { useState, useEffect } from 'react';
import { Product } from '@/types';
import { useCart } from '@/hooks/useCart';

export function ProductGrid() {
  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);
  const { addToCart } = useCart();

  useEffect(() => {
    fetchProducts();
  }, []);

  const fetchProducts = async () => {
    try {
      const res = await fetch('/api/products');
      const data = await res.json();
      setProducts(data);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <ProductSkeleton count={8} />;
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      {products.map((product) => (
        <div key={product.id} className="group relative">
          <div className="aspect-h-1 aspect-w-1 w-full overflow-hidden rounded-lg bg-gray-200">
            <img
              src={product.imageUrl}
              alt={product.name}
              className="h-full w-full object-cover object-center group-hover:opacity-75"
            />
          </div>
          <div className="mt-4 flex justify-between">
            <div>
              <h3 className="text-sm text-gray-700">{product.name}</h3>
              <p className="mt-1 text-sm text-gray-500">{product.category}</p>
            </div>
            <p className="text-sm font-medium text-gray-900">
              ${product.price.toFixed(2)}
            </p>
          </div>
          <button
            onClick={() => addToCart(product)}
            className="mt-4 w-full bg-indigo-600 text-white py-2 px-4 rounded hover:bg-indigo-700"
          >
            Add to Cart
          </button>
        </div>
      ))}
    </div>
  );
}
"#.bright_white());
                } else if signal.content.contains("backend") {
                    println!("{}", "║ 📦 Backend Implementation (Node.js + Express)              ║".bright_cyan());
                    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
                    println!("{}", r#"
// server.js
const express = require('express');
const jwt = require('jsonwebtoken');
const bcrypt = require('bcrypt');
const { PrismaClient } = require('@prisma/client');

const app = express();
const prisma = new PrismaClient();

app.use(express.json());

// Authentication middleware
const authenticate = async (req, res, next) => {
  const token = req.headers.authorization?.split(' ')[1];
  if (!token) return res.status(401).json({ error: 'Unauthorized' });
  
  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET);
    req.user = await prisma.user.findUnique({ where: { id: decoded.id } });
    next();
  } catch (error) {
    res.status(401).json({ error: 'Invalid token' });
  }
};

// User authentication endpoint
app.post('/api/auth/login', async (req, res) => {
  const { email, password } = req.body;
  
  const user = await prisma.user.findUnique({ where: { email } });
  if (!user || !await bcrypt.compare(password, user.hashedPassword)) {
    return res.status(401).json({ error: 'Invalid credentials' });
  }
  
  const token = jwt.sign({ id: user.id }, process.env.JWT_SECRET);
  res.json({ token, user: { id: user.id, email: user.email } });
});

// Task management endpoints
app.get('/api/tasks', authenticate, async (req, res) => {
  const tasks = await prisma.task.findMany({
    where: { userId: req.user.id },
    orderBy: { createdAt: 'desc' }
  });
  res.json(tasks);
});

app.post('/api/tasks', authenticate, async (req, res) => {
  const task = await prisma.task.create({
    data: { ...req.body, userId: req.user.id }
  });
  res.json(task);
});
"#.bright_white());
                } else {
                    println!("{}", "║ 🎨 Frontend Implementation (React + TypeScript)            ║".bright_cyan());
                    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
                    println!("{}", r#"
// App.tsx
import React, { useState, useEffect } from 'react';
import { TaskList } from './components/TaskList';
import { TaskForm } from './components/TaskForm';
import { useAuth } from './hooks/useAuth';
import { Task } from './types';

export function App() {
  const { user, login, logout } = useAuth();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (user) {
      fetchTasks();
    }
  }, [user]);

  const fetchTasks = async () => {
    try {
      const response = await fetch('/api/tasks', {
        headers: {
          'Authorization': `Bearer ${user.token}`
        }
      });
      const data = await response.json();
      setTasks(data);
    } finally {
      setLoading(false);
    }
  };

  const addTask = async (task: Omit<Task, 'id'>) => {
    const response = await fetch('/api/tasks', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${user.token}`
      },
      body: JSON.stringify(task)
    });
    const newTask = await response.json();
    setTasks([newTask, ...tasks]);
  };

  if (!user) {
    return <LoginForm onLogin={login} />;
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 py-6 flex justify-between">
          <h1 className="text-3xl font-bold text-gray-900">Task Manager</h1>
          <button onClick={logout} className="text-gray-500 hover:text-gray-700">
            Logout
          </button>
        </div>
      </header>
      
      <main className="max-w-7xl mx-auto px-4 py-8">
        <TaskForm onSubmit={addTask} />
        {loading ? (
          <div className="text-center py-8">Loading...</div>
        ) : (
          <TaskList tasks={tasks} />
        )}
      </main>
    </div>
  );
}
"#.bright_white());
                }
                
                println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
                vec![] // L2 doesn't forward signals
            }
            _ => vec![],
        };
        
        let elapsed = start_time.elapsed();
        println!("  ⏱️  Completed in {:.2}s", elapsed.as_secs_f64());
        
        results
    }
}

/// Tracks signal flow for visualization
#[derive(Clone)]
struct SignalTracker {
    signals: Arc<Mutex<Vec<Signal>>>,
}

impl SignalTracker {
    fn new() -> Self {
        Self {
            signals: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn record(&self, signal: Signal) {
        let mut signals = self.signals.lock().await;
        signals.push(signal);
    }
    
    async fn visualize_hierarchy(&self) {
        let signals = self.signals.lock().await;
        
        // Calculate total elapsed time
        let first_time = signals.first().map(|s| s.timestamp).unwrap_or_else(Utc::now);
        let last_time = signals.last().map(|s| s.timestamp).unwrap_or_else(Utc::now);
        let total_elapsed = (last_time - first_time).num_milliseconds() as f64 / 1000.0;
        
        println!("\n{}", "━".repeat(80).bright_black());
        println!("{}", "📊 SIGNAL FLOW HIERARCHY".cyan().bold());
        println!("   Total processing time: {:.2}s", total_elapsed);
        println!("{}", "━".repeat(80).bright_black());
        
        // Build hierarchy map
        let mut hierarchy: HashMap<Option<Uuid>, Vec<&Signal>> = HashMap::new();
        for signal in signals.iter() {
            hierarchy.entry(signal.parent_id).or_insert_with(Vec::new).push(signal);
        }
        
        // Find root signals
        if let Some(roots) = hierarchy.get(&None) {
            for root in roots {
                self.print_signal_tree(root, &hierarchy, 0);
            }
        }
        
        println!("{}\n", "━".repeat(80).bright_black());
    }
    
    fn print_signal_tree(&self, signal: &Signal, hierarchy: &HashMap<Option<Uuid>, Vec<&Signal>>, depth: usize) {
        let indent = "  ".repeat(depth);
        let arrow = if depth > 0 { "└─" } else { "🎯" };
        
        let layer_color = match signal.layer.as_str() {
            "Input" => "yellow",
            "L4" => "magenta",
            "L3" => "blue",
            "L2" => "green",
            _ => "white",
        };
        
        let timestamp = signal.timestamp.format("%H:%M:%S%.3f");
        
        println!("{}{} {} {} → {} [{}]",
            indent.bright_black(),
            arrow.bright_black(),
            signal.layer.color(layer_color).bold(),
            signal.from.cyan(),
            signal.to.cyan(),
            timestamp.to_string().bright_black()
        );
        
        println!("{}   {}", 
            indent.bright_black(),
            signal.content.white()
        );
        
        // Recursively print children
        if let Some(children) = hierarchy.get(&Some(signal.id)) {
            for child in children {
                self.print_signal_tree(child, hierarchy, depth + 1);
            }
        }
    }
}

/// Simple 3-neuron orchestrator
pub struct Orchestrator {
    neurons: Vec<Arc<MockNeuron>>,
    signal_tx: mpsc::Sender<Signal>,
    tracker: SignalTracker,
    broadcast_tx: broadcast::Sender<Signal>,
    recorder: Arc<DemoRecorder>,
}

impl Orchestrator {
    fn new() -> (Self, mpsc::Receiver<Signal>) {
        let (signal_tx, signal_rx) = mpsc::channel(100);
        let (broadcast_tx, _) = broadcast::channel(100);
        
        let neurons = vec![
            Arc::new(MockNeuron::new("neuron-1", "L4")),
            Arc::new(MockNeuron::new("neuron-2", "L3")),
            Arc::new(MockNeuron::new("neuron-3", "L3")),
            Arc::new(MockNeuron::new("neuron-4", "L2")),
        ];
        
        let tracker = SignalTracker::new();
        let recorder = Arc::new(DemoRecorder::new());
        
        (Self { neurons, signal_tx, tracker, broadcast_tx, recorder }, signal_rx)
    }
    
    async fn start(&self, mut signal_rx: mpsc::Receiver<Signal>) {
        println!("{}", "🧠 2HAL9 Orchestrator Started".green().bold());
        println!("Neurons: L4→L3→L2 pipeline ready\n");
        
        while let Some(signal) = signal_rx.recv().await {
            debug!("Processing signal: {:?}", signal);
            
            // Record signal for visualization
            self.tracker.record(signal.clone()).await;
            
            // Broadcast signal for web UI
            let _ = self.broadcast_tx.send(signal.clone());
            
            // Record for demo replay if active
            let _ = self.recorder.record_signal(signal.clone()).await;
            
            // Find target neuron
            if let Some(neuron) = self.neurons.iter().find(|n| n.id == signal.to) {
                let neuron = neuron.clone();
                let tx = self.signal_tx.clone();
                
                // Process in separate task
                let recorder = self.recorder.clone();
                let neuron_id_clone = neuron.id.clone();
                let neuron_layer = neuron.layer.clone();
                tokio::spawn(async move {
                    // Record neuron activation
                    let _ = recorder.record_neuron_activation(neuron_id_clone, neuron_layer).await;
                    
                    let outputs = neuron.process(&signal).await;
                    for output in outputs {
                        let _ = tx.send(output).await;
                    }
                });
            }
        }
    }
    
    pub async fn send_signal(&self, signal: Signal) -> Result<()> {
        self.signal_tx.send(signal).await?;
        Ok(())
    }
    
    async fn visualize_flow(&self) {
        self.tracker.visualize_hierarchy().await;
    }
    
    async fn clear_tracker(&self) {
        let mut signals = self.tracker.signals.lock().await;
        signals.clear();
    }
    
    pub async fn get_signals(&self) -> Vec<Signal> {
        let signals = self.tracker.signals.lock().await;
        signals.clone()
    }
    
    pub fn subscribe_to_signals(&self) -> broadcast::Receiver<Signal> {
        self.broadcast_tx.subscribe()
    }
    
    pub async fn start_recording(&self, scenario: String) -> Result<Uuid> {
        self.recorder.start_recording(scenario).await
    }
    
    pub async fn stop_recording(&self) -> Result<recorder::DemoRecording> {
        self.recorder.stop_recording().await
    }
    
    pub fn get_recorder(&self) -> Arc<DemoRecorder> {
        self.recorder.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();
    
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    let web_mode = args.contains(&"--web".to_string());
    let record_mode = args.contains(&"--record".to_string());
    let replay_mode = args.iter().any(|a| a.starts_with("--replay="));
    let export_svg_mode = args.iter().any(|a| a.starts_with("--export-svg="));
    let export_frames_mode = args.iter().any(|a| a.starts_with("--export-frames="));
    let export_gif_script_mode = args.iter().any(|a| a.starts_with("--export-gif-script="));
    
    // Handle export modes
    if export_svg_mode {
        if let Some(export_arg) = args.iter().find(|a| a.starts_with("--export-svg=")) {
            let path = export_arg.strip_prefix("--export-svg=").unwrap();
            return export_to_svg(path).await;
        }
    }
    
    if export_frames_mode {
        if let Some(export_arg) = args.iter().find(|a| a.starts_with("--export-frames=")) {
            let path = export_arg.strip_prefix("--export-frames=").unwrap();
            return export_to_frames(path).await;
        }
    }
    
    if export_gif_script_mode {
        if let Some(export_arg) = args.iter().find(|a| a.starts_with("--export-gif-script=")) {
            let path = export_arg.strip_prefix("--export-gif-script=").unwrap();
            return export_gif_script(path).await;
        }
    }
    
    // Handle replay mode
    if replay_mode {
        if let Some(replay_arg) = args.iter().find(|a| a.starts_with("--replay=")) {
            let path = replay_arg.strip_prefix("--replay=").unwrap();
            return replay_demo(path).await;
        }
    }
    
    // Print enhanced banner
    println!("{}", r#"
╔═══════════════════════════════════════════════════════════════════════╗
║          ____  _   _    _    _     ___                                ║
║         |___ \| | | |  / \  | |   / _ \                               ║
║           __) | |_| | / _ \ | |  | (_) |                              ║
║          / __/|  _  |/ ___ \| |___\__, |                              ║
║         |_____|_| |_/_/   \_\_____|  /_/                              ║
║                                                                       ║
║              HIERARCHICAL AI LAYER ORCHESTRATION                      ║
║                  Simplified MVP Demonstration                         ║
╚═══════════════════════════════════════════════════════════════════════╝"#.cyan().bold());
    
    println!("\n{}", "🧠 Strategic (L4) → 💡 Design (L3) → ⚙️ Implementation (L2)".white());
    
    // Create orchestrator
    let (orchestrator, signal_rx) = Orchestrator::new();
    let orchestrator = Arc::new(orchestrator);
    
    // Start processing in background
    let orch_clone = orchestrator.clone();
    tokio::spawn(async move {
        orch_clone.start(signal_rx).await;
    });
    
    // Start web server if requested
    if web_mode {
        println!("{}", "Starting in web mode...".cyan());
        let web_server = Arc::new(web::WebServer::new(orchestrator.clone()));
        tokio::spawn(async move {
            if let Err(e) = web_server.start().await {
                eprintln!("Web server error: {}", e);
            }
        });
        
        // Keep running
        tokio::signal::ctrl_c().await?;
        println!("\nShutting down...");
        return Ok(());
    }
    
    // Demo scenarios
    let scenarios = vec![
        ("Create a task management web application", "📝"),
        ("Build an e-commerce platform", "🛒"),
        ("Develop a real-time chat system", "💬"),
    ];
    
    println!("\n{}", "╔═══════════════════════ Demo Scenarios ════════════════════════╗".yellow().bold());
    println!("{}", "║                                                                ║".yellow());
    for (i, (scenario, icon)) in scenarios.iter().enumerate() {
        println!("{}  {} {} {}",
            "║".yellow(),
            format!("[{}]", i + 1).cyan().bold(),
            icon,
            scenario.white()
        );
    }
    println!("{}", "║                                                                ║".yellow());
    println!("{}", "║  [q] Exit                                                      ║".yellow());
    println!("{}", "╚════════════════════════════════════════════════════════════════╝".yellow().bold());
    
    // Simple input loop
    loop {
        print!("\n{} ", ">".green().bold());
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input == "q" {
            println!("{}", "Shutting down...".yellow());
            break;
        }
        
        let (scenario, icon) = match input {
            "1" => scenarios[0],
            "2" => scenarios[1],
            "3" => scenarios[2],
            _ => {
                println!("{}", "❌ Invalid selection. Please choose 1-3 or 'q' to exit.".red());
                continue;
            }
        };
        
        println!("\n{} {} {}", icon, "Selected:".green().bold(), scenario.white());
        
        // Start recording if requested
        let recording_id = if record_mode {
            println!("{}", "🔴 Recording demo...".red().bold());
            Some(orchestrator.start_recording(scenario.to_string()).await?)
        } else {
            None
        };
        
        // Send initial signal to L4
        let signal = Signal {
            id: Uuid::new_v4(),
            parent_id: None,
            from: "user".to_string(),
            to: "neuron-1".to_string(),
            content: scenario.to_string(),
            layer: "Input".to_string(),
            timestamp: Utc::now(),
        };
        
        // Clear previous signals
        orchestrator.clear_tracker().await;
        
        orchestrator.send_signal(signal).await?;
        
        // Show processing animation
        print!("\n{} Processing", "⏳".yellow());
        for i in 0..12 {
            print!(".");
            io::stdout().flush()?;
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            
            // Show layer progression
            if i == 3 {
                print!(" [{}]", "L4 Strategic".magenta());
            } else if i == 6 {
                print!(" [{}]", "L3 Design".blue());
            } else if i == 9 {
                print!(" [{}]", "L2 Implementation".green());
            }
            io::stdout().flush()?;
        }
        println!();
        
        // Show visual hierarchy
        orchestrator.visualize_flow().await;
        
        println!("\n{} {}", "✅".green(), "Processing complete! All layers executed successfully.".green().bold());
        
        // Stop and save recording if active
        if let Some(_id) = recording_id {
            let recording = orchestrator.stop_recording().await?;
            let recordings_dir = std::path::Path::new("mvp/recordings");
            let path = orchestrator.get_recorder().save_recording(&recording, recordings_dir).await?;
            println!("\n{} Recording saved to: {}", "💾".yellow(), path.display().to_string().cyan());
        }
    }
    
    Ok(())
}

/// Replay a recorded demo
async fn replay_demo(path: &str) -> Result<()> {
    println!("{}", r#"
╔═══════════════════════════════════════════════════════════════════════╗
║                    2HAL9 Demo Replay Mode                              ║
╚═══════════════════════════════════════════════════════════════════════╝"#.cyan().bold());
    
    let path = std::path::Path::new(path);
    let recording = DemoRecorder::load_recording(path).await?;
    
    let mut player = DemoPlayer::new();
    
    // Allow speed selection
    println!("\n{}", "Select playback speed:".yellow());
    println!("  {} Normal speed", "[1]".cyan());
    println!("  {} Double speed", "[2]".cyan());
    println!("  {} Half speed", "[0.5]".cyan());
    
    print!("\n{} ", ">".green().bold());
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let speed: f32 = input.trim().parse().unwrap_or(1.0);
    player.set_speed(speed);
    
    // Play the recording
    player.play(&recording, |event| {
        match &event.event_type {
            recorder::EventType::Signal { signal } => {
                println!("\n{}", "─".repeat(80).bright_black());
                println!("{} {} → {} [{}ms]", 
                    signal.layer.color(match signal.layer.as_str() {
                        "L4" => "magenta",
                        "L3" => "blue",
                        "L2" => "green",
                        _ => "yellow",
                    }).bold(),
                    signal.from.cyan(),
                    signal.to.cyan(),
                    event.timestamp_ms
                );
                println!("  {}", signal.content.white());
            }
            recorder::EventType::NeuronActivation { neuron_id, layer } => {
                println!("  {} {} activated", "⚡".yellow(), neuron_id.cyan());
            }
            recorder::EventType::CodeGenerated { layer, content } => {
                println!("\n{}", "╔══════════════════════ Generated Code ══════════════════════╗".bright_cyan());
                println!("{}", content.bright_white());
                println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
            }
            recorder::EventType::StatusUpdate { message } => {
                println!("  {} {}", "ℹ️".blue(), message.white());
            }
        }
        Ok(())
    }).await?;
    
    Ok(())
}

/// Export recording to animated SVG
async fn export_to_svg(recording_path: &str) -> Result<()> {
    use exporter::DemoExporter;
    
    println!("{}", r#"
╔═══════════════════════════════════════════════════════════════════════╗
║                    2HAL9 Export to SVG                                 ║
╚═══════════════════════════════════════════════════════════════════════╝"#.cyan().bold());
    
    let recording_path = std::path::Path::new(recording_path);
    let recording = DemoRecorder::load_recording(recording_path).await?;
    
    let output_path = recording_path.with_extension("svg");
    
    println!("📼 Loading recording: {}", recording_path.display());
    println!("🎬 Scenario: {}", recording.scenario);
    println!("⏱️  Duration: {:.1}s", recording.duration_ms as f32 / 1000.0);
    println!("📊 Events: {}", recording.events.len());
    
    println!("\n🎨 Exporting to SVG...");
    DemoExporter::export_as_svg(&recording, &output_path).await?;
    
    println!("✅ Export complete: {}", output_path.display().to_string().green());
    println!("💡 Open in browser to view animation");
    
    Ok(())
}

/// Export recording to frame sequence
async fn export_to_frames(recording_path: &str) -> Result<()> {
    use exporter::DemoExporter;
    
    println!("{}", r#"
╔═══════════════════════════════════════════════════════════════════════╗
║                    2HAL9 Export to Frames                              ║
╚═══════════════════════════════════════════════════════════════════════╝"#.cyan().bold());
    
    let recording_path = std::path::Path::new(recording_path);
    let recording = DemoRecorder::load_recording(recording_path).await?;
    
    let output_dir = recording_path.parent().unwrap_or(std::path::Path::new("."))
        .join(format!("{}_frames", recording_path.file_stem().unwrap().to_str().unwrap()));
    
    println!("📼 Loading recording: {}", recording_path.display());
    println!("🎬 Scenario: {}", recording.scenario);
    println!("⏱️  Duration: {:.1}s", recording.duration_ms as f32 / 1000.0);
    
    print!("\n🎞️  FPS (default 30): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let fps: u32 = input.trim().parse().unwrap_or(30);
    
    println!("\n🖼️  Exporting frames at {} FPS...", fps);
    let frame_paths = DemoExporter::export_frames(&recording, &output_dir, fps).await?;
    
    println!("✅ Exported {} frames to: {}", frame_paths.len(), output_dir.display().to_string().green());
    println!("\n💡 Convert to GIF with:");
    println!("   convert -delay {} -loop 0 {}/*.svg demo.gif", 100/fps, output_dir.display());
    
    Ok(())
}

/// Export GIF conversion script
async fn export_gif_script(recording_path: &str) -> Result<()> {
    use exporter::DemoExporter;
    
    println!("{}", r#"
╔═══════════════════════════════════════════════════════════════════════╗
║                    2HAL9 Export GIF Script                             ║
╚═══════════════════════════════════════════════════════════════════════╝"#.cyan().bold());
    
    let recording_path = std::path::Path::new(recording_path);
    let recording = DemoRecorder::load_recording(recording_path).await?;
    
    let script_path = recording_path.with_extension("gif.sh");
    
    println!("📼 Loading recording: {}", recording_path.display());
    println!("🎬 Scenario: {}", recording.scenario);
    
    println!("\n📝 Creating GIF conversion script...");
    DemoExporter::export_gif_script(&recording, &script_path).await?;
    
    println!("✅ Script created: {}", script_path.display().to_string().green());
    println!("\n💡 Run with:");
    println!("   bash {} {}", script_path.display(), recording_path.display());
    
    Ok(())
}