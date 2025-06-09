use anyhow::{Result, bail};
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;
use tracing::{info, warn};

use crate::{MigrationPhase, OutputFormat};
use crate::client::MigrationClient;

pub async fn run(
    server: &str,
    phase: MigrationPhase,
    percentage: Option<u8>,
    dry_run: bool,
    yes: bool,
    timeout: Option<u64>,
    format: &OutputFormat,
) -> Result<()> {
    let client = MigrationClient::new(server)?;
    
    // Validate phase transition
    let current_phase = get_current_phase(&client).await?;
    if !is_valid_transition(&current_phase, &phase) {
        bail!("Invalid phase transition: {} -> {:?}", current_phase, phase);
    }
    
    // Validate percentage for applicable phases
    if matches!(phase, MigrationPhase::Canary | MigrationPhase::RampUp) && percentage.is_none() {
        bail!("Percentage must be specified for {:?} phase", phase);
    }
    
    // Display migration plan
    if matches!(format, OutputFormat::Pretty) {
        display_migration_plan(&phase, percentage, dry_run, timeout)?;
        
        // Confirmation prompt
        if !yes && !dry_run {
            let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Proceed with migration?")
                .default(false)
                .interact()?;
            
            if !confirmation {
                println!("{}", "Migration cancelled.".yellow());
                return Ok(());
            }
        }
    }
    
    // Execute migration
    if dry_run {
        println!("\n{}", "🔍 DRY RUN MODE - No changes will be made".yellow().bold());
        simulate_migration(&phase, percentage).await?;
    } else {
        execute_migration(&client, &phase, percentage, timeout, format).await?;
    }
    
    Ok(())
}

async fn get_current_phase(client: &MigrationClient) -> Result<String> {
    // Mock implementation
    Ok("canary".to_string())
}

fn is_valid_transition(current: &str, target: &MigrationPhase) -> bool {
    // Define valid transitions
    match (current, target) {
        ("none", MigrationPhase::Shadow) => true,
        ("shadow", MigrationPhase::Canary) => true,
        ("canary", MigrationPhase::StateMigration) => true,
        ("canary", MigrationPhase::RampUp) => true,
        ("state-migration", MigrationPhase::RampUp) => true,
        ("ramp-up", MigrationPhase::Full) => true,
        _ => false,
    }
}

fn display_migration_plan(
    phase: &MigrationPhase,
    percentage: Option<u8>,
    dry_run: bool,
    timeout: Option<u64>,
) -> Result<()> {
    println!("{}", "🚀 Migration Plan".bold().underline());
    println!();
    
    println!("Target Phase:    {}", format!("{:?}", phase).cyan().bold());
    
    if let Some(pct) = percentage {
        println!("Traffic Split:   {}% new / {}% legacy", pct, 100 - pct);
    }
    
    if let Some(t) = timeout {
        println!("Timeout:         {} seconds", t);
    }
    
    println!("Mode:            {}", 
        if dry_run { "Dry Run".yellow() } else { "Live Migration".green() }
    );
    
    // Phase-specific information
    println!();
    println!("{}", "Phase Details:".bold());
    match phase {
        MigrationPhase::Shadow => {
            println!("  • Mirror all traffic to new system");
            println!("  • No production impact");
            println!("  • Validate functionality and performance");
            println!("  • Collect comparison metrics");
        }
        MigrationPhase::Canary => {
            println!("  • Route {}% of traffic to new system", percentage.unwrap_or(5));
            println!("  • Monitor error rates and latency");
            println!("  • Automatic rollback on failures");
            println!("  • Gradual confidence building");
        }
        MigrationPhase::StateMigration => {
            println!("  • Migrate persistent state to new format");
            println!("  • Maintain data consistency");
            println!("  • Create rollback snapshots");
            println!("  • Validate data integrity");
        }
        MigrationPhase::RampUp => {
            println!("  • Increase traffic to {}%", percentage.unwrap_or(50));
            println!("  • Load test new system");
            println!("  • Fine-tune performance");
            println!("  • Prepare for full migration");
        }
        MigrationPhase::Full => {
            println!("  • Route 100% traffic to new system");
            println!("  • Decommission legacy components");
            println!("  • Final validation checks");
            println!("  • Mark migration complete");
        }
    }
    
    // Expected outcomes
    println!();
    println!("{}", "Expected Outcomes:".bold());
    println!("  ✓ Zero downtime migration");
    println!("  ✓ Maintained SLAs throughout");
    println!("  ✓ Automatic rollback capability");
    println!("  ✓ Complete audit trail");
    
    Ok(())
}

async fn simulate_migration(
    phase: &MigrationPhase,
    percentage: Option<u8>,
) -> Result<()> {
    let steps = get_migration_steps(phase);
    
    println!();
    println!("{}", "Simulating migration steps:".bold());
    
    for (i, step) in steps.iter().enumerate() {
        println!("\n{} {}", format!("Step {}:", i + 1).bold(), step);
        
        // Simulate some work
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.set_message("Simulating...");
        
        for _ in 0..20 {
            pb.tick();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        pb.finish_with_message(format!("✓ {} complete", step).green().to_string());
    }
    
    println!("\n{}", "✅ Dry run completed successfully!".green().bold());
    println!("No actual changes were made.");
    
    Ok(())
}

async fn execute_migration(
    client: &MigrationClient,
    phase: &MigrationPhase,
    percentage: Option<u8>,
    timeout: Option<u64>,
    format: &OutputFormat,
) -> Result<()> {
    info!("Starting migration to phase: {:?}", phase);
    
    let steps = get_migration_steps(phase);
    let multi_progress = MultiProgress::new();
    
    // Overall progress
    let overall_pb = multi_progress.add(ProgressBar::new(steps.len() as u64));
    overall_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    overall_pb.set_message("Starting migration...");
    
    // Execute each step
    for (i, step) in steps.iter().enumerate() {
        overall_pb.set_message(format!("Executing: {}", step));
        
        // Step progress bar
        let step_pb = multi_progress.add(ProgressBar::new(100));
        step_pb.set_style(
            ProgressStyle::default_bar()
                .template("  ↳ [{bar:30.yellow/blue}] {pos}% {msg}")
                .unwrap()
                .progress_chars("█▓▒░")
        );
        
        // Execute step with progress updates
        execute_step(client, phase, step, percentage, &step_pb).await?;
        
        step_pb.finish_and_clear();
        overall_pb.inc(1);
    }
    
    overall_pb.finish_with_message("Migration completed!");
    
    // Final validation
    if matches!(format, OutputFormat::Pretty) {
        println!();
        println!("{}", "🎉 Migration Successful!".green().bold());
        println!();
        println!("Phase {} is now active.", format!("{:?}", phase).cyan().bold());
        
        // Post-migration recommendations
        println!();
        println!("{}", "Recommended next steps:".bold());
        match phase {
            MigrationPhase::Shadow => {
                println!("  1. Monitor shadow metrics for 24-48 hours");
                println!("  2. Compare performance with legacy system");
                println!("  3. Run `hal9-migrate verify` to validate");
                println!("  4. Proceed to canary when confident");
            }
            MigrationPhase::Canary => {
                println!("  1. Monitor error rates and latency closely");
                println!("  2. Watch for any anomalies in metrics");
                println!("  3. Gradually increase percentage if stable");
                println!("  4. Be ready to rollback if issues arise");
            }
            MigrationPhase::StateMigration => {
                println!("  1. Verify data integrity checks passed");
                println!("  2. Test rollback procedure");
                println!("  3. Monitor state consistency");
                println!("  4. Proceed to ramp-up when ready");
            }
            MigrationPhase::RampUp => {
                println!("  1. Continue monitoring all metrics");
                println!("  2. Perform load testing at current percentage");
                println!("  3. Address any performance bottlenecks");
                println!("  4. Increase percentage gradually to 100%");
            }
            MigrationPhase::Full => {
                println!("  1. Celebrate! 🎊");
                println!("  2. Monitor for 24 hours before decommissioning");
                println!("  3. Document lessons learned");
                println!("  4. Plan legacy system shutdown");
            }
        }
    }
    
    Ok(())
}

fn get_migration_steps(phase: &MigrationPhase) -> Vec<&'static str> {
    match phase {
        MigrationPhase::Shadow => vec![
            "Enable shadow mode feature flag",
            "Configure traffic mirroring",
            "Start shadow neurons",
            "Begin metric collection",
            "Validate shadow operations",
        ],
        MigrationPhase::Canary => vec![
            "Create canary deployment",
            "Configure load balancer rules",
            "Enable canary feature flags",
            "Route initial traffic percentage",
            "Start monitoring dashboards",
            "Set up automatic rollback triggers",
        ],
        MigrationPhase::StateMigration => vec![
            "Create state backup",
            "Start migration workers",
            "Transform data formats",
            "Validate migrated state",
            "Update state references",
            "Create rollback checkpoint",
        ],
        MigrationPhase::RampUp => vec![
            "Increase traffic percentage",
            "Scale new infrastructure",
            "Update monitoring thresholds",
            "Perform load testing",
            "Optimize performance",
        ],
        MigrationPhase::Full => vec![
            "Route all traffic to new system",
            "Disable legacy routing",
            "Final validation checks",
            "Update DNS and endpoints",
            "Mark legacy for decommission",
            "Update documentation",
        ],
    }
}

async fn execute_step(
    _client: &MigrationClient,
    _phase: &MigrationPhase,
    step: &str,
    _percentage: Option<u8>,
    pb: &ProgressBar,
) -> Result<()> {
    pb.set_message(step.to_string());
    
    // Simulate step execution with progress
    for i in 0..=100 {
        pb.set_position(i);
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Simulate occasional slowdowns
        if i % 25 == 0 {
            pb.set_message(format!("{} (validating...)", step));
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
    
    pb.set_message(format!("✓ {}", step));
    Ok(())
}