//! Demo recording and replay functionality

use anyhow::{Result, Context};
#[allow(unused_imports)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::Signal;

/// A recorded demo session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoRecording {
    pub id: Uuid,
    pub scenario: String,
    pub recorded_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub events: Vec<RecordedEvent>,
    pub metadata: RecordingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    pub version: String,
    pub platform: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub timestamp_ms: u64,
    pub event_type: EventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventType {
    Signal {
        signal: Signal,
    },
    NeuronActivation {
        neuron_id: String,
        layer: String,
    },
    CodeGenerated {
        layer: String,
        content: String,
    },
    StatusUpdate {
        message: String,
    },
}

/// Records demo sessions for later replay
pub struct DemoRecorder {
    recording: Arc<Mutex<Option<ActiveRecording>>>,
}

struct ActiveRecording {
    id: Uuid,
    scenario: String,
    start_time: Instant,
    start_timestamp: DateTime<Utc>,
    events: Vec<RecordedEvent>,
}

impl DemoRecorder {
    pub fn new() -> Self {
        Self {
            recording: Arc::new(Mutex::new(None)),
        }
    }

    /// Start recording a new demo
    pub async fn start_recording(&self, scenario: String) -> Result<Uuid> {
        let mut recording = self.recording.lock().await;
        
        if recording.is_some() {
            anyhow::bail!("A recording is already in progress");
        }

        let id = Uuid::new_v4();
        *recording = Some(ActiveRecording {
            id,
            scenario,
            start_time: Instant::now(),
            start_timestamp: Utc::now(),
            events: Vec::new(),
        });

        Ok(id)
    }

    /// Record a signal event
    pub async fn record_signal(&self, signal: Signal) -> Result<()> {
        self.record_event(EventType::Signal { signal }).await
    }

    /// Record neuron activation
    pub async fn record_neuron_activation(&self, neuron_id: String, layer: String) -> Result<()> {
        self.record_event(EventType::NeuronActivation { neuron_id, layer }).await
    }

    /// Record generated code
    pub async fn record_code_generated(&self, layer: String, content: String) -> Result<()> {
        self.record_event(EventType::CodeGenerated { layer, content }).await
    }

    /// Record status update
    pub async fn record_status(&self, message: String) -> Result<()> {
        self.record_event(EventType::StatusUpdate { message }).await
    }

    /// Internal: record any event type
    async fn record_event(&self, event_type: EventType) -> Result<()> {
        let mut recording = self.recording.lock().await;
        
        if let Some(active) = recording.as_mut() {
            let elapsed = active.start_time.elapsed();
            active.events.push(RecordedEvent {
                timestamp_ms: elapsed.as_millis() as u64,
                event_type,
            });
            Ok(())
        } else {
            anyhow::bail!("No active recording")
        }
    }

    /// Stop recording and return the completed recording
    pub async fn stop_recording(&self) -> Result<DemoRecording> {
        let mut recording = self.recording.lock().await;
        
        let active = recording.take()
            .context("No active recording to stop")?;

        let duration_ms = active.start_time.elapsed().as_millis() as u64;

        Ok(DemoRecording {
            id: active.id,
            scenario: active.scenario,
            recorded_at: active.start_timestamp,
            duration_ms,
            events: active.events,
            metadata: RecordingMetadata {
                version: "1.0".to_string(),
                platform: std::env::consts::OS.to_string(),
                description: None,
            },
        })
    }

    /// Save recording to file
    pub async fn save_recording(&self, recording: &DemoRecording, dir: &Path) -> Result<PathBuf> {
        tokio::fs::create_dir_all(dir).await?;
        
        let filename = format!(
            "demo_{}_{}.json",
            recording.scenario.replace(' ', "_").to_lowercase(),
            recording.recorded_at.format("%Y%m%d_%H%M%S")
        );
        
        let path = dir.join(filename);
        let json = serde_json::to_string_pretty(recording)?;
        
        tokio::fs::write(&path, json).await?;
        
        Ok(path)
    }

    /// Load recording from file
    pub async fn load_recording(path: &Path) -> Result<DemoRecording> {
        let json = tokio::fs::read_to_string(path).await?;
        let recording = serde_json::from_str(&json)?;
        Ok(recording)
    }
}

/// Replays recorded demo sessions
pub struct DemoPlayer {
    speed: f32,
}

impl DemoPlayer {
    pub fn new() -> Self {
        Self { speed: 1.0 }
    }

    /// Set playback speed (1.0 = normal, 2.0 = double speed, etc.)
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.1).min(10.0);
    }

    /// Play a recorded demo with callbacks for each event
    pub async fn play<F>(
        &self,
        recording: &DemoRecording,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(&RecordedEvent) -> Result<()>,
    {
        println!("🎬 Playing demo: {} (recorded {})", 
            recording.scenario,
            recording.recorded_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!("⏱️  Duration: {:.1}s at {}x speed", 
            recording.duration_ms as f32 / 1000.0,
            self.speed
        );

        let start_time = Instant::now();
        let mut last_timestamp = 0u64;

        for event in &recording.events {
            // Calculate delay until this event
            let delay_ms = event.timestamp_ms.saturating_sub(last_timestamp);
            last_timestamp = event.timestamp_ms;

            if delay_ms > 0 {
                let adjusted_delay = (delay_ms as f32 / self.speed) as u64;
                tokio::time::sleep(Duration::from_millis(adjusted_delay)).await;
            }

            // Process the event
            callback(event)?;
        }

        let actual_duration = start_time.elapsed();
        println!("\n✅ Playback complete in {:.1}s", actual_duration.as_secs_f32());

        Ok(())
    }

    /// Export recording as a script that can be replayed
    #[allow(dead_code)]
    pub async fn export_as_script(
        &self,
        recording: &DemoRecording,
        path: &Path,
    ) -> Result<()> {
        let mut script = String::new();
        
        script.push_str(&format!(
            "#!/usr/bin/env node\n\
            // 2HAL9 Demo Replay Script\n\
            // Scenario: {}\n\
            // Recorded: {}\n\
            // Duration: {:.1}s\n\n",
            recording.scenario,
            recording.recorded_at.format("%Y-%m-%d %H:%M:%S"),
            recording.duration_ms as f32 / 1000.0
        ));

        script.push_str("const events = ");
        script.push_str(&serde_json::to_string_pretty(&recording.events)?);
        script.push_str(";\n\n");

        script.push_str(r#"
// Replay function
async function replay() {
    let lastTime = 0;
    
    for (const event of events) {
        const delay = event.timestamp_ms - lastTime;
        if (delay > 0) {
            await new Promise(resolve => setTimeout(resolve, delay));
        }
        lastTime = event.timestamp_ms;
        
        console.log(`[${event.timestamp_ms}ms]`, event.event_type);
        
        // Add custom event handling here
        switch (event.event_type.type) {
            case 'Signal':
                console.log('  Signal:', event.event_type.signal.from, '→', event.event_type.signal.to);
                break;
            case 'CodeGenerated':
                console.log('  Generated', event.event_type.content.length, 'chars of code');
                break;
        }
    }
}

// Run replay
console.log('🎬 Starting replay...\n');
replay().then(() => {
    console.log('\n✅ Replay complete!');
}).catch(console.error);
"#);

        tokio::fs::write(path, script).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recording_lifecycle() {
        let recorder = DemoRecorder::new();
        
        // Start recording
        let id = recorder.start_recording("Test Demo".to_string()).await.unwrap();
        
        // Record some events
        recorder.record_status("Starting demo".to_string()).await.unwrap();
        
        let signal = Signal {
            id: Uuid::new_v4(),
            parent_id: None,
            from: "user".to_string(),
            to: "neuron-1".to_string(),
            content: "Test signal".to_string(),
            layer: "Input".to_string(),
            timestamp: Utc::now(),
        };
        recorder.record_signal(signal).await.unwrap();
        
        recorder.record_neuron_activation("neuron-1".to_string(), "L4".to_string()).await.unwrap();
        
        // Stop recording
        let recording = recorder.stop_recording().await.unwrap();
        
        assert_eq!(recording.id, id);
        assert_eq!(recording.scenario, "Test Demo");
        assert_eq!(recording.events.len(), 3);
    }
}