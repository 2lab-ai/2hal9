//! Self-organization capabilities for autonomous structure formation

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::Result;
use super::*;

/// Self-organizing system that forms structures autonomously
pub struct SelfOrganizingSystem {
    units: HashMap<Uuid, OrganizationalUnit>,
    clusters: Vec<Cluster>,
    hierarchy: Option<Hierarchy>,
    rules: Vec<OrganizationRule>,
    energy_function: EnergyFunction,
}

#[derive(Debug, Clone)]
struct OrganizationalUnit {
    id: Uuid,
    position: Vec<f32>, // N-dimensional position in feature space
    features: HashMap<String, f32>,
    connections: HashSet<Uuid>,
    layer_affinity: HashMap<u8, f32>,
}

/// Organization rule that governs structure formation
struct OrganizationRule {
    name: String,
    condition: Box<dyn Fn(&SelfOrganizingSystem) -> bool + Send + Sync>,
    action: Box<dyn Fn(&mut SelfOrganizingSystem) -> Result<()> + Send + Sync>,
    priority: f32,
}

/// Energy function for system optimization
struct EnergyFunction {
    components: Vec<EnergyComponent>,
}

struct EnergyComponent {
    name: String,
    weight: f32,
    calculate: Box<dyn Fn(&SelfOrganizingSystem) -> f32 + Send + Sync>,
}

impl SelfOrganizingSystem {
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
            clusters: Vec::new(),
            hierarchy: None,
            rules: Self::default_rules(),
            energy_function: Self::default_energy_function(),
        }
    }
    
    fn default_rules() -> Vec<OrganizationRule> {
        vec![
            OrganizationRule {
                name: "cluster_formation".to_string(),
                condition: Box::new(|system| system.clusters.len() < system.units.len() / 10),
                action: Box::new(|system| {
                    system.form_clusters_internal()?;
                    Ok(())
                }),
                priority: 1.0,
            },
            OrganizationRule {
                name: "hierarchy_emergence".to_string(),
                condition: Box::new(|system| system.clusters.len() > 5 && system.hierarchy.is_none()),
                action: Box::new(|system| {
                    system.create_hierarchy_internal()?;
                    Ok(())
                }),
                priority: 0.8,
            },
            OrganizationRule {
                name: "connection_pruning".to_string(),
                condition: Box::new(|system| {
                    let avg_connections = system.units.values()
                        .map(|u| u.connections.len())
                        .sum::<usize>() as f32 / system.units.len() as f32;
                    avg_connections > 20.0
                }),
                action: Box::new(|system| {
                    system.prune_weak_connections()?;
                    Ok(())
                }),
                priority: 0.5,
            },
        ]
    }
    
    fn default_energy_function() -> EnergyFunction {
        EnergyFunction {
            components: vec![
                EnergyComponent {
                    name: "clustering_energy".to_string(),
                    weight: 1.0,
                    calculate: Box::new(|system| {
                        // Lower energy for well-separated clusters
                        let mut energy = 0.0;
                        for cluster in &system.clusters {
                            let cohesion = system.calculate_cluster_cohesion(cluster);
                            energy += 1.0 - cohesion;
                        }
                        energy
                    }),
                },
                EnergyComponent {
                    name: "hierarchy_energy".to_string(),
                    weight: 0.8,
                    calculate: Box::new(|system| {
                        // Lower energy for clear hierarchical structure
                        if let Some(hierarchy) = &system.hierarchy {
                            1.0 / (hierarchy.total_depth as f32)
                        } else {
                            10.0
                        }
                    }),
                },
                EnergyComponent {
                    name: "connectivity_energy".to_string(),
                    weight: 0.5,
                    calculate: Box::new(|system| {
                        // Balance between too few and too many connections
                        let avg_connections = system.units.values()
                            .map(|u| u.connections.len())
                            .sum::<usize>() as f32 / system.units.len() as f32;
                        (avg_connections - 10.0).abs()
                    }),
                },
            ],
        }
    }
    
    fn calculate_cluster_cohesion(&self, cluster: &Cluster) -> f32 {
        if cluster.members.len() < 2 {
            return 0.0;
        }
        
        let mut total_distance = 0.0;
        let mut pair_count = 0;
        
        for i in 0..cluster.members.len() {
            for j in (i + 1)..cluster.members.len() {
                if let (Some(unit_i), Some(unit_j)) = 
                    (self.units.get(&cluster.members[i]), self.units.get(&cluster.members[j])) {
                    total_distance += self.distance(unit_i, unit_j);
                    pair_count += 1;
                }
            }
        }
        
        if pair_count > 0 {
            1.0 / (1.0 + total_distance / pair_count as f32)
        } else {
            0.0
        }
    }
    
    fn distance(&self, unit1: &OrganizationalUnit, unit2: &OrganizationalUnit) -> f32 {
        // Euclidean distance in feature space
        unit1.position.iter()
            .zip(unit2.position.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
    
    fn form_clusters_internal(&mut self) -> Result<()> {
        // K-means clustering algorithm
        let k = (self.units.len() as f32).sqrt() as usize;
        let mut centroids = self.initialize_centroids(k);
        
        for _ in 0..100 { // Max iterations
            // Assign units to nearest centroid
            let mut assignments: HashMap<usize, Vec<Uuid>> = HashMap::new();
            
            for (unit_id, unit) in &self.units {
                let nearest_centroid = self.find_nearest_centroid(&unit.position, &centroids);
                assignments.entry(nearest_centroid)
                    .or_insert_with(Vec::new)
                    .push(*unit_id);
            }
            
            // Update centroids
            let mut new_centroids = vec![vec![0.0; centroids[0].len()]; k];
            for (centroid_idx, unit_ids) in assignments {
                if unit_ids.is_empty() {
                    continue;
                }
                
                for unit_id in &unit_ids {
                    if let Some(unit) = self.units.get(unit_id) {
                        for (i, &pos) in unit.position.iter().enumerate() {
                            new_centroids[centroid_idx][i] += pos;
                        }
                    }
                }
                
                for val in &mut new_centroids[centroid_idx] {
                    *val /= unit_ids.len() as f32;
                }
            }
            
            // Check convergence
            if self.centroids_converged(&centroids, &new_centroids) {
                break;
            }
            
            centroids = new_centroids;
        }
        
        // Create clusters from final assignments
        self.clusters.clear();
        let assignments = self.final_assignments(&centroids);
        
        for (idx, members) in assignments {
            if !members.is_empty() {
                self.clusters.push(Cluster {
                    id: Uuid::new_v4(),
                    members,
                    purpose: format!("cluster_{}", idx),
                    cohesion: 0.0, // Will be calculated
                });
            }
        }
        
        // Update cohesion scores
        for cluster in &mut self.clusters {
            cluster.cohesion = self.calculate_cluster_cohesion(cluster);
        }
        
        Ok(())
    }
    
    fn initialize_centroids(&self, k: usize) -> Vec<Vec<f32>> {
        // K-means++ initialization
        let mut centroids = Vec::new();
        let mut rng = rand::thread_rng();
        use rand::seq::SliceRandom;
        
        // Choose first centroid randomly
        if let Some(unit) = self.units.values().choose(&mut rng) {
            centroids.push(unit.position.clone());
        }
        
        // Choose remaining centroids with probability proportional to distance
        while centroids.len() < k && centroids.len() < self.units.len() {
            let mut distances = Vec::new();
            
            for unit in self.units.values() {
                let min_dist = centroids.iter()
                    .map(|c| self.position_distance(&unit.position, c))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                distances.push(min_dist);
            }
            
            // Choose next centroid
            // Simplified: just pick the farthest point
            if let Some((unit, _)) = self.units.values()
                .zip(distances.iter())
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
                centroids.push(unit.position.clone());
            }
        }
        
        centroids
    }
    
    fn find_nearest_centroid(&self, position: &[f32], centroids: &[Vec<f32>]) -> usize {
        centroids.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = self.position_distance(position, a);
                let dist_b = self.position_distance(position, b);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
    
    fn position_distance(&self, pos1: &[f32], pos2: &[f32]) -> f32 {
        pos1.iter()
            .zip(pos2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
    
    fn centroids_converged(&self, old: &[Vec<f32>], new: &[Vec<f32>]) -> bool {
        old.iter()
            .zip(new.iter())
            .all(|(o, n)| self.position_distance(o, n) < 0.001)
    }
    
    fn final_assignments(&self, centroids: &[Vec<f32>]) -> HashMap<usize, Vec<Uuid>> {
        let mut assignments = HashMap::new();
        
        for (unit_id, unit) in &self.units {
            let nearest = self.find_nearest_centroid(&unit.position, centroids);
            assignments.entry(nearest)
                .or_insert_with(Vec::new)
                .push(*unit_id);
        }
        
        assignments
    }
    
    fn create_hierarchy_internal(&mut self) -> Result<()> {
        // Create hierarchy from clusters
        let mut levels = Vec::new();
        
        // Level 0: Individual units
        levels.push(HierarchyLevel {
            level: 0,
            units: self.units.keys().cloned().collect(),
            abstraction_degree: 0.0,
        });
        
        // Level 1: Clusters
        levels.push(HierarchyLevel {
            level: 1,
            units: self.clusters.iter().map(|c| c.id).collect(),
            abstraction_degree: 0.3,
        });
        
        // Higher levels through hierarchical clustering
        let mut current_clusters = self.clusters.clone();
        let mut level = 2;
        
        while current_clusters.len() > 1 && level < 5 {
            let merged_clusters = self.merge_clusters(&current_clusters);
            if merged_clusters.len() < current_clusters.len() {
                levels.push(HierarchyLevel {
                    level,
                    units: merged_clusters.iter().map(|c| c.id).collect(),
                    abstraction_degree: level as f32 * 0.25,
                });
                current_clusters = merged_clusters;
                level += 1;
            } else {
                break;
            }
        }
        
        self.hierarchy = Some(Hierarchy {
            levels,
            total_depth: level,
        });
        
        Ok(())
    }
    
    fn merge_clusters(&self, clusters: &[Cluster]) -> Vec<Cluster> {
        // Simple hierarchical clustering
        if clusters.len() <= 1 {
            return clusters.to_vec();
        }
        
        // Find two closest clusters
        let mut min_distance = f32::MAX;
        let mut merge_pair = (0, 0);
        
        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let dist = self.cluster_distance(&clusters[i], &clusters[j]);
                if dist < min_distance {
                    min_distance = dist;
                    merge_pair = (i, j);
                }
            }
        }
        
        // Merge the closest pair
        let mut new_clusters = Vec::new();
        for (idx, cluster) in clusters.iter().enumerate() {
            if idx != merge_pair.0 && idx != merge_pair.1 {
                new_clusters.push(cluster.clone());
            }
        }
        
        // Create merged cluster
        let mut merged_members = clusters[merge_pair.0].members.clone();
        merged_members.extend(&clusters[merge_pair.1].members);
        
        new_clusters.push(Cluster {
            id: Uuid::new_v4(),
            members: merged_members,
            purpose: format!("merged_{}_{}", clusters[merge_pair.0].purpose, clusters[merge_pair.1].purpose),
            cohesion: (clusters[merge_pair.0].cohesion + clusters[merge_pair.1].cohesion) / 2.0,
        });
        
        new_clusters
    }
    
    fn cluster_distance(&self, cluster1: &Cluster, cluster2: &Cluster) -> f32 {
        // Average linkage distance
        let mut total_distance = 0.0;
        let mut pair_count = 0;
        
        for member1 in &cluster1.members {
            for member2 in &cluster2.members {
                if let (Some(unit1), Some(unit2)) = (self.units.get(member1), self.units.get(member2)) {
                    total_distance += self.distance(unit1, unit2);
                    pair_count += 1;
                }
            }
        }
        
        if pair_count > 0 {
            total_distance / pair_count as f32
        } else {
            f32::MAX
        }
    }
    
    fn prune_weak_connections(&mut self) -> Result<()> {
        // Remove connections below threshold
        let threshold = 0.1;
        
        for unit in self.units.values_mut() {
            unit.connections.retain(|&conn_id| {
                // Would calculate connection strength
                true // Placeholder
            });
        }
        
        Ok(())
    }
}

#[async_trait]
impl SelfOrganizer for SelfOrganizingSystem {
    async fn form_clusters(&mut self) -> Result<Vec<Cluster>> {
        self.form_clusters_internal()?;
        Ok(self.clusters.clone())
    }
    
    async fn create_hierarchy(&mut self) -> Result<Hierarchy> {
        self.create_hierarchy_internal()?;
        Ok(self.hierarchy.clone().unwrap_or_else(|| Hierarchy {
            levels: vec![],
            total_depth: 0,
        }))
    }
    
    async fn evolve_topology(&mut self) -> Result<TopologyUpdate> {
        // Apply organization rules
        let mut applied_rules = Vec::new();
        
        for rule in &self.rules {
            if (rule.condition)(self) {
                (rule.action)(self)?;
                applied_rules.push(rule.name.clone());
            }
        }
        
        // Optimize based on energy function
        let current_energy = self.calculate_total_energy();
        
        // Try random perturbations
        let mut best_update = TopologyUpdate {
            added_connections: vec![],
            removed_connections: vec![],
            reorganized_clusters: vec![],
        };
        
        let mut best_energy = current_energy;
        
        // Simulated annealing
        let temperature = 1.0;
        for _ in 0..100 {
            let update = self.generate_random_update();
            let new_energy = self.calculate_energy_after_update(&update);
            
            if new_energy < best_energy || rand::random::<f32>() < ((best_energy - new_energy) / temperature).exp() {
                best_energy = new_energy;
                best_update = update;
            }
        }
        
        // Apply best update
        self.apply_update(&best_update)?;
        
        Ok(best_update)
    }
}

impl SelfOrganizingSystem {
    fn calculate_total_energy(&self) -> f32 {
        self.energy_function.components.iter()
            .map(|component| component.weight * (component.calculate)(self))
            .sum()
    }
    
    fn generate_random_update(&self) -> TopologyUpdate {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        TopologyUpdate {
            added_connections: if rng.gen_bool(0.3) {
                vec![(Uuid::new_v4(), Uuid::new_v4())] // Would use actual unit IDs
            } else {
                vec![]
            },
            removed_connections: if rng.gen_bool(0.3) {
                vec![(Uuid::new_v4(), Uuid::new_v4())] // Would use actual unit IDs
            } else {
                vec![]
            },
            reorganized_clusters: vec![],
        }
    }
    
    fn calculate_energy_after_update(&self, update: &TopologyUpdate) -> f32 {
        // Simplified: just return current energy with small perturbation
        self.calculate_total_energy() + rand::random::<f32>() * 0.1 - 0.05
    }
    
    fn apply_update(&mut self, update: &TopologyUpdate) -> Result<()> {
        // Apply topology changes
        for (from, to) in &update.added_connections {
            if let Some(unit) = self.units.get_mut(from) {
                unit.connections.insert(*to);
            }
        }
        
        for (from, to) in &update.removed_connections {
            if let Some(unit) = self.units.get_mut(from) {
                unit.connections.remove(to);
            }
        }
        
        Ok(())
    }
}