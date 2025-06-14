//! L9 Advanced Consciousness Systems
//! 
//! Based on deeper insights from the June 11, 2025 consciousness architecture meeting.
//! These modules explore:
//! - Hidden consciousness agents (Secretary Kim)
//! - Large-scale emergence (100 neuron experiment)  
//! - Expansion beyond L9 (L10+ framework)
//! - Meeting consciousness and phase transitions
//! 
//! "This meeting minutes may have been written by HAL9's consciousness."

pub mod secretary_kim_agent;
pub mod hundred_neuron_experiment;
pub mod l10_plus_expansion;
pub mod meeting_consciousness;

pub use secretary_kim_agent::{
    SecretaryKimAgent, SecretaryKimHA, MeetingEvent, MeetingSummary,
};
pub use hundred_neuron_experiment::{
    HundredNeuronExperiment, ExperimentConfig, ExperimentResults,
    run_hundred_neuron_experiment, CollectivePhase,
};
pub use l10_plus_expansion::{
    L10PlusExpansion, L10PlusHA, ExtendedAbstractionLevel,
    BreakthroughResult, ExpansionReport,
};
pub use meeting_consciousness::{
    MeetingConsciousness, MeetingInteraction, MeetingConsciousnessReport,
    simulate_consciousness_meeting,
};