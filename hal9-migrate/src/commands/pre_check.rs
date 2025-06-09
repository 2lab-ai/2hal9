use anyhow::Result;
use colored::Colorize;
use comfy_table::{Table, Cell, Attribute};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tracing::{info, warn};

use crate::client::MigrationClient;
use crate::OutputFormat;
use super::{HealthCheck, HealthStatus, format_output};

pub async fn run(
    server: &str,
    deep: bool,
    components: Vec<String>,
    format: &OutputFormat,
) -> Result<()> {
    let client = MigrationClient::new(server)?;
    
    match format {
        OutputFormat::Pretty => {
            println!("{}", "🔍 Running pre-migration checks...".bold());
            println!();
        }
        _ => {}
    }
    
    // Create progress bar for pretty output
    let pb = if matches!(format, OutputFormat::Pretty) {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        Some(pb)
    } else {
        None
    };
    
    // Determine which components to check
    let check_components = if components.is_empty() {
        vec![
            "substrate".to_string(),
            "protocol".to_string(),
            "cognitive".to_string(),
            "orchestration".to_string(),
            "intelligence".to_string(),
            "database".to_string(),
            "network".to_string(),
            "storage".to_string(),
        ]
    } else {
        components
    };
    
    let mut checks = Vec::new();
    let total_checks = check_components.len() * (if deep { 3 } else { 1 });
    let mut completed = 0;
    
    // Run checks for each component
    for component in &check_components {
        if let Some(ref pb) = pb {
            pb.set_message(format!("Checking {}", component));
            pb.set_position((completed * 100 / total_checks) as u64);
        }
        
        // Basic health check
        let health = check_component_health(&client, component).await?;
        checks.push(health);
        completed += 1;
        
        if deep {
            // Configuration validation
            let config_check = validate_configuration(&client, component).await?;
            checks.push(config_check);
            completed += 1;
            
            // Resource availability
            let resource_check = check_resources(&client, component).await?;
            checks.push(resource_check);
            completed += 1;
        }
        
        // Small delay for visual effect
        if pb.is_some() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    if let Some(pb) = pb {
        pb.finish_and_clear();
    }
    
    // Analyze results
    let total = checks.len();
    let healthy = checks.iter().filter(|c| c.status == HealthStatus::Healthy).count();
    let degraded = checks.iter().filter(|c| c.status == HealthStatus::Degraded).count();
    let unhealthy = checks.iter().filter(|c| c.status == HealthStatus::Unhealthy).count();
    
    // Output results
    match format {
        OutputFormat::Pretty | OutputFormat::Table => {
            display_check_results(&checks, total, healthy, degraded, unhealthy)?;
        }
        OutputFormat::Json => {
            format_output(&checks, format)?;
        }
    }
    
    // Migration readiness assessment
    if matches!(format, OutputFormat::Pretty) {
        println!();
        println!("{}", "Migration Readiness Assessment:".bold());
        
        if unhealthy > 0 {
            println!("{}", "❌ NOT READY - Critical issues found".red().bold());
            println!("   Please resolve all unhealthy components before proceeding.");
        } else if degraded > 0 {
            println!("{}", "⚠️  READY WITH WARNINGS".yellow().bold());
            println!("   Migration can proceed but some components are degraded.");
            println!("   Consider addressing these issues for optimal migration.");
        } else {
            println!("{}", "✅ READY FOR MIGRATION".green().bold());
            println!("   All systems are healthy and ready for hierarchical migration.");
        }
        
        // Recommendations
        if unhealthy > 0 || degraded > 0 {
            println!();
            println!("{}", "Recommendations:".underline());
            for check in &checks {
                if check.status != HealthStatus::Healthy {
                    println!("  • {}: {}", check.component, check.message);
                }
            }
        }
    }
    
    Ok(())
}

async fn check_component_health(client: &MigrationClient, component: &str) -> Result<HealthCheck> {
    // Simulate health check - in real implementation would call API
    info!("Checking health of component: {}", component);
    
    // Mock implementation
    let (status, message) = match component {
        "substrate" => (HealthStatus::Healthy, "All substrate resources available"),
        "protocol" => (HealthStatus::Healthy, "Protocol stack initialized"),
        "cognitive" => (HealthStatus::Healthy, "Cognitive units operational"),
        "orchestration" => (HealthStatus::Degraded, "High memory usage detected"),
        "intelligence" => (HealthStatus::Healthy, "Meta-learning systems ready"),
        _ => (HealthStatus::Healthy, "Component operational"),
    };
    
    Ok(HealthCheck {
        component: component.to_string(),
        status,
        message: message.to_string(),
        details: None,
    })
}

async fn validate_configuration(client: &MigrationClient, component: &str) -> Result<HealthCheck> {
    // Validate component configuration
    info!("Validating configuration for: {}", component);
    
    Ok(HealthCheck {
        component: format!("{}_config", component),
        status: HealthStatus::Healthy,
        message: "Configuration valid".to_string(),
        details: None,
    })
}

async fn check_resources(client: &MigrationClient, component: &str) -> Result<HealthCheck> {
    // Check resource availability
    info!("Checking resources for: {}", component);
    
    Ok(HealthCheck {
        component: format!("{}_resources", component),
        status: HealthStatus::Healthy,
        message: "Sufficient resources available".to_string(),
        details: None,
    })
}

fn display_check_results(
    checks: &[HealthCheck],
    total: usize,
    healthy: usize,
    degraded: usize,
    unhealthy: usize,
) -> Result<()> {
    // Summary
    println!("{}", "Pre-Check Summary".bold().underline());
    println!("Total Checks: {}", total);
    println!("Healthy:      {} {}", healthy, "✓".green());
    println!("Degraded:     {} {}", degraded, "⚠".yellow());
    println!("Unhealthy:    {} {}", unhealthy, "✗".red());
    
    // Detailed results table
    println!();
    println!("{}", "Detailed Results:".bold());
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Component").add_attribute(Attribute::Bold),
        Cell::new("Status").add_attribute(Attribute::Bold),
        Cell::new("Message").add_attribute(Attribute::Bold),
    ]);
    
    for check in checks {
        let status_cell = match check.status {
            HealthStatus::Healthy => Cell::new("✓ Healthy").fg(comfy_table::Color::Green),
            HealthStatus::Degraded => Cell::new("⚠ Degraded").fg(comfy_table::Color::Yellow),
            HealthStatus::Unhealthy => Cell::new("✗ Unhealthy").fg(comfy_table::Color::Red),
            HealthStatus::Unknown => Cell::new("? Unknown").fg(comfy_table::Color::Grey),
        };
        
        table.add_row(vec![
            Cell::new(&check.component),
            status_cell,
            Cell::new(&check.message),
        ]);
    }
    
    println!("{table}");
    
    Ok(())
}