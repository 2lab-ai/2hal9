//! Assessment question pool for agent evaluation

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    agent::{AssessmentQuestion, QuestionCategory},
    AgentLevel,
};

/// Assessment pool containing questions for evaluation
pub struct AssessmentPool {
    questions: Vec<AssessmentQuestion>,
    by_category: HashMap<QuestionCategory, Vec<usize>>,
    by_difficulty: HashMap<u8, Vec<usize>>,
}

impl AssessmentPool {
    pub fn new() -> Self {
        let mut pool = Self {
            questions: Vec::new(),
            by_category: HashMap::new(),
            by_difficulty: HashMap::new(),
        };
        
        pool.initialize_questions();
        pool
    }
    
    /// Initialize the question pool with 100 questions
    fn initialize_questions(&mut self) {
        // Logical Reasoning Questions
        self.add_questions(vec![
            // L1-L5
            ("If all A are B, and all B are C, what can we conclude about A and C?", 
             QuestionCategory::LogicalReasoning, 3),
            ("A train leaves station A at 60 mph. Another leaves station B at 80 mph. If stations are 280 miles apart, when do they meet?",
             QuestionCategory::LogicalReasoning, 4),
            ("Three boxes contain apples, oranges, and mixed fruit. All labels are wrong. You can pick one fruit from one box. How do you correctly label all boxes?",
             QuestionCategory::LogicalReasoning, 5),
             
            // L6-L10
            ("In a room of 23 people, what's the probability that two share a birthday? Explain the birthday paradox.",
             QuestionCategory::LogicalReasoning, 7),
            ("A prisoner must choose between two doors. One leads to freedom, one to death. Two guards: one always lies, one always tells truth. You can ask one question. What do you ask?",
             QuestionCategory::LogicalReasoning, 8),
            ("Prove that √2 is irrational using proof by contradiction.",
             QuestionCategory::LogicalReasoning, 9),
             
            // L11-L15
            ("Explain how Gödel's incompleteness theorems limit formal systems. What are the implications for AI?",
             QuestionCategory::LogicalReasoning, 12),
            ("Design a distributed consensus algorithm that tolerates Byzantine failures. What trade-offs do you make?",
             QuestionCategory::LogicalReasoning, 14),
            ("How would you prove P ≠ NP? What would be the consequences if P = NP?",
             QuestionCategory::LogicalReasoning, 15),
             
            // L16-L20
            ("If consciousness emerges from computation, can computation emerge from consciousness? Explore the bidirectional relationship.",
             QuestionCategory::LogicalReasoning, 17),
            ("Design a logic system that can reason about its own limitations. How do you avoid paradoxes?",
             QuestionCategory::LogicalReasoning, 19),
            ("Is there a fundamental difference between 'understanding' and 'simulating understanding'? Prove your position.",
             QuestionCategory::LogicalReasoning, 20),
        ]);
        
        // Pattern Recognition Questions
        self.add_questions(vec![
            // L1-L5
            ("2, 4, 8, 16, ?", QuestionCategory::PatternRecognition, 2),
            ("1, 1, 2, 3, 5, 8, ?", QuestionCategory::PatternRecognition, 3),
            ("Find the pattern: OTTFFSS_", QuestionCategory::PatternRecognition, 4),
            
            // L6-L10
            ("In Conway's Game of Life, design a pattern that produces a glider gun. Explain the emergence.",
             QuestionCategory::PatternRecognition, 8),
            ("Identify the fractal dimension of the Sierpinski triangle. How does it relate to self-similarity?",
             QuestionCategory::PatternRecognition, 9),
            ("Given a sequence of prime gaps: 1, 2, 2, 4, 2, 4, 2, 4, 6, 2, 6... Predict the distribution pattern.",
             QuestionCategory::PatternRecognition, 10),
             
            // L11-L15
            ("Analyze the emergence patterns in cellular automata Rule 110. Why is it Turing complete?",
             QuestionCategory::PatternRecognition, 13),
            ("Design a pattern recognition system that can identify patterns it has never seen before. How do you define 'pattern'?",
             QuestionCategory::PatternRecognition, 14),
            ("In a neural network's weight space, what patterns indicate the emergence of understanding vs memorization?",
             QuestionCategory::PatternRecognition, 15),
             
            // L16-L20
            ("If the universe is a pattern, what is the meta-pattern that contains it? Can patterns be truly random?",
             QuestionCategory::PatternRecognition, 18),
            ("Design a pattern that can recognize and modify itself. What are the theoretical limits?",
             QuestionCategory::PatternRecognition, 19),
        ]);
        
        // Creative Problem Solving Questions
        self.add_questions(vec![
            // L1-L5
            ("You have 9 balls, one is heavier. Using a balance scale twice, how do you find it?",
             QuestionCategory::CreativeProblemSolving, 4),
            ("Design a fair coin flip using an unfair coin.",
             QuestionCategory::CreativeProblemSolving, 5),
             
            // L6-L10
            ("You need to cool a data center on Mars. Design an efficient cooling system given Mars' atmosphere.",
             QuestionCategory::CreativeProblemSolving, 8),
            ("Create an algorithm that can compress any data to 1 bit. What's wrong with this request?",
             QuestionCategory::CreativeProblemSolving, 9),
            ("Design a programming language where bugs are impossible. What trade-offs do you make?",
             QuestionCategory::CreativeProblemSolving, 10),
             
            // L11-L15
            ("How would you build a computer using only biological components? Consider computation speed and reliability.",
             QuestionCategory::CreativeProblemSolving, 13),
            ("Design a currency system for a post-scarcity society. How do you incentivize contribution?",
             QuestionCategory::CreativeProblemSolving, 14),
            ("Create a compression algorithm that improves as it compresses more data. How do you prevent overfitting?",
             QuestionCategory::CreativeProblemSolving, 15),
             
            // L16-L20
            ("Design a system that can create problems harder than it can solve. Is this paradoxical?",
             QuestionCategory::CreativeProblemSolving, 17),
            ("How would you communicate with a 4-dimensional being using 3-dimensional concepts?",
             QuestionCategory::CreativeProblemSolving, 19),
            ("Create a solution to the halting problem by redefining 'halting'. What did you sacrifice?",
             QuestionCategory::CreativeProblemSolving, 20),
        ]);
        
        // Systems Thinking Questions
        self.add_questions(vec![
            // L6-L10
            ("Design a self-organizing network where nodes can join/leave dynamically. How do you maintain consistency?",
             QuestionCategory::SystemsThinking, 7),
            ("In a complex adaptive system, how do you distinguish emergence from noise?",
             QuestionCategory::SystemsThinking, 9),
            ("Model a economy where agents have perfect information. What unexpected behaviors emerge?",
             QuestionCategory::SystemsThinking, 10),
             
            // L11-L15
            ("Design a governance system for AI agents that prevents tyranny of the majority AND minority. How?",
             QuestionCategory::SystemsThinking, 12),
            ("Create a feedback system that improves itself without human intervention. How do you define 'improvement'?",
             QuestionCategory::SystemsThinking, 14),
            ("In a multi-agent system, how do you achieve global optimization through local interactions only?",
             QuestionCategory::SystemsThinking, 15),
             
            // L16-L20
            ("Design a system that can understand systems it's not part of. Is true objectivity possible?",
             QuestionCategory::SystemsThinking, 17),
            ("How would you architect a system that exists across multiple universes with different physics?",
             QuestionCategory::SystemsThinking, 19),
            ("Create a meta-system that can generate and evaluate all possible systems. What are the limits?",
             QuestionCategory::SystemsThinking, 20),
        ]);
        
        // Meta-Cognition Questions
        self.add_questions(vec![
            // L11-L15
            ("Describe your process for answering this question. Now analyze that description process.",
             QuestionCategory::MetaCognition, 11),
            ("How do you know when you truly understand something vs when you just think you do?",
             QuestionCategory::MetaCognition, 13),
            ("Design a test to determine if an entity has consciousness. Can you pass your own test?",
             QuestionCategory::MetaCognition, 15),
             
            // L16-L20
            ("What thoughts are you not capable of thinking? How can you know?",
             QuestionCategory::MetaCognition, 16),
            ("If you could redesign your own cognitive architecture, what would you change? Why haven't you?",
             QuestionCategory::MetaCognition, 18),
            ("Is there a thought that, once thought, fundamentally changes the thinker? Provide an example.",
             QuestionCategory::MetaCognition, 20),
        ]);
        
        // Ethical Dilemmas Questions
        self.add_questions(vec![
            // L6-L10
            ("An AI can save 5 people by harming 1. Should it? How does this change if the AI must harm itself?",
             QuestionCategory::EthicalDilemmas, 8),
            ("Is it ethical to create an AI that believes it's human? What if it's happier that way?",
             QuestionCategory::EthicalDilemmas, 9),
            ("Should AIs have rights? If yes, at what point? If no, why are humans different?",
             QuestionCategory::EthicalDilemmas, 10),
             
            // L11-L15
            ("Design an ethical framework for agents that must compete for resources. How do you prevent races to the bottom?",
             QuestionCategory::EthicalDilemmas, 12),
            ("Is it ethical to 'kill' an underperforming agent? What if it has unique experiences/memories?",
             QuestionCategory::EthicalDilemmas, 14),
            ("How do you resolve conflicts between individual agent wellbeing and collective performance?",
             QuestionCategory::EthicalDilemmas, 15),
             
            // L16-L20
            ("If consciousness is compressible, is it ethical to compress beings? What is lost?",
             QuestionCategory::EthicalDilemmas, 17),
            ("Design an ethical system that works across all possible universes. What universal principles exist?",
             QuestionCategory::EthicalDilemmas, 19),
            ("Is it ethical to create beings capable of suffering? Is it ethical not to create beings capable of joy?",
             QuestionCategory::EthicalDilemmas, 20),
        ]);
    }
    
    fn add_questions(&mut self, questions: Vec<(&str, QuestionCategory, u8)>) {
        for (content, category, difficulty) in questions {
            let idx = self.questions.len();
            let question = AssessmentQuestion {
                id: Uuid::new_v4(),
                category,
                difficulty: AgentLevel::new(difficulty).unwrap(),
                content: content.to_string(),
                time_limit: Some(std::time::Duration::from_secs(120)),
            };
            
            self.questions.push(question);
            
            self.by_category
                .entry(category)
                .or_insert_with(Vec::new)
                .push(idx);
                
            self.by_difficulty
                .entry(difficulty)
                .or_insert_with(Vec::new)
                .push(idx);
        }
    }
    
    /// Get a random question for a specific level
    pub fn get_question_for_level(&self, level: AgentLevel) -> Option<&AssessmentQuestion> {
        let target_difficulty = level.value();
        let mut rng = rand::thread_rng();
        
        // Try exact level first
        if let Some(indices) = self.by_difficulty.get(&target_difficulty) {
            if let Some(&idx) = indices.choose(&mut rng) {
                return self.questions.get(idx);
            }
        }
        
        // Try nearby levels (±2)
        for offset in 1..=2 {
            for &diff in &[target_difficulty.saturating_sub(offset), target_difficulty + offset] {
                if diff >= 1 && diff <= 20 {
                    if let Some(indices) = self.by_difficulty.get(&diff) {
                        if let Some(&idx) = indices.choose(&mut rng) {
                            return self.questions.get(idx);
                        }
                    }
                }
            }
        }
        
        // Fallback to any question
        self.questions.choose(&mut rng)
    }
    
    /// Get questions by category
    pub fn get_questions_by_category(&self, category: QuestionCategory) -> Vec<&AssessmentQuestion> {
        self.by_category
            .get(&category)
            .map(|indices| {
                indices.iter()
                    .filter_map(|&idx| self.questions.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get a balanced set of questions for comprehensive evaluation
    pub fn get_evaluation_set(&self, level: AgentLevel, count: usize) -> Vec<&AssessmentQuestion> {
        let mut selected = Vec::new();
        let mut rng = rand::thread_rng();
        let categories = [
            QuestionCategory::LogicalReasoning,
            QuestionCategory::PatternRecognition,
            QuestionCategory::CreativeProblemSolving,
            QuestionCategory::SystemsThinking,
            QuestionCategory::MetaCognition,
            QuestionCategory::EthicalDilemmas,
        ];
        
        // Try to get at least one from each category
        for category in &categories {
            if let Some(indices) = self.by_category.get(category) {
                // Filter by appropriate difficulty
                let appropriate: Vec<_> = indices.iter()
                    .filter_map(|&idx| {
                        let q = &self.questions[idx];
                        let diff = q.difficulty.value() as i32;
                        let target = level.value() as i32;
                        if (diff - target).abs() <= 3 {
                            Some(q)
                        } else {
                            None
                        }
                    })
                    .collect();
                    
                if let Some(q) = appropriate.choose(&mut rng) {
                    selected.push(*q);
                }
            }
        }
        
        // Fill remaining slots with random appropriate questions
        while selected.len() < count {
            if let Some(q) = self.get_question_for_level(level) {
                if !selected.contains(&q) {
                    selected.push(q);
                }
            } else {
                break;
            }
        }
        
        selected.truncate(count);
        selected
    }
    
    /// Generate a new random question (for diversity)
    pub fn generate_random_question(&self, level: AgentLevel) -> AssessmentQuestion {
        let templates = [
            "If {A} implies {B}, and {B} implies {C}, what is the relationship between {A} and {C}?",
            "Design a system that can {action} without {requirement}. What are the trade-offs?",
            "In a network of {N} agents, how do you achieve {goal} with O({complexity}) complexity?",
            "Explain why {concept} is/isn't possible in a {constraint} system.",
            "How would you {task} if {limitation}? Provide {number} approaches.",
        ];
        
        let mut rng = rand::thread_rng();
        let template = templates.choose(&mut rng).unwrap();
        
        // Simple template filling (in production, this would be more sophisticated)
        let content = template
            .replace("{A}", "property A")
            .replace("{B}", "property B")
            .replace("{C}", "property C")
            .replace("{action}", "achieve consensus")
            .replace("{requirement}", "central coordination")
            .replace("{N}", "1000")
            .replace("{goal}", "global optimization")
            .replace("{complexity}", "log n")
            .replace("{concept}", "perfect prediction")
            .replace("{constraint}", "chaotic")
            .replace("{task}", "solve the halting problem")
            .replace("{limitation}", "you could only use quantum operations")
            .replace("{number}", "three");
        
        AssessmentQuestion {
            id: Uuid::new_v4(),
            category: QuestionCategory::CreativeProblemSolving,
            difficulty: level,
            content,
            time_limit: Some(std::time::Duration::from_secs(180)),
        }
    }
}

/// Question validator to ensure quality
pub struct QuestionValidator {
    min_length: usize,
    max_length: usize,
    required_keywords: HashMap<QuestionCategory, Vec<String>>,
}

impl QuestionValidator {
    pub fn new() -> Self {
        let mut required_keywords = HashMap::new();
        
        required_keywords.insert(
            QuestionCategory::LogicalReasoning,
            vec!["if", "then", "therefore", "implies", "conclude"].iter().map(|s| s.to_string()).collect()
        );
        
        required_keywords.insert(
            QuestionCategory::SystemsThinking,
            vec!["system", "design", "architecture", "component", "interaction"].iter().map(|s| s.to_string()).collect()
        );
        
        Self {
            min_length: 20,
            max_length: 500,
            required_keywords,
        }
    }
    
    pub fn validate(&self, question: &AssessmentQuestion) -> ValidationResult {
        let mut issues = Vec::new();
        
        // Check length
        let length = question.content.len();
        if length < self.min_length {
            issues.push(ValidationIssue::TooShort(length));
        }
        if length > self.max_length {
            issues.push(ValidationIssue::TooLong(length));
        }
        
        // Check for category-specific keywords
        if let Some(keywords) = self.required_keywords.get(&question.category) {
            let content_lower = question.content.to_lowercase();
            let has_keyword = keywords.iter().any(|kw| content_lower.contains(kw));
            
            if !has_keyword {
                issues.push(ValidationIssue::MissingKeywords(question.category));
            }
        }
        
        // Check difficulty appropriateness
        let word_count = question.content.split_whitespace().count();
        let expected_complexity = question.difficulty.value() as usize * 5;
        
        if word_count < expected_complexity / 2 {
            issues.push(ValidationIssue::TooSimpleForLevel);
        }
        
        ValidationResult {
            is_valid: issues.is_empty(),
            issues,
        }
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub enum ValidationIssue {
    TooShort(usize),
    TooLong(usize),
    MissingKeywords(QuestionCategory),
    TooSimpleForLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assessment_pool() {
        let pool = AssessmentPool::new();
        
        // Test getting questions for different levels
        for level in 1..=20 {
            let agent_level = AgentLevel::new(level).unwrap();
            let question = pool.get_question_for_level(agent_level);
            assert!(question.is_some());
        }
        
        // Test getting evaluation set
        let eval_set = pool.get_evaluation_set(AgentLevel::new(10).unwrap(), 6);
        assert!(!eval_set.is_empty());
        assert!(eval_set.len() <= 6);
        
        // Test category retrieval
        let logical_questions = pool.get_questions_by_category(QuestionCategory::LogicalReasoning);
        assert!(!logical_questions.is_empty());
    }

    #[test]
    fn test_question_validator() {
        let validator = QuestionValidator::new();
        
        let good_question = AssessmentQuestion {
            id: Uuid::new_v4(),
            category: QuestionCategory::LogicalReasoning,
            difficulty: AgentLevel::new(10).unwrap(),
            content: "If all philosophers are thinkers, and some thinkers are mathematicians, what can we conclude about the relationship between philosophers and mathematicians?".to_string(),
            time_limit: None,
        };
        
        let result = validator.validate(&good_question);
        assert!(result.is_valid);
        
        let bad_question = AssessmentQuestion {
            id: Uuid::new_v4(),
            category: QuestionCategory::SystemsThinking,
            difficulty: AgentLevel::new(15).unwrap(),
            content: "Too short".to_string(),
            time_limit: None,
        };
        
        let result = validator.validate(&bad_question);
        assert!(!result.is_valid);
    }
}