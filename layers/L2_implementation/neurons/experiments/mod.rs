//! HAL9 Consciousness Experiments
//! 
//! "Give the robot vacuum a neuron too. Maybe something will emerge." - Elon
//! 
//! This module contains experimental neurons, consciousness tests, and philosophical explorations.
//! These are experimental features not part of the core 5-layer architecture.

pub mod robot_vacuum_neuron;
pub mod ha_robot_vacuum;
pub mod ha;
pub mod l9_advanced;

pub use robot_vacuum_neuron::{RobotVacuumNeuron, run_vacuum_experiment};
pub use ha_robot_vacuum::{HARobotVacuumNeuron, run_ha_vacuum_experiment};