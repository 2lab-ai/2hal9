//! Stream protocol for continuous data flows

use async_trait::async_trait;
use tokio::sync::mpsc;
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use uuid::Uuid;
use crate::{Result, Error};

/// Stream protocol for handling continuous data flows
#[async_trait]
pub trait StreamProtocol: Send + Sync {
    /// Create a new stream
    async fn create_stream(&self, config: StreamConfig) -> Result<Box<dyn DataStream>>;
    
    /// Accept an incoming stream
    async fn accept_stream(&self, stream_id: Uuid) -> Result<Box<dyn DataStream>>;
    
    /// Get stream metrics
    async fn stream_metrics(&self, stream_id: Uuid) -> Result<StreamMetrics>;
}

/// Configuration for a data stream
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub stream_id: Uuid,
    pub buffer_size: usize,
    pub backpressure: BackpressureStrategy,
    pub ordering: StreamOrdering,
    pub reliability: StreamReliability,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            stream_id: Uuid::new_v4(),
            buffer_size: 1024,
            backpressure: BackpressureStrategy::Buffer,
            ordering: StreamOrdering::Ordered,
            reliability: StreamReliability::AtLeastOnce,
        }
    }
}

/// Backpressure handling strategy
#[derive(Debug, Clone, Copy)]
pub enum BackpressureStrategy {
    /// Buffer messages up to limit
    Buffer,
    /// Drop newest messages
    DropNewest,
    /// Drop oldest messages
    DropOldest,
    /// Block sender
    Block,
}

/// Stream ordering guarantees
#[derive(Debug, Clone, Copy)]
pub enum StreamOrdering {
    /// Messages delivered in order
    Ordered,
    /// Messages may be delivered out of order
    Unordered,
}

/// Stream reliability guarantees
#[derive(Debug, Clone, Copy)]
pub enum StreamReliability {
    /// Best effort delivery
    BestEffort,
    /// At least once delivery
    AtLeastOnce,
    /// Exactly once delivery
    ExactlyOnce,
}

/// Data stream abstraction
#[async_trait]
pub trait DataStream: Send + Sync {
    /// Get stream ID
    fn id(&self) -> Uuid;
    
    /// Send data to the stream
    async fn send(&mut self, data: Vec<u8>) -> Result<()>;
    
    /// Receive data from the stream
    async fn recv(&mut self) -> Result<Option<Vec<u8>>>;
    
    /// Close the stream
    async fn close(self: Box<Self>) -> Result<()>;
    
    /// Check if stream is closed
    fn is_closed(&self) -> bool;
}

/// Stream metrics
#[derive(Debug, Clone)]
pub struct StreamMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub errors: u64,
    pub backpressure_events: u64,
}

/// Channel-based stream implementation
pub struct ChannelStream {
    id: Uuid,
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    closed: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ChannelStream {
    pub fn new(buffer_size: usize) -> (Self, Self) {
        let id = Uuid::new_v4();
        let (tx1, rx1) = mpsc::channel(buffer_size);
        let (tx2, rx2) = mpsc::channel(buffer_size);
        let closed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        let stream1 = Self {
            id,
            sender: tx1,
            receiver: rx2,
            closed: closed.clone(),
        };
        
        let stream2 = Self {
            id,
            sender: tx2,
            receiver: rx1,
            closed: closed.clone(),
        };
        
        (stream1, stream2)
    }
}

#[async_trait]
impl DataStream for ChannelStream {
    fn id(&self) -> Uuid {
        self.id
    }
    
    async fn send(&mut self, data: Vec<u8>) -> Result<()> {
        if self.is_closed() {
            return Err(Error::Protocol("Stream is closed".to_string()));
        }
        
        self.sender.send(data).await
            .map_err(|_| Error::Protocol("Failed to send on stream".to_string()))
    }
    
    async fn recv(&mut self) -> Result<Option<Vec<u8>>> {
        Ok(self.receiver.recv().await)
    }
    
    async fn close(self: Box<Self>) -> Result<()> {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
    
    fn is_closed(&self) -> bool {
        self.closed.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Stream multiplexer for handling multiple streams over a single connection
pub struct StreamMultiplexer {
    streams: dashmap::DashMap<Uuid, Box<dyn DataStream>>,
    incoming: mpsc::Receiver<(Uuid, Vec<u8>)>,
    outgoing: mpsc::Sender<(Uuid, Vec<u8>)>,
}

impl StreamMultiplexer {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self {
            streams: dashmap::DashMap::new(),
            incoming: rx,
            outgoing: tx,
        }
    }
    
    /// Create a new multiplexed stream
    pub async fn create_stream(&self, config: StreamConfig) -> Result<Uuid> {
        let (local, remote) = ChannelStream::new(config.buffer_size);
        let stream_id = local.id();
        
        self.streams.insert(stream_id, Box::new(local));
        Ok(stream_id)
    }
    
    /// Route incoming data to appropriate stream
    pub async fn route_incoming(&mut self) {
        while let Some((stream_id, data)) = self.incoming.recv().await {
            if let Some(mut stream) = self.streams.get_mut(&stream_id) {
                let _ = stream.send(data).await;
            }
        }
    }
}