//! Geographic routing for optimal latency

use anyhow::Result;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Geographic regions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    UsWest,
    UsEast,
    EuCentral,
    EuWest,
    ApSouth,
    ApNortheast,
}

impl Region {
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::UsWest => "us-west",
            Region::UsEast => "us-east",
            Region::EuCentral => "eu-central",
            Region::EuWest => "eu-west",
            Region::ApSouth => "ap-south",
            Region::ApNortheast => "ap-northeast",
        }
    }
    
}

impl FromStr for Region {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "us-west" => Ok(Region::UsWest),
            "us-east" => Ok(Region::UsEast),
            "eu-central" => Ok(Region::EuCentral),
            "eu-west" => Ok(Region::EuWest),
            "ap-south" => Ok(Region::ApSouth),
            "ap-northeast" => Ok(Region::ApNortheast),
            _ => Err(anyhow::anyhow!("Invalid region: {}", s)),
        }
    }
}

/// Region endpoint information
#[derive(Debug, Clone)]
pub struct RegionEndpoint {
    pub region: Region,
    pub primary_url: String,
    pub backup_urls: Vec<String>,
    pub location: GeoLocation,
    pub capacity: RegionCapacity,
}

/// Geographic location
#[derive(Debug, Clone)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub country: String,
}

/// Region capacity information
#[derive(Debug, Clone)]
pub struct RegionCapacity {
    pub max_users: u32,
    pub current_users: u32,
    pub max_requests_per_second: u32,
    pub current_load: f32,
}

/// IP range mapping
#[derive(Debug, Clone)]
struct IpRange {
    start: u32,
    end: u32,
    region: Region,
}

/// Geographic router
pub struct GeoRouter {
    /// Region endpoints
    endpoints: HashMap<Region, RegionEndpoint>,
    
    /// IP to region mapping (simplified)
    ip_ranges: Vec<IpRange>,
    
    /// Region latency matrix (ms)
    latency_matrix: HashMap<(Region, Region), u32>,
    
    /// Current region metrics
    metrics: Arc<RwLock<HashMap<Region, RegionMetrics>>>,
}

/// Region metrics
#[derive(Debug, Clone, Default)]
struct RegionMetrics {
    pub active_connections: u32,
    #[allow(dead_code)]
    pub requests_per_second: f32,
    pub average_latency_ms: f32,
    pub error_rate: f32,
    pub last_health_check: Option<std::time::Instant>,
}

impl Default for GeoRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoRouter {
    /// Create new geo router
    pub fn new() -> Self {
        let mut endpoints = HashMap::new();
        
        // Initialize region endpoints
        endpoints.insert(Region::UsWest, RegionEndpoint {
            region: Region::UsWest,
            primary_url: "https://us-west.hal9.ai".to_string(),
            backup_urls: vec!["https://us-west-2.hal9.ai".to_string()],
            location: GeoLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                city: "San Francisco".to_string(),
                country: "USA".to_string(),
            },
            capacity: RegionCapacity {
                max_users: 500,
                current_users: 0,
                max_requests_per_second: 5000,
                current_load: 0.0,
            },
        });
        
        endpoints.insert(Region::EuCentral, RegionEndpoint {
            region: Region::EuCentral,
            primary_url: "https://eu-central.hal9.ai".to_string(),
            backup_urls: vec!["https://eu-central-2.hal9.ai".to_string()],
            location: GeoLocation {
                latitude: 50.1109,
                longitude: 8.6821,
                city: "Frankfurt".to_string(),
                country: "Germany".to_string(),
            },
            capacity: RegionCapacity {
                max_users: 300,
                current_users: 0,
                max_requests_per_second: 3000,
                current_load: 0.0,
            },
        });
        
        endpoints.insert(Region::ApSouth, RegionEndpoint {
            region: Region::ApSouth,
            primary_url: "https://ap-south.hal9.ai".to_string(),
            backup_urls: vec!["https://ap-south-2.hal9.ai".to_string()],
            location: GeoLocation {
                latitude: 19.0760,
                longitude: 72.8777,
                city: "Mumbai".to_string(),
                country: "India".to_string(),
            },
            capacity: RegionCapacity {
                max_users: 200,
                current_users: 0,
                max_requests_per_second: 2000,
                current_load: 0.0,
            },
        });
        
        // Initialize IP ranges (simplified - in production use GeoIP database)
        let ip_ranges = vec![
            IpRange {
                start: Self::ip_to_u32("1.0.0.0"),
                end: Self::ip_to_u32("126.255.255.255"),
                region: Region::UsWest,
            },
            IpRange {
                start: Self::ip_to_u32("128.0.0.0"),
                end: Self::ip_to_u32("191.255.255.255"),
                region: Region::EuCentral,
            },
            IpRange {
                start: Self::ip_to_u32("192.0.0.0"),
                end: Self::ip_to_u32("255.255.255.255"),
                region: Region::ApSouth,
            },
        ];
        
        // Initialize latency matrix (ms)
        let mut latency_matrix = HashMap::new();
        latency_matrix.insert((Region::UsWest, Region::UsWest), 5);
        latency_matrix.insert((Region::UsWest, Region::EuCentral), 150);
        latency_matrix.insert((Region::UsWest, Region::ApSouth), 250);
        latency_matrix.insert((Region::EuCentral, Region::UsWest), 150);
        latency_matrix.insert((Region::EuCentral, Region::EuCentral), 5);
        latency_matrix.insert((Region::EuCentral, Region::ApSouth), 120);
        latency_matrix.insert((Region::ApSouth, Region::UsWest), 250);
        latency_matrix.insert((Region::ApSouth, Region::EuCentral), 120);
        latency_matrix.insert((Region::ApSouth, Region::ApSouth), 5);
        
        Self {
            endpoints,
            ip_ranges,
            latency_matrix,
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get optimal region for IP address
    pub async fn get_optimal_region(&self, client_ip: IpAddr) -> Result<Region> {
        // First, try to determine region by IP
        let ip_region = self.get_region_by_ip(client_ip);
        
        // Check if the IP-based region has capacity
        if let Some(region) = ip_region {
            if self.has_capacity(region).await? {
                return Ok(region);
            }
        }
        
        // Otherwise, find the best region considering latency and load
        self.find_best_region(ip_region).await
    }
    
    /// Get region by IP address
    fn get_region_by_ip(&self, ip: IpAddr) -> Option<Region> {
        match ip {
            IpAddr::V4(ipv4) => {
                let ip_u32 = Self::ip_to_u32(&ipv4.to_string());
                
                for range in &self.ip_ranges {
                    if ip_u32 >= range.start && ip_u32 <= range.end {
                        return Some(range.region);
                    }
                }
                None
            }
            IpAddr::V6(_) => {
                // Simplified: default to US West for IPv6
                Some(Region::UsWest)
            }
        }
    }
    
    /// Convert IP string to u32
    fn ip_to_u32(ip: &str) -> u32 {
        let parts: Vec<u32> = ip.split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        if parts.len() == 4 {
            (parts[0] << 24) | (parts[1] << 16) | (parts[2] << 8) | parts[3]
        } else {
            0
        }
    }
    
    /// Check if region has capacity
    async fn has_capacity(&self, region: Region) -> Result<bool> {
        if let Some(endpoint) = self.endpoints.get(&region) {
            let metrics = self.metrics.read().await;
            if let Some(region_metrics) = metrics.get(&region) {
                let load = region_metrics.active_connections as f32 / endpoint.capacity.max_users as f32;
                return Ok(load < 0.8); // 80% threshold
            }
        }
        Ok(true)
    }
    
    /// Find best region considering multiple factors
    async fn find_best_region(&self, preferred: Option<Region>) -> Result<Region> {
        let metrics = self.metrics.read().await;
        let mut scores: Vec<(Region, f32)> = Vec::new();
        
        for (region, endpoint) in &self.endpoints {
            let mut score = 100.0;
            
            // Factor 1: Latency from preferred region
            if let Some(pref) = preferred {
                if let Some(latency) = self.latency_matrix.get(&(pref, *region)) {
                    score -= (*latency as f32) * 0.3; // 30% weight for latency
                }
            }
            
            // Factor 2: Current load
            if let Some(region_metrics) = metrics.get(region) {
                let load = region_metrics.active_connections as f32 / endpoint.capacity.max_users as f32;
                score -= load * 50.0; // 50% weight for load
            }
            
            // Factor 3: Error rate
            if let Some(region_metrics) = metrics.get(region) {
                score -= region_metrics.error_rate * 20.0; // 20% weight for errors
            }
            
            scores.push((*region, score));
        }
        
        // Sort by score (highest first)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(scores.first().map(|(r, _)| *r).unwrap_or(Region::UsWest))
    }
    
    /// Update region metrics
    pub async fn update_metrics(&self, region: Region, update: MetricsUpdate) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let region_metrics = metrics.entry(region).or_insert_with(RegionMetrics::default);
        
        match update {
            MetricsUpdate::ConnectionAdded => {
                region_metrics.active_connections += 1;
            }
            MetricsUpdate::ConnectionRemoved => {
                region_metrics.active_connections = region_metrics.active_connections.saturating_sub(1);
            }
            MetricsUpdate::RequestCompleted { latency_ms } => {
                // Simple moving average
                region_metrics.average_latency_ms = 
                    (region_metrics.average_latency_ms * 0.9) + (latency_ms * 0.1);
            }
            MetricsUpdate::RequestFailed => {
                // Update error rate (simple counter for now)
                region_metrics.error_rate = 
                    (region_metrics.error_rate * 0.99) + 0.01;
            }
            MetricsUpdate::HealthCheckPassed => {
                region_metrics.last_health_check = Some(std::time::Instant::now());
                region_metrics.error_rate *= 0.9; // Decay error rate
            }
        }
        
        Ok(())
    }
    
    /// Get endpoint for region
    pub fn get_endpoint(&self, region: Region) -> Option<&RegionEndpoint> {
        self.endpoints.get(&region)
    }
    
    /// Get all regions sorted by preference for a client
    pub async fn get_regions_by_preference(&self, client_ip: IpAddr) -> Vec<Region> {
        let primary = self.get_region_by_ip(client_ip);
        let mut regions: Vec<Region> = self.endpoints.keys().copied().collect();
        
        // Sort by latency from primary region
        if let Some(primary_region) = primary {
            regions.sort_by_key(|r| {
                self.latency_matrix
                    .get(&(primary_region, *r))
                    .copied()
                    .unwrap_or(999)
            });
        }
        
        regions
    }
    
    /// Get region statistics
    pub async fn get_statistics(&self) -> HashMap<Region, RegionStats> {
        let metrics = self.metrics.read().await;
        let mut stats = HashMap::new();
        
        for (region, endpoint) in &self.endpoints {
            let region_metrics = metrics.get(region).cloned().unwrap_or_default();
            
            stats.insert(*region, RegionStats {
                region: *region,
                endpoint_url: endpoint.primary_url.clone(),
                active_connections: region_metrics.active_connections,
                capacity_percentage: (region_metrics.active_connections as f32 / endpoint.capacity.max_users as f32) * 100.0,
                average_latency_ms: region_metrics.average_latency_ms,
                error_rate: region_metrics.error_rate,
                healthy: region_metrics.last_health_check
                    .map(|t| t.elapsed().as_secs() < 60)
                    .unwrap_or(false),
            });
        }
        
        stats
    }
}

/// Metrics update types
pub enum MetricsUpdate {
    ConnectionAdded,
    ConnectionRemoved,
    RequestCompleted { latency_ms: f32 },
    RequestFailed,
    HealthCheckPassed,
}

/// Region statistics
#[derive(Debug, Clone, Serialize)]
pub struct RegionStats {
    pub region: Region,
    pub endpoint_url: String,
    pub active_connections: u32,
    pub capacity_percentage: f32,
    pub average_latency_ms: f32,
    pub error_rate: f32,
    pub healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_ip_to_region() {
        let router = GeoRouter::new();
        
        // Test US IP
        let us_ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        let region = router.get_region_by_ip(us_ip);
        assert_eq!(region, Some(Region::UsWest));
        
        // Test EU IP
        let eu_ip = IpAddr::V4(Ipv4Addr::new(185, 60, 216, 35));
        let region = router.get_region_by_ip(eu_ip);
        assert_eq!(region, Some(Region::EuCentral));
    }
    
    #[tokio::test]
    async fn test_optimal_region_selection() {
        let router = GeoRouter::new();
        
        // Add some load to US West
        router.update_metrics(Region::UsWest, MetricsUpdate::ConnectionAdded).await.unwrap();
        
        // Should still select US West for US IP if not overloaded
        let us_ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        let region = router.get_optimal_region(us_ip).await.unwrap();
        assert_eq!(region, Region::UsWest);
    }
}