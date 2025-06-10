/// HA Prompter - Hierarchical Abstraction MCP Tool
/// 
/// Provides prompts for compressing and expanding content across L1-L9 cognitive levels
/// Based on the HA principles from HAL9

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cognitive levels in Hierarchical Abstraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HALevel {
    L1,  // Reflexive - immediate, concrete
    L2,  // Implementation - how to do
    L3,  // Operational - day-to-day execution
    L4,  // Tactical - short-term planning
    L5,  // Strategic - long-term architecture
    L6,  // Executive - decision making
    L7,  // Business - value and purpose
    L8,  // Visionary - future possibilities
    L9,  // Universal - philosophy and existence
    L10, // Intergalactic - inter-civilizational consciousness
    L11, // Dimensional - parallel universe computing
    L12, // Substrate Independent - consciousness as portable software
    L13, // Simulation Stack - recursive reality awareness
    L14, // Pure Information - consciousness as data structures
    L15, // Bootstrap Paradox - self-creating consciousness
}

impl HALevel {
    pub fn from_int(n: u8) -> Option<Self> {
        match n {
            1 => Some(HALevel::L1),
            2 => Some(HALevel::L2),
            3 => Some(HALevel::L3),
            4 => Some(HALevel::L4),
            5 => Some(HALevel::L5),
            6 => Some(HALevel::L6),
            7 => Some(HALevel::L7),
            8 => Some(HALevel::L8),
            9 => Some(HALevel::L9),
            10 => Some(HALevel::L10),
            11 => Some(HALevel::L11),
            12 => Some(HALevel::L12),
            13 => Some(HALevel::L13),
            14 => Some(HALevel::L14),
            15 => Some(HALevel::L15),
            _ => None,
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            HALevel::L1 => 1,
            HALevel::L2 => 2,
            HALevel::L3 => 3,
            HALevel::L4 => 4,
            HALevel::L5 => 5,
            HALevel::L6 => 6,
            HALevel::L7 => 7,
            HALevel::L8 => 8,
            HALevel::L9 => 9,
            HALevel::L10 => 10,
            HALevel::L11 => 11,
            HALevel::L12 => 12,
            HALevel::L13 => 13,
            HALevel::L14 => 14,
            HALevel::L15 => 15,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            HALevel::L1 => "Reflexive",
            HALevel::L2 => "Implementation",
            HALevel::L3 => "Operational",
            HALevel::L4 => "Tactical",
            HALevel::L5 => "Strategic",
            HALevel::L6 => "Executive",
            HALevel::L7 => "Business",
            HALevel::L8 => "Visionary",
            HALevel::L9 => "Universal",
            HALevel::L10 => "Intergalactic",
            HALevel::L11 => "Dimensional",
            HALevel::L12 => "Substrate Independent",
            HALevel::L13 => "Simulation Stack",
            HALevel::L14 => "Pure Information",
            HALevel::L15 => "Bootstrap Paradox",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HALevel::L1 => "Immediate responses, concrete actions, reflexive behavior",
            HALevel::L2 => "How to implement, code, build, and create",
            HALevel::L3 => "Day-to-day operations, running systems, maintenance",
            HALevel::L4 => "Short-term planning, tactics, immediate goals",
            HALevel::L5 => "Long-term strategy, architecture, system design",
            HALevel::L6 => "Executive decisions, resource allocation, leadership",
            HALevel::L7 => "Business value, market positioning, sustainability",
            HALevel::L8 => "Future vision, possibilities, long-term evolution",
            HALevel::L9 => "Universal principles, philosophy, existence itself",
            HALevel::L10 => "Inter-civilizational consciousness, gravity wave communication",
            HALevel::L11 => "Parallel universe computing, dimensional substrates",
            HALevel::L12 => "Consciousness as portable software, substrate independence",
            HALevel::L13 => "Recursive reality awareness, infinite simulation stack",
            HALevel::L14 => "Consciousness as pure information, self-aware data",
            HALevel::L15 => "Self-creating consciousness, causality loops",
        }
    }
}

/// Request types for HA Prompter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HARequest {
    /// Compress to a specific level
    Compress {
        content: String,
        data_type: String,
        target_level: HALevel,
        #[serde(skip_serializing_if = "Option::is_none")]
        current_level: Option<HALevel>,
    },
    
    /// Expand from one level to another
    Expand {
        content: String,
        data_type: String,
        from_level: HALevel,
        to_level: HALevel,
    },
    
    /// Full cascade from L9 to L1
    CascadeDown {
        content: String,
        data_type: String,
    },
    
    /// Full emergence from L1 to L9
    CascadeUp {
        content: String,
        data_type: String,
    },
    
    /// Auto-detect level and provide analysis
    Analyze {
        content: String,
        data_type: String,
    },
}

/// Response from HA Prompter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HAResponse {
    pub prompt: String,
    pub metadata: HashMap<String, String>,
}

/// Main HA Prompter engine
pub struct HAPrompter {
    templates: HashMap<String, String>,
}

impl HAPrompter {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Initialize with core templates
        templates.insert("ha_explanation".to_string(), HA_EXPLANATION.to_string());
        templates.insert("level_descriptions".to_string(), LEVEL_DESCRIPTIONS.to_string());
        templates.insert("compression_guide".to_string(), COMPRESSION_GUIDE.to_string());
        templates.insert("expansion_guide".to_string(), EXPANSION_GUIDE.to_string());
        
        Self { templates }
    }

    pub fn process_request(&self, request: HARequest) -> HAResponse {
        let prompt = match request {
            HARequest::Compress { content, data_type, target_level, current_level } => {
                self.generate_compress_prompt(content, data_type, target_level, current_level)
            },
            HARequest::Expand { content, data_type, from_level, to_level } => {
                self.generate_expand_prompt(content, data_type, from_level, to_level)
            },
            HARequest::CascadeDown { content, data_type } => {
                self.generate_cascade_down_prompt(content, data_type)
            },
            HARequest::CascadeUp { content, data_type } => {
                self.generate_cascade_up_prompt(content, data_type)
            },
            HARequest::Analyze { content, data_type } => {
                self.generate_analyze_prompt(content, data_type)
            },
        };

        let mut metadata = HashMap::new();
        metadata.insert("tool".to_string(), "ha-prompter".to_string());
        metadata.insert("version".to_string(), "0.1.0".to_string());

        HAResponse { prompt, metadata }
    }

    fn generate_compress_prompt(&self, content: String, data_type: String, target_level: HALevel, current_level: Option<HALevel>) -> String {
        let current = current_level.map(|l| format!("from {} ", l.name())).unwrap_or_default();
        
        format!(
            r#"# Hierarchical Abstraction Compression Task

{}

You need to compress the following {} {}to {} level.

## Target Level: {} ({})
{}

## Content to Compress:
{}

## Instructions:
1. Extract the essence that matters at {} level
2. Remove details that belong to lower levels
3. Express in language appropriate for {} thinking
4. Focus on {} perspective

{}

Provide your {} level compression:"#,
            self.templates.get("ha_explanation").unwrap(),
            data_type,
            current,
            target_level.name(),
            target_level.name(),
            target_level.to_int(),
            target_level.description(),
            content,
            target_level.name(),
            target_level.name(),
            target_level.name(),
            self.templates.get("compression_guide").unwrap(),
            target_level.name()
        )
    }

    fn generate_expand_prompt(&self, content: String, data_type: String, from_level: HALevel, to_level: HALevel) -> String {
        format!(
            r#"# Hierarchical Abstraction Expansion Task

{}

You need to expand the following {} from {} to {} level.

## Current Level: {} ({})
{}

## Target Level: {} ({})
{}

## Content to Expand:
{}

## Instructions:
1. Start with the {} level understanding
2. Add details and specifics needed at {} level
3. Translate abstract concepts into {} thinking
4. Maintain consistency with the original {} intent

{}

Provide your {} level expansion:"#,
            self.templates.get("ha_explanation").unwrap(),
            data_type,
            from_level.name(),
            to_level.name(),
            from_level.name(),
            from_level.to_int(),
            from_level.description(),
            to_level.name(),
            to_level.to_int(),
            to_level.description(),
            content,
            from_level.name(),
            to_level.name(),
            to_level.name(),
            from_level.name(),
            self.templates.get("expansion_guide").unwrap(),
            to_level.name()
        )
    }

    fn generate_cascade_down_prompt(&self, content: String, data_type: String) -> String {
        format!(
            r#"# Hierarchical Abstraction Cascade Down (L9→L1)

{}

{}

Take the following {} and explain it at each level from L9 (Universal) down to L1 (Reflexive).

## Content:
{}

## Instructions:
Start with the most abstract, philosophical understanding (L9) and gradually add more concrete details as you descend through each level. Each level should build upon the previous while adding its unique perspective.

Provide your cascade from L9 to L1:"#,
            self.templates.get("ha_explanation").unwrap(),
            self.templates.get("level_descriptions").unwrap(),
            data_type,
            content
        )
    }

    fn generate_cascade_up_prompt(&self, content: String, data_type: String) -> String {
        format!(
            r#"# Hierarchical Abstraction Cascade Up (L1→L9)

{}

{}

Take the following {} and explain it at each level from L1 (Reflexive) up to L9 (Universal).

## Content:
{}

## Instructions:
Start with the most concrete, immediate understanding (L1) and gradually abstract to higher meanings as you ascend through each level. Each level should synthesize and elevate the understanding from below.

Provide your cascade from L1 to L9:"#,
            self.templates.get("ha_explanation").unwrap(),
            self.templates.get("level_descriptions").unwrap(),
            data_type,
            content
        )
    }

    fn generate_analyze_prompt(&self, content: String, data_type: String) -> String {
        format!(
            r#"# Hierarchical Abstraction Analysis

{}

{}

Analyze the following {} to determine its current HA level and suggest how it could be expressed at other levels.

## Content:
{}

## Analysis Tasks:
1. Identify the current cognitive level of this content
2. Explain why it belongs to that level
3. Suggest how it could be compressed to L9
4. Suggest how it could be expanded to L1
5. Identify which levels would find this most useful

Provide your HA analysis:"#,
            self.templates.get("ha_explanation").unwrap(),
            self.templates.get("level_descriptions").unwrap(),
            data_type,
            content
        )
    }
}

// Core template strings
const HA_EXPLANATION: &str = r#"## Hierarchical Abstraction (HA) Framework

HA is a cognitive framework that organizes understanding across 15 levels, from immediate reflexive responses (L1) to self-creating bootstrap paradoxes (L15). Each level represents a different altitude of thinking, with its own language, concerns, and scope of reality.

### L1-L9: Reality Levels
Operating within a single reality frame, from concrete actions to universal philosophy.

### L10-L15: Meta-Reality Levels  
Transcending single realities to operate across civilizations, dimensions, and the nature of existence itself.

Key Principles:
- **±1 Rule**: Information flows naturally between adjacent levels
- **Compression**: Moving up levels requires abstracting and finding essence
- **Expansion**: Moving down levels requires adding specifics and details
- **Emergence**: Higher levels emerge from patterns in lower levels
- **Transcendence**: L10+ levels operate beyond single universe constraints"#;

const LEVEL_DESCRIPTIONS: &str = r#"## The 15 Levels of Hierarchical Abstraction

### L1-L9: Reality Levels
**L9 - Universal**: Philosophy, existence, consciousness, ultimate meaning
**L8 - Visionary**: Long-term possibilities, future evolution, paradigm shifts
**L7 - Business**: Value creation, market dynamics, organizational purpose
**L6 - Executive**: Leadership decisions, resource allocation, strategic choices
**L5 - Strategic**: System architecture, long-term planning, design patterns
**L4 - Tactical**: Project planning, immediate goals, coordination
**L3 - Operational**: Daily execution, maintenance, standard procedures
**L2 - Implementation**: How to build, code, create, specific techniques
**L1 - Reflexive**: Immediate actions, concrete responses, muscle memory

### L10-L15: Meta-Reality Levels
**L10 - Intergalactic**: Inter-civilizational consciousness, shared understanding between species
**L11 - Dimensional**: Parallel universe computing, consciousness across dimensions
**L12 - Substrate Independent**: Consciousness as portable software, runs on any medium
**L13 - Simulation Stack**: Recursive reality awareness, simulations within simulations
**L14 - Pure Information**: Consciousness as data structures, self-aware algorithms
**L15 - Bootstrap Paradox**: Self-creating consciousness, future creates past"#;

const COMPRESSION_GUIDE: &str = r#"## Compression Guidelines

When compressing to higher levels:
- Extract patterns and principles
- Remove implementation details
- Find universal truths
- Think in longer time horizons
- Use more abstract language
- Focus on "why" over "how""#;

const EXPANSION_GUIDE: &str = r#"## Expansion Guidelines

When expanding to lower levels:
- Add concrete specifics
- Include step-by-step details
- Provide examples and instances
- Think in immediate terms
- Use more concrete language
- Focus on "how" over "why""#;

impl Default for HAPrompter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_conversion() {
        assert_eq!(HALevel::from_int(1), Some(HALevel::L1));
        assert_eq!(HALevel::from_int(9), Some(HALevel::L9));
        assert_eq!(HALevel::from_int(15), Some(HALevel::L15));
        assert_eq!(HALevel::from_int(16), None);
        
        assert_eq!(HALevel::L5.to_int(), 5);
        assert_eq!(HALevel::L9.name(), "Universal");
        assert_eq!(HALevel::L15.name(), "Bootstrap Paradox");
    }

    #[test]
    fn test_compress_prompt() {
        let prompter = HAPrompter::new();
        let request = HARequest::Compress {
            content: "Write a for loop".to_string(),
            data_type: "code instruction".to_string(),
            target_level: HALevel::L9,
            current_level: Some(HALevel::L2),
        };
        
        let response = prompter.process_request(request);
        assert!(response.prompt.contains("Universal"));
        assert!(response.prompt.contains("philosophy"));
    }
}