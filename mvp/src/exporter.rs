//! Export functionality for demo recordings to various formats

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use crate::recorder::{DemoRecording, EventType};

/// Exports demo recordings to various visual formats
pub struct DemoExporter;

impl DemoExporter {
    /// Export recording as animated SVG
    pub async fn export_as_svg(
        recording: &DemoRecording,
        output_path: &Path,
    ) -> Result<()> {
        let mut svg = String::new();
        
        // SVG header with animation duration
        let duration_secs = recording.duration_ms as f32 / 1000.0;
        
        // Define colors as variables to avoid prefix issues
        let background_color = "#1a1a1a";
        let text_color = "#ffffff";
        let subtitle_color = "#888888";
        let l4_fill = "#6366f1";
        let l4_stroke = "#8b92ff";
        let l3_fill = "#f59e0b";
        let l3_stroke = "#fbbf24";
        let l2_fill = "#10b981";
        let l2_stroke = "#34d399";
        
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <rect width="800" height="600" fill="{}"/>
  
  <!-- Title -->
  <text x="400" y="40" font-family="Arial, sans-serif" font-size="24" fill="{}" text-anchor="middle">
    2HAL9 Demo: {}
  </text>
  <text x="400" y="65" font-family="Arial, sans-serif" font-size="14" fill="{}" text-anchor="middle">
    Duration: {:.1}s
  </text>
  
  <!-- Neurons -->
  <g id="neurons">
    <!-- L4 Strategic (neuron-1) -->
    <circle cx="400" cy="150" r="40" fill="{}" stroke="{}" stroke-width="2">
      <animate attributeName="r" values="40;45;40" dur="2s" repeatCount="indefinite"/>
    </circle>
    <text x="400" y="155" font-family="Arial" font-size="20" fill="white" text-anchor="middle">L4</text>
    
    <!-- L3 Design (neuron-2) -->
    <circle cx="250" cy="300" r="35" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="250" y="305" font-family="Arial" font-size="18" fill="white" text-anchor="middle">L3</text>
    
    <!-- L3 Design (neuron-3) -->
    <circle cx="550" cy="300" r="35" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="550" y="305" font-family="Arial" font-size="18" fill="white" text-anchor="middle">L3</text>
    
    <!-- L2 Implementation (neuron-4) -->
    <circle cx="400" cy="450" r="40" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="400" y="455" font-family="Arial" font-size="20" fill="white" text-anchor="middle">L2</text>
  </g>
  
  <!-- Signal Flow Animations -->
  <g id="signals">
"#, 
            background_color, text_color, recording.scenario, subtitle_color, duration_secs,
            l4_fill, l4_stroke, l3_fill, l3_stroke, l3_fill, l3_stroke, l2_fill, l2_stroke
        ));

        // Generate signal animations from events
        let mut signal_count = 0;
        for event in &recording.events {
            if let EventType::Signal { signal } = &event.event_type {
                let time_offset = event.timestamp_ms as f32 / 1000.0;
                
                // Determine coordinates based on neuron positions
                let (x1, y1, x2, y2) = match (signal.from.as_str(), signal.to.as_str()) {
                    ("user", "neuron-1") => (400, 50, 400, 150),
                    ("neuron-1", "neuron-2") => (400, 150, 250, 300),
                    ("neuron-1", "neuron-3") => (400, 150, 550, 300),
                    ("neuron-2", "neuron-4") => (250, 300, 400, 450),
                    ("neuron-3", "neuron-4") => (550, 300, 400, 450),
                    _ => continue,
                };
                
                // Create animated signal path
                let signal_color = "#4ade80";
                svg.push_str(&format!(
                    "    <!-- Signal {} -->\n    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"3\" opacity=\"0\">\n      <animate attributeName=\"opacity\" values=\"0;1;1;0\" dur=\"1s\" begin=\"{}s\" fill=\"freeze\"/>\n    </line>\n    <circle r=\"5\" fill=\"{}\" opacity=\"0\">\n      <animateMotion dur=\"0.5s\" begin=\"{}s\" fill=\"freeze\">\n        <mpath xlink:href=\"#path{}\"/>\n      </animateMotion>\n      <animate attributeName=\"opacity\" values=\"0;1;1;0\" dur=\"1s\" begin=\"{}s\" fill=\"freeze\"/>\n    </circle>\n    <path id=\"path{}\" d=\"M {} {} L {} {}\" fill=\"none\"/>\n",
                    signal_count, x1, y1, x2, y2, signal_color, time_offset,
                    signal_color, time_offset, signal_count, time_offset,
                    signal_count, x1, y1, x2, y2
                ));
                signal_count += 1;
            }
        }
        
        svg.push_str("  </g>\n\n");
        
        // Add status messages
        svg.push_str("  <!-- Status Messages -->\n  <g id=\"status\">\n");
        let mut message_y = 520;
        for event in &recording.events {
            if let EventType::StatusUpdate { message } = &event.event_type {
                let time_offset = event.timestamp_ms as f32 / 1000.0;
                let status_color = "#888888";
                svg.push_str(&format!(
                    r#"    <text x="400" y="{}" font-family="monospace" font-size="12" fill="{}" text-anchor="middle" opacity="0">
      {}
      <animate attributeName="opacity" values="0;1;1;0" dur="2s" begin="{}s" fill="freeze"/>
    </text>
"#,
                    message_y, status_color, message, time_offset
                ));
                message_y += 20;
            }
        }
        svg.push_str("  </g>\n");
        
        // Close SVG
        svg.push_str("</svg>");
        
        tokio::fs::write(output_path, svg).await
            .context("Failed to write SVG file")?;
        
        Ok(())
    }
    
    /// Export recording as a series of PNG frames (for GIF conversion)
    pub async fn export_frames(
        recording: &DemoRecording,
        output_dir: &Path,
        fps: u32,
    ) -> Result<Vec<PathBuf>> {
        tokio::fs::create_dir_all(output_dir).await?;
        
        let frame_duration_ms = 1000 / fps;
        let total_frames = (recording.duration_ms / frame_duration_ms as u64) + 1;
        let mut frame_paths = Vec::new();
        
        // Generate frame data for each timestamp
        for frame_num in 0..total_frames {
            let timestamp_ms = frame_num * frame_duration_ms as u64;
            
            // Find all events that should be visible at this timestamp
            let visible_events: Vec<_> = recording.events.iter()
                .filter(|e| e.timestamp_ms <= timestamp_ms)
                .collect();
            
            // Generate SVG for this frame
            let svg = Self::generate_frame_svg(&recording.scenario, timestamp_ms, &visible_events);
            
            // Save frame as SVG (can be converted to PNG with external tools)
            let frame_path = output_dir.join(format!("frame_{:05}.svg", frame_num));
            tokio::fs::write(&frame_path, svg).await?;
            frame_paths.push(frame_path);
        }
        
        Ok(frame_paths)
    }
    
    /// Generate SVG for a single frame
    fn generate_frame_svg(scenario: &str, timestamp_ms: u64, events: &[&crate::recorder::RecordedEvent]) -> String {
        // Define colors as variables
        let bg_color = "#1a1a1a";
        let title_color = "#ffffff";
        let timer_color = "#888888";
        let l4_fill = "#6366f1";
        let l4_stroke = "#8b92ff";
        let l3_fill = "#f59e0b";
        let l3_stroke = "#fbbf24";
        let l2_fill = "#10b981";
        let l2_stroke = "#34d399";
        
        let mut svg = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">
  <rect width="800" height="600" fill="{}"/>
  
  <!-- Title and Timer -->
  <text x="400" y="40" font-family="Arial, sans-serif" font-size="24" fill="{}" text-anchor="middle">{}</text>
  <text x="400" y="65" font-family="Arial, sans-serif" font-size="14" fill="{}" text-anchor="middle">Time: {:.1}s</text>
  
  <!-- Neurons -->
  <g id="neurons">
    <circle cx="400" cy="150" r="40" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="400" y="155" font-family="Arial" font-size="20" fill="white" text-anchor="middle">L4</text>
    
    <circle cx="250" cy="300" r="35" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="250" y="305" font-family="Arial" font-size="18" fill="white" text-anchor="middle">L3</text>
    
    <circle cx="550" cy="300" r="35" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="550" y="305" font-family="Arial" font-size="18" fill="white" text-anchor="middle">L3</text>
    
    <circle cx="400" cy="450" r="40" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="400" y="455" font-family="Arial" font-size="20" fill="white" text-anchor="middle">L2</text>
  </g>
  
  <!-- Active Signals -->
  <g id="signals">
"#, bg_color, title_color, scenario, timer_color, timestamp_ms as f32 / 1000.0,
    l4_fill, l4_stroke, l3_fill, l3_stroke, l3_fill, l3_stroke, l2_fill, l2_stroke
        );
        
        // Show recent signals (within last 500ms)
        for event in events.iter().rev().take(10) {
            if let EventType::Signal { signal } = &event.event_type {
                let age_ms = timestamp_ms.saturating_sub(event.timestamp_ms);
                if age_ms < 500 {
                    let opacity = 1.0 - (age_ms as f32 / 500.0);
                    
                    let (x1, y1, x2, y2) = match (signal.from.as_str(), signal.to.as_str()) {
                        ("user", "neuron-1") => (400, 50, 400, 150),
                        ("neuron-1", "neuron-2") => (400, 150, 250, 300),
                        ("neuron-1", "neuron-3") => (400, 150, 550, 300),
                        ("neuron-2", "neuron-4") => (250, 300, 400, 450),
                        ("neuron-3", "neuron-4") => (550, 300, 400, 450),
                        _ => continue,
                    };
                    
                    let signal_color = "#4ade80";
                    svg.push_str(&format!(
                        r#"    <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="3" opacity="{}"/>
"#,
                        x1, y1, x2, y2, signal_color, opacity
                    ));
                }
            }
        }
        
        svg.push_str("  </g>\n</svg>");
        svg
    }
    
    /// Export as GIF conversion script
    pub async fn export_gif_script(
        recording: &DemoRecording,
        output_path: &Path,
    ) -> Result<()> {
        let script = format!(
            r#"#!/bin/bash
# 2HAL9 Demo to GIF Conversion Script
# Scenario: {}
# Duration: {:.1}s

# This script converts the demo recording to an animated GIF
# Requirements: ImageMagick (convert command) or FFmpeg

echo "Converting 2HAL9 demo to GIF..."

# Option 1: Using ImageMagick
# First export frames, then convert to GIF
echo "Exporting frames..."
cargo run -p hal9_mvp -- export-frames "$1" frames/ --fps 30

echo "Converting SVG frames to PNG..."
for svg in frames/*.svg; do
    png="$(echo $svg | sed 's/\.svg$/.png/')"
    convert -background '#1a1a1a' "$svg" "$png"
done

echo "Creating animated GIF..."
convert -delay 3 -loop 0 frames/*.png demo.gif

# Option 2: Using FFmpeg (alternative)
# ffmpeg -framerate 30 -pattern_type glob -i 'frames/*.png' \
#        -vf scale=800:600 \
#        -c:v gif demo.gif

echo "GIF created successfully"
echo "File size: $(du -h demo.gif | cut -f1)"

# Cleanup
# rm -rf frames/
"#,
            recording.scenario,
            recording.duration_ms as f32 / 1000.0
        );
        
        tokio::fs::write(output_path, script).await?;
        
        // Make script executable on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(output_path).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(output_path, perms).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_svg_export() {
        let recording = DemoRecording {
            id: Uuid::new_v4(),
            scenario: "Test Demo".to_string(),
            recorded_at: Utc::now(),
            duration_ms: 3000,
            events: vec![
                crate::recorder::RecordedEvent {
                    timestamp_ms: 100,
                    event_type: EventType::Signal {
                        signal: crate::Signal {
                            id: Uuid::new_v4(),
                            parent_id: None,
                            from: "user".to_string(),
                            to: "neuron-1".to_string(),
                            content: "Test".to_string(),
                            layer: "Input".to_string(),
                            timestamp: Utc::now(),
                        }
                    }
                }
            ],
            metadata: crate::recorder::RecordingMetadata {
                version: "1.0".to_string(),
                platform: "test".to_string(),
                description: None,
            }
        };
        
        let temp_dir = tempfile::TempDir::new().unwrap();
        let svg_path = temp_dir.path().join("test.svg");
        
        DemoExporter::export_as_svg(&recording, &svg_path).await.unwrap();
        
        // Verify SVG was created
        assert!(svg_path.exists());
        let svg_content = tokio::fs::read_to_string(&svg_path).await.unwrap();
        assert!(svg_content.contains("<svg"));
        assert!(svg_content.contains("Test Demo"));
    }
}