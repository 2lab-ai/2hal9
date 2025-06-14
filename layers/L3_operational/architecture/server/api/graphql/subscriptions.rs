use async_graphql::{Context, ID};
use chrono::{DateTime, Utc};
use futures_util::Stream;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use super::schema::*;
use crate::{
    auth::User,
    error::HAL9Error,
};

// ============ Event Types ============

#[derive(Clone, Debug)]
pub enum SystemEvent {
    SignalUpdate(SignalUpdateEvent),
    NeuronStateChange(NeuronStateChangeEvent),
    MetricsUpdate(MetricsUpdateEvent),
    LearningEvent(LearningEventData),
}

#[derive(Clone, Debug)]
pub struct SignalUpdateEvent {
    pub signal_id: Uuid,
    pub status: String,
    pub progress: Option<f32>,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NeuronStateChangeEvent {
    pub neuron_id: Uuid,
    pub previous_state: String,
    pub new_state: String,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct MetricsUpdateEvent {
    pub metric_type: String,
    pub value: f64,
    pub labels: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct LearningEventData {
    pub event_type: String,
    pub pattern_id: Option<Uuid>,
    pub confidence_delta: f64,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

// ============ Event Bus ============

pub struct EventBus {
    sender: broadcast::Sender<SystemEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
    
    pub fn publish(&self, event: SystemEvent) {
        let _ = self.sender.send(event);
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.sender.subscribe()
    }
}

// ============ Subscription Implementations ============

impl SubscriptionRoot {
    pub async fn resolve_signal_updates(
        &self,
        ctx: &Context<'_>,
        signal_id: Option<ID>,
    ) -> impl Stream<Item = SignalUpdate> {
        let event_bus = ctx.data::<Arc<EventBus>>().unwrap();
        let _user = ctx.data::<Arc<User>>().unwrap();
        let filter_id = signal_id.and_then(|id| Uuid::parse_str(&id.0).ok());
        
        BroadcastStream::new(event_bus.subscribe())
            .filter_map(move |event| {
                match event {
                    Ok(SystemEvent::SignalUpdate(update)) => {
                        // Filter by signal ID if provided
                        if let Some(id) = filter_id {
                            if update.signal_id != id {
                                return None;
                            }
                        }
                        
                        Some(SignalUpdate {
                            signal_id: ID(update.signal_id.to_string()),
                            status: update.status,
                            progress: update.progress,
                            message: update.message,
                            timestamp: update.timestamp,
                        })
                    }
                    _ => None,
                }
            })
    }
    
    pub async fn resolve_neuron_state_changes(
        &self,
        ctx: &Context<'_>,
        neuron_id: Option<ID>,
        _layer: Option<String>,
    ) -> impl Stream<Item = NeuronStateChange> {
        let event_bus = ctx.data::<Arc<EventBus>>().unwrap();
        let filter_id = neuron_id.and_then(|id| Uuid::parse_str(&id.0).ok());
        
        BroadcastStream::new(event_bus.subscribe())
            .filter_map(move |event| {
                match event {
                    Ok(SystemEvent::NeuronStateChange(change)) => {
                        // Filter by neuron ID if provided
                        if let Some(id) = filter_id {
                            if change.neuron_id != id {
                                return None;
                            }
                        }
                        
                        // TODO: Filter by layer if provided
                        
                        Some(NeuronStateChange {
                            neuron_id: ID(change.neuron_id.to_string()),
                            previous_state: change.previous_state,
                            new_state: change.new_state,
                            reason: change.reason,
                            timestamp: change.timestamp,
                        })
                    }
                    _ => None,
                }
            })
    }
    
    pub async fn resolve_metrics_updates(
        &self,
        ctx: &Context<'_>,
        metric_type: Option<String>,
    ) -> impl Stream<Item = MetricsUpdate> {
        let event_bus = ctx.data::<Arc<EventBus>>().unwrap();
        
        BroadcastStream::new(event_bus.subscribe())
            .filter_map(move |event| {
                match event {
                    Ok(SystemEvent::MetricsUpdate(update)) => {
                        // Filter by metric type if provided
                        if let Some(ref filter_type) = metric_type {
                            if &update.metric_type != filter_type {
                                return None;
                            }
                        }
                        
                        Some(MetricsUpdate {
                            metric_type: update.metric_type,
                            value: update.value,
                            labels: update.labels,
                            timestamp: update.timestamp,
                        })
                    }
                    _ => None,
                }
            })
    }
    
    pub async fn resolve_learning_events(
        &self,
        ctx: &Context<'_>,
        _layer: Option<String>,
    ) -> impl Stream<Item = LearningEvent> {
        let event_bus = ctx.data::<Arc<EventBus>>().unwrap();
        
        BroadcastStream::new(event_bus.subscribe())
            .filter_map(move |event| {
                match event {
                    Ok(SystemEvent::LearningEvent(learning)) => {
                        // TODO: Filter by layer if provided
                        
                        Some(LearningEvent {
                            event_type: learning.event_type,
                            pattern_id: learning.pattern_id.map(|id| ID(id.to_string())),
                            confidence_delta: learning.confidence_delta,
                            description: learning.description,
                            timestamp: learning.timestamp,
                        })
                    }
                    _ => None,
                }
            })
    }
}

// ============ Helper Functions ============

pub fn publish_signal_update(
    event_bus: &EventBus,
    signal_id: Uuid,
    status: String,
    progress: Option<f32>,
    message: Option<String>,
) {
    event_bus.publish(SystemEvent::SignalUpdate(SignalUpdateEvent {
        signal_id,
        status,
        progress,
        message,
        timestamp: Utc::now(),
    }));
}

pub fn publish_neuron_state_change(
    event_bus: &EventBus,
    neuron_id: Uuid,
    previous_state: String,
    new_state: String,
    reason: Option<String>,
) {
    event_bus.publish(SystemEvent::NeuronStateChange(NeuronStateChangeEvent {
        neuron_id,
        previous_state,
        new_state,
        reason,
        timestamp: Utc::now(),
    }));
}

pub fn publish_metrics_update(
    event_bus: &EventBus,
    metric_type: String,
    value: f64,
    labels: serde_json::Value,
) {
    event_bus.publish(SystemEvent::MetricsUpdate(MetricsUpdateEvent {
        metric_type,
        value,
        labels,
        timestamp: Utc::now(),
    }));
}

pub fn publish_learning_event(
    event_bus: &EventBus,
    event_type: String,
    pattern_id: Option<Uuid>,
    confidence_delta: f64,
    description: String,
) {
    event_bus.publish(SystemEvent::LearningEvent(LearningEventData {
        event_type,
        pattern_id,
        confidence_delta,
        description,
        timestamp: Utc::now(),
    }));
}