//! Prometheus metrics exporter

use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use crate::server::HAL9Server;

/// Prometheus metric types
#[allow(dead_code)]
enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Generate Prometheus-formatted metrics
pub async fn export_metrics(server: Arc<HAL9Server>) -> String {
    let mut output = String::new();
    let metrics = server.metrics();
    let snapshot = metrics.snapshot();
    let server_id = server.server_id();
    
    // System metrics
    write_metric(
        &mut output,
        "hal9_server_uptime_seconds",
        "Server uptime in seconds",
        MetricType::Gauge,
        snapshot.uptime_seconds as f64,
        &[("server_id", server_id)],
    );
    
    write_metric(
        &mut output,
        "hal9_server_memory_bytes",
        "Server memory usage in bytes",
        MetricType::Gauge,
        snapshot.memory_usage_mb * 1024.0 * 1024.0,
        &[("server_id", server_id), ("type", "total")],
    );
    
    // Signal metrics
    write_metric(
        &mut output,
        "hal9_signals_sent_total",
        "Total signals sent",
        MetricType::Counter,
        snapshot.signals_sent as f64,
        &[("server_id", server_id)],
    );
    
    write_metric(
        &mut output,
        "hal9_signals_processed_total",
        "Total signals processed",
        MetricType::Counter,
        snapshot.signals_processed as f64,
        &[("server_id", server_id), ("status", "success")],
    );
    
    write_metric(
        &mut output,
        "hal9_signals_processed_total",
        "Total signals processed",
        MetricType::Counter,
        snapshot.signals_failed as f64,
        &[("server_id", server_id), ("status", "error")],
    );
    
    write_metric(
        &mut output,
        "hal9_signals_rate_per_second",
        "Signal processing rate per second",
        MetricType::Gauge,
        snapshot.signals_per_second,
        &[("server_id", server_id)],
    );
    
    // Neuron metrics
    write_metric(
        &mut output,
        "hal9_neurons_active",
        "Number of active neurons",
        MetricType::Gauge,
        snapshot.neurons_active as f64,
        &[("server_id", server_id)],
    );
    
    write_metric(
        &mut output,
        "hal9_neurons_processing",
        "Number of neurons currently processing",
        MetricType::Gauge,
        snapshot.neurons_processing as f64,
        &[("server_id", server_id)],
    );
    
    write_metric(
        &mut output,
        "hal9_neurons_failed_total",
        "Total neuron failures",
        MetricType::Counter,
        snapshot.neurons_failed as f64,
        &[("server_id", server_id)],
    );
    
    // Latency metrics by layer
    for (layer, stats) in &snapshot.layer_latencies {
        write_metric(
            &mut output,
            "hal9_signal_processing_duration_seconds_sum",
            "Total signal processing time",
            MetricType::Counter,
            stats.avg_ms * stats.count as f64 / 1000.0,
            &[("server_id", server_id), ("layer", layer)],
        );
        
        write_metric(
            &mut output,
            "hal9_signal_processing_duration_seconds_count",
            "Number of signals processed",
            MetricType::Counter,
            stats.count as f64,
            &[("server_id", server_id), ("layer", layer)],
        );
        
        // Add histogram buckets
        write_histogram_buckets(
            &mut output,
            "hal9_signal_processing_duration_seconds",
            stats,
            &[("server_id", server_id), ("layer", layer)],
        );
    }
    
    // Processing times by neuron
    for (neuron_id, stats) in &snapshot.processing_times {
        write_metric(
            &mut output,
            "hal9_neuron_processing_duration_seconds_sum",
            "Total neuron processing time",
            MetricType::Counter,
            stats.avg_ms * stats.count as f64 / 1000.0,
            &[("server_id", server_id), ("neuron_id", neuron_id)],
        );
        
        write_metric(
            &mut output,
            "hal9_neuron_processing_duration_seconds_count",
            "Number of processing operations",
            MetricType::Counter,
            stats.count as f64,
            &[("server_id", server_id), ("neuron_id", neuron_id)],
        );
    }
    
    // Claude API metrics
    write_metric(
        &mut output,
        "hal9_claude_tokens_used_total",
        "Total tokens used",
        MetricType::Counter,
        snapshot.tokens_prompt as f64,
        &[("server_id", server_id), ("type", "prompt")],
    );
    
    write_metric(
        &mut output,
        "hal9_claude_tokens_used_total",
        "Total tokens used",
        MetricType::Counter,
        snapshot.tokens_completion as f64,
        &[("server_id", server_id), ("type", "completion")],
    );
    
    write_metric(
        &mut output,
        "hal9_claude_tokens_used_total",
        "Total tokens used",
        MetricType::Counter,
        snapshot.tokens_total as f64,
        &[("server_id", server_id), ("type", "total")],
    );
    
    // Cost metrics
    write_metric(
        &mut output,
        "hal9_claude_cost_dollars_total",
        "Total API cost in dollars",
        MetricType::Counter,
        snapshot.cost_total,
        &[("server_id", server_id)],
    );
    
    write_metric(
        &mut output,
        "hal9_claude_cost_rate_dollars_per_hour",
        "Current cost rate in dollars per hour",
        MetricType::Gauge,
        snapshot.cost_hourly,
        &[("server_id", server_id)],
    );
    
    write_metric(
        &mut output,
        "hal9_claude_cost_rate_dollars_per_day",
        "Current cost rate in dollars per day",
        MetricType::Gauge,
        snapshot.cost_daily,
        &[("server_id", server_id)],
    );
    
    // Error metrics
    for (error_type, count) in &snapshot.errors_by_type {
        write_metric(
            &mut output,
            "hal9_errors_total",
            "Total errors by type",
            MetricType::Counter,
            *count as f64,
            &[("server_id", server_id), ("error_type", error_type)],
        );
    }
    
    // Authentication metrics (if enabled)
    if server.user_manager.is_some() {
        if let Ok(user_count) = server.get_active_user_count().await {
            write_metric(
                &mut output,
                "hal9_auth_users_active",
                "Number of active users",
                MetricType::Gauge,
                user_count as f64,
                &[("server_id", server_id)],
            );
        }
    }
    
    // MCP tool metrics
    if let Some(tool_metrics) = server.get_mcp_tool_metrics().await {
        for (tool_name, invocations) in &tool_metrics.invocations {
            write_metric(
                &mut output,
                "hal9_mcp_tool_invocations_total",
                "Total tool invocations",
                MetricType::Counter,
                invocations.success as f64,
                &[("server_id", server_id), ("tool", tool_name), ("status", "success")],
            );
            
            write_metric(
                &mut output,
                "hal9_mcp_tool_invocations_total",
                "Total tool invocations",
                MetricType::Counter,
                invocations.error as f64,
                &[("server_id", server_id), ("tool", tool_name), ("status", "error")],
            );
        }
    }
    
    // Memory system metrics
    if let Some(memory_metrics) = server.get_memory_metrics().await {
        write_metric(
            &mut output,
            "hal9_memory_entries_total",
            "Total memory entries",
            MetricType::Gauge,
            memory_metrics.total_entries as f64,
            &[("server_id", server_id)],
        );
        
        write_metric(
            &mut output,
            "hal9_memory_database_bytes",
            "Memory database size in bytes",
            MetricType::Gauge,
            memory_metrics.database_size_bytes as f64,
            &[("server_id", server_id)],
        );
    }
    
    // Learning metrics
    if let Some(learning_metrics) = server.get_learning_metrics().await {
        write_metric(
            &mut output,
            "hal9_learning_cycles_total",
            "Total learning cycles completed",
            MetricType::Counter,
            learning_metrics.cycles_completed as f64,
            &[("server_id", server_id)],
        );
        
        write_metric(
            &mut output,
            "hal9_learning_adjustments_total",
            "Total prompt adjustments made",
            MetricType::Counter,
            learning_metrics.adjustments_success as f64,
            &[("server_id", server_id), ("type", "success")],
        );
        
        write_metric(
            &mut output,
            "hal9_learning_adjustments_total",
            "Total prompt adjustments made",
            MetricType::Counter,
            learning_metrics.adjustments_failure as f64,
            &[("server_id", server_id), ("type", "failure")],
        );
    }
    
    output
}

/// Write a single metric in Prometheus format
fn write_metric(
    output: &mut String,
    name: &str,
    help: &str,
    metric_type: MetricType,
    value: f64,
    labels: &[(&str, &str)],
) {
    // Write HELP and TYPE only once per metric name
    if !output.contains(&format!("# HELP {}", name)) {
        writeln!(output, "# HELP {} {}", name, help).unwrap();
        writeln!(output, "# TYPE {} {}", name, match metric_type {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
        }).unwrap();
    }
    
    // Write metric with labels
    write!(output, "{}", name).unwrap();
    if !labels.is_empty() {
        write!(output, "{{").unwrap();
        for (i, (key, value)) in labels.iter().enumerate() {
            if i > 0 {
                write!(output, ",").unwrap();
            }
            write!(output, r#"{}="{}""#, key, value).unwrap();
        }
        write!(output, "}}").unwrap();
    }
    writeln!(output, " {}", value).unwrap();
}

/// Write histogram buckets
fn write_histogram_buckets(
    output: &mut String,
    name: &str,
    stats: &crate::metrics::LatencyStats,
    base_labels: &[(&str, &str)],
) {
    // Define bucket boundaries (in seconds)
    let buckets = [0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 60.0];
    
    // Calculate bucket counts based on average (simplified)
    let avg_seconds = stats.avg_ms / 1000.0;
    let mut cumulative_count = 0u64;
    
    for &bucket in &buckets {
        if avg_seconds <= bucket {
            cumulative_count = stats.count;
        }
        
        // Create labels with le bucket
        let bucket_str = format!("{}", bucket);
        let mut labels = base_labels.to_vec();
        labels.push(("le", &bucket_str));
        
        write!(output, "{}_bucket{{", name).unwrap();
        for (i, (key, value)) in labels.iter().enumerate() {
            if i > 0 {
                write!(output, ",").unwrap();
            }
            write!(output, r#"{}="{}""#, key, value).unwrap();
        }
        writeln!(output, "}} {}", cumulative_count).unwrap();
    }
    
    // +Inf bucket
    let mut labels = base_labels.to_vec();
    labels.push(("le", "+Inf"));
    
    write!(output, "{}_bucket{{", name).unwrap();
    for (i, (key, value)) in labels.iter().enumerate() {
        if i > 0 {
            write!(output, ",").unwrap();
        }
        write!(output, r#"{}="{}""#, key, value).unwrap();
    }
    writeln!(output, "}} {}", stats.count).unwrap();
}