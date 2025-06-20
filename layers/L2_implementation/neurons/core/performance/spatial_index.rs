//! Spatial indexing for efficient neuron discovery
//!
//! Uses a grid-based spatial index for O(1) neighbor lookups
//! instead of O(n²) brute force search

use super::NeuronId;
use std::collections::{HashMap, HashSet};

/// 3D point representing neuron position
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuronPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl NeuronPoint {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Distance squared to another point (faster than distance)
    pub fn distance_squared(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
    
    /// Actual distance to another point
    pub fn distance(&self, other: &Self) -> f32 {
        self.distance_squared(other).sqrt()
    }
}

/// Grid cell coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridCell {
    x: i32,
    y: i32,
    z: i32,
}

/// Spatial index using a 3D grid
pub struct SpatialIndex {
    /// Grid cell size
    cell_size: f32,
    
    /// Neurons in each grid cell
    grid: HashMap<GridCell, HashSet<NeuronId>>,
    
    /// Neuron positions
    positions: HashMap<NeuronId, NeuronPoint>,
}

impl SpatialIndex {
    /// Create a new spatial index with given cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
            positions: HashMap::new(),
        }
    }
    
    /// Insert a neuron at a position
    pub fn insert(&mut self, id: NeuronId, position: NeuronPoint) {
        // Remove from old position if exists
        if let Some(old_pos) = self.positions.get(&id) {
            let old_cell = self.point_to_cell(old_pos);
            if let Some(neurons) = self.grid.get_mut(&old_cell) {
                neurons.remove(&id);
            }
        }
        
        // Insert at new position
        let cell = self.point_to_cell(&position);
        self.grid.entry(cell).or_default().insert(id);
        self.positions.insert(id, position);
    }
    
    /// Remove a neuron
    pub fn remove(&mut self, id: NeuronId) {
        if let Some(position) = self.positions.remove(&id) {
            let cell = self.point_to_cell(&position);
            if let Some(neurons) = self.grid.get_mut(&cell) {
                neurons.remove(&id);
                if neurons.is_empty() {
                    self.grid.remove(&cell);
                }
            }
        }
    }
    
    /// Find all neurons within radius of a point
    pub fn find_within_radius(&self, center: &NeuronPoint, radius: f32) -> Vec<NeuronId> {
        let mut results = Vec::new();
        let radius_squared = radius * radius;
        
        // Calculate grid cells to check
        let min_cell = self.point_to_cell(&NeuronPoint::new(
            center.x - radius,
            center.y - radius,
            center.z - radius,
        ));
        let max_cell = self.point_to_cell(&NeuronPoint::new(
            center.x + radius,
            center.y + radius,
            center.z + radius,
        ));
        
        // Check each cell in the bounding box
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let cell = GridCell { x, y, z };
                    if let Some(neurons) = self.grid.get(&cell) {
                        // Check each neuron in the cell
                        for &id in neurons {
                            if let Some(pos) = self.positions.get(&id) {
                                if pos.distance_squared(center) <= radius_squared {
                                    results.push(id);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Find K nearest neighbors
    pub fn find_k_nearest(&self, center: &NeuronPoint, k: usize) -> Vec<(NeuronId, f32)> {
        // Start with a small radius and expand
        let mut radius = self.cell_size;
        let mut candidates = Vec::new();
        
        while candidates.len() < k && radius < 1000.0 {
            candidates.clear();
            let neurons = self.find_within_radius(center, radius);
            
            for id in neurons {
                if let Some(pos) = self.positions.get(&id) {
                    let distance = pos.distance(center);
                    candidates.push((id, distance));
                }
            }
            
            radius *= 2.0;
        }
        
        // Sort by distance and take k nearest
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates.truncate(k);
        candidates
    }
    
    /// Get neuron position
    pub fn get_position(&self, id: NeuronId) -> Option<&NeuronPoint> {
        self.positions.get(&id)
    }
    
    /// Number of neurons in the index
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    
    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
    
    /// Clear the index
    pub fn clear(&mut self) {
        self.grid.clear();
        self.positions.clear();
    }
    
    /// Convert point to grid cell
    fn point_to_cell(&self, point: &NeuronPoint) -> GridCell {
        GridCell {
            x: (point.x / self.cell_size).floor() as i32,
            y: (point.y / self.cell_size).floor() as i32,
            z: (point.z / self.cell_size).floor() as i32,
        }
    }
}

/// Builder for spatial index with automatic cell size selection
pub struct SpatialIndexBuilder {
    neurons: Vec<(NeuronId, NeuronPoint)>,
}

impl Default for SpatialIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialIndexBuilder {
    pub fn new() -> Self {
        Self {
            neurons: Vec::new(),
        }
    }
    
    /// Add a neuron
    pub fn add(&mut self, id: NeuronId, position: NeuronPoint) -> &mut Self {
        self.neurons.push((id, position));
        self
    }
    
    /// Build the index with optimal cell size
    pub fn build(self) -> SpatialIndex {
        // Calculate optimal cell size based on neuron distribution
        let cell_size = self.calculate_optimal_cell_size();
        let mut index = SpatialIndex::new(cell_size);
        
        for (id, pos) in self.neurons {
            index.insert(id, pos);
        }
        
        index
    }
    
    /// Calculate optimal cell size for performance
    fn calculate_optimal_cell_size(&self) -> f32 {
        if self.neurons.is_empty() {
            return 1.0;
        }
        
        // Find bounding box
        let mut min = self.neurons[0].1;
        let mut max = self.neurons[0].1;
        
        for (_, pos) in &self.neurons[1..] {
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }
        
        // Calculate volume
        let volume = (max.x - min.x) * (max.y - min.y) * (max.z - min.z);
        
        // Optimal cell size: ~10 neurons per cell
        let target_cells = self.neurons.len() as f32 / 10.0;
        let cell_volume = volume / target_cells;
        cell_volume.powf(1.0 / 3.0).max(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spatial_index() {
        let mut index = SpatialIndex::new(1.0);
        
        let id1 = NeuronId::new(1);
        let id2 = NeuronId::new(2);
        let id3 = NeuronId::new(3);
        
        index.insert(id1, NeuronPoint::new(0.0, 0.0, 0.0));
        index.insert(id2, NeuronPoint::new(0.5, 0.5, 0.5));
        index.insert(id3, NeuronPoint::new(5.0, 5.0, 5.0));
        
        // Find within radius
        let nearby = index.find_within_radius(&NeuronPoint::new(0.0, 0.0, 0.0), 1.0);
        assert!(nearby.contains(&id1));
        assert!(nearby.contains(&id2));
        assert!(!nearby.contains(&id3));
        
        // Find k nearest
        let nearest = index.find_k_nearest(&NeuronPoint::new(0.0, 0.0, 0.0), 2);
        assert_eq!(nearest.len(), 2);
        assert_eq!(nearest[0].0, id1);
        assert_eq!(nearest[1].0, id2);
    }
}