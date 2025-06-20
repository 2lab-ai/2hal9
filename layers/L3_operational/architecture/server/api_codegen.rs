//! Code Generation API endpoints

use axum::{
    extract::{State, Json, Path},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    server::HAL9Server,
    error::ServerError,
};
use hal9_core::NeuronSignal;

/// Code generation API state
#[derive(Clone)]
pub struct CodegenApiState {
    pub server: Arc<HAL9Server>,
}

/// Project generation request
#[derive(Debug, Deserialize)]
pub struct GenerateProjectRequest {
    pub description: String,
    pub project_type: ProjectType,
    pub preferences: ProjectPreferences,
}

/// Project type enum
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectType {
    WebApp,
    Api,
    Cli,
    Library,
    Microservice,
}

/// Project preferences
#[derive(Debug, Deserialize)]
pub struct ProjectPreferences {
    pub backend: Option<String>,
    pub frontend: Option<String>,
    pub database: Option<String>,
    pub testing: bool,
    pub docker: bool,
    pub ci_cd: bool,
}

/// Project generation response
#[derive(Debug, Serialize)]
pub struct GenerateProjectResponse {
    pub project_id: String,
    pub status: String,
    pub message: String,
    pub location: Option<String>,
}

/// Code completion request
#[derive(Debug, Deserialize)]
pub struct CodeCompletionRequest {
    pub file_path: String,
    pub cursor_position: usize,
    pub context: String,
    pub language: Option<String>,
}

/// Code completion response
#[derive(Debug, Serialize)]
pub struct CodeCompletionResponse {
    pub suggestions: Vec<CodeSuggestion>,
}

/// Individual code suggestion
#[derive(Debug, Serialize)]
pub struct CodeSuggestion {
    pub text: String,
    pub description: String,
    pub confidence: f32,
}

/// Code review request
#[derive(Debug, Deserialize)]
pub struct CodeReviewRequest {
    pub file_path: String,
    pub content: String,
    pub focus: Vec<ReviewFocus>,
}

/// Review focus areas
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewFocus {
    Security,
    Performance,
    BestPractices,
    Bugs,
    Style,
}

/// Code review response
#[derive(Debug, Serialize)]
pub struct CodeReviewResponse {
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<CodeSuggestion>,
    pub overall_score: f32,
}

/// Code issue found during review
#[derive(Debug, Serialize)]
pub struct CodeIssue {
    pub severity: String,
    pub line: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Refactoring request
#[derive(Debug, Deserialize)]
pub struct RefactorRequest {
    pub file_path: String,
    pub refactor_type: RefactorType,
    pub selection: Option<CodeSelection>,
}

/// Refactoring types
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefactorType {
    ExtractMethod,
    ExtractVariable,
    InlineVariable,
    RenameSymbol,
    OptimizeImports,
    FormatCode,
}

/// Code selection
#[derive(Debug, Deserialize)]
pub struct CodeSelection {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: Option<usize>,
    pub end_column: Option<usize>,
}

/// Refactoring response
#[derive(Debug, Serialize)]
pub struct RefactorResponse {
    pub success: bool,
    pub original_code: String,
    pub refactored_code: String,
    pub changes: Vec<CodeChange>,
}

/// Code change description
#[derive(Debug, Serialize)]
pub struct CodeChange {
    pub line: usize,
    pub description: String,
}

/// Generate a new project
pub async fn generate_project(
    State(state): State<Arc<CodegenApiState>>,
    Json(request): Json<GenerateProjectRequest>,
) -> Result<impl IntoResponse, ServerError> {
    // Create project generation signal
    let project_id = Uuid::new_v4().to_string();
    let signal_content = format!(
        "Generate {} project: {}",
        match request.project_type {
            ProjectType::WebApp => "web application",
            ProjectType::Api => "API service",
            ProjectType::Cli => "CLI tool",
            ProjectType::Library => "library",
            ProjectType::Microservice => "microservice",
        },
        request.description
    );
    
    // Add preferences to metadata
    let mut metadata = HashMap::new();
    if let Some(backend) = request.preferences.backend {
        metadata.insert("backend".to_string(), backend);
    }
    if let Some(frontend) = request.preferences.frontend {
        metadata.insert("frontend".to_string(), frontend);
    }
    if let Some(database) = request.preferences.database {
        metadata.insert("database".to_string(), database);
    }
    metadata.insert("testing".to_string(), request.preferences.testing.to_string());
    metadata.insert("docker".to_string(), request.preferences.docker.to_string());
    metadata.insert("ci_cd".to_string(), request.preferences.ci_cd.to_string());
    
    // Send to architect neuron
    let mut signal = NeuronSignal::forward(
        "codegen-api",
        "codegen-architect",
        "API",
        "L4",
        signal_content,
    );
    signal.metadata = metadata;
    
    match state.server.send_signal(signal).await {
        Ok(_) => {
            Ok(Json(GenerateProjectResponse {
                project_id: project_id.clone(),
                status: "processing".to_string(),
                message: "Project generation started".to_string(),
                location: Some(format!("/projects/{}", project_id)),
            }))
        }
        Err(e) => {
            Err(ServerError::Internal(format!("Failed to start generation: {}", e)))
        }
    }
}

/// Get code completion suggestions
pub async fn code_completion(
    State(_state): State<Arc<CodegenApiState>>,
    Json(request): Json<CodeCompletionRequest>,
) -> Result<impl IntoResponse, ServerError> {
    // Determine the appropriate implementation neuron based on file extension
    let language = request.language.or_else(|| {
        match request.file_path.split('.').next_back() {
            Some("rs") => Some("rust".to_string()),
            Some("py") => Some("python".to_string()),
            Some("ts") | Some("tsx") => Some("typescript".to_string()),
            Some("js") | Some("jsx") => Some("javascript".to_string()),
            Some("go") => Some("go".to_string()),
            _ => None,
        }
    });
    
    let target_neuron = match language.as_deref() {
        Some("rust") => "codegen-rust-impl",
        Some("python") => "codegen-python-impl",
        Some("typescript") | Some("javascript") => "codegen-typescript-impl",
        Some("go") => "codegen-go-impl",
        _ => return Err(ServerError::InvalidInput("Unsupported language".to_string())),
    };
    
    // Create completion signal
    let signal_content = format!(
        "Complete code at position {} in file {}:\n{}",
        request.cursor_position,
        request.file_path,
        request.context
    );
    
    let _signal = NeuronSignal::forward(
        "codegen-api",
        target_neuron,
        "API",
        "L2",
        signal_content,
    );
    
    // For now, return mock suggestions
    // In production, this would wait for neuron response
    Ok(Json(CodeCompletionResponse {
        suggestions: vec![
            CodeSuggestion {
                text: "// TODO: Implement completion".to_string(),
                description: "Add implementation".to_string(),
                confidence: 0.9,
            },
        ],
    }))
}

/// Review code for issues and improvements
pub async fn review_code(
    State(_state): State<Arc<CodegenApiState>>,
    Json(request): Json<CodeReviewRequest>,
) -> Result<impl IntoResponse, ServerError> {
    // Send to test designer for comprehensive review
    let mut review_aspects = vec![];
    for focus in &request.focus {
        review_aspects.push(match focus {
            ReviewFocus::Security => "security vulnerabilities",
            ReviewFocus::Performance => "performance issues",
            ReviewFocus::BestPractices => "best practice violations",
            ReviewFocus::Bugs => "potential bugs",
            ReviewFocus::Style => "style inconsistencies",
        });
    }
    
    let signal_content = format!(
        "Review code for {}: {}\n\nCode:\n{}",
        review_aspects.join(", "),
        request.file_path,
        request.content
    );
    
    let _signal = NeuronSignal::forward(
        "codegen-api",
        "codegen-test-designer",
        "API",
        "L3",
        signal_content,
    );
    
    // Mock response for now
    Ok(Json(CodeReviewResponse {
        issues: vec![
            CodeIssue {
                severity: "warning".to_string(),
                line: Some(10),
                message: "Consider adding error handling".to_string(),
                suggestion: Some("Use Result<T, E> for error handling".to_string()),
            },
        ],
        suggestions: vec![
            CodeSuggestion {
                text: "Add unit tests for this function".to_string(),
                description: "Improve test coverage".to_string(),
                confidence: 0.85,
            },
        ],
        overall_score: 7.5,
    }))
}

/// Refactor code
pub async fn refactor_code(
    State(_state): State<Arc<CodegenApiState>>,
    Json(request): Json<RefactorRequest>,
) -> Result<impl IntoResponse, ServerError> {
    // Determine appropriate neuron based on file type
    let target_neuron = match request.file_path.split('.').next_back() {
        Some("rs") => "codegen-rust-impl",
        Some("py") => "codegen-python-impl",
        Some("ts") | Some("tsx") => "codegen-typescript-impl",
        Some("go") => "codegen-go-impl",
        _ => return Err(ServerError::InvalidInput("Unsupported file type".to_string())),
    };
    
    let refactor_instruction = match request.refactor_type {
        RefactorType::ExtractMethod => "Extract selected code into a method",
        RefactorType::ExtractVariable => "Extract expression into a variable",
        RefactorType::InlineVariable => "Inline the selected variable",
        RefactorType::RenameSymbol => "Rename the selected symbol",
        RefactorType::OptimizeImports => "Optimize and organize imports",
        RefactorType::FormatCode => "Format code according to style guide",
    };
    
    let signal_content = if let Some(selection) = request.selection {
        format!(
            "{} in {} (lines {}-{})",
            refactor_instruction,
            request.file_path,
            selection.start_line,
            selection.end_line
        )
    } else {
        format!("{} in {}", refactor_instruction, request.file_path)
    };
    
    let _signal = NeuronSignal::forward(
        "codegen-api",
        target_neuron,
        "API",
        "L2",
        signal_content,
    );
    
    // Mock response
    Ok(Json(RefactorResponse {
        success: true,
        original_code: "// Original code".to_string(),
        refactored_code: "// Refactored code".to_string(),
        changes: vec![
            CodeChange {
                line: 10,
                description: "Extracted method 'processData'".to_string(),
            },
        ],
    }))
}

/// Get project generation status
pub async fn get_project_status(
    State(_state): State<Arc<CodegenApiState>>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    // In production, this would check actual generation status
    Ok(Json(serde_json::json!({
        "project_id": project_id,
        "status": "completed",
        "progress": 100,
        "files_generated": 25,
        "location": format!("./generated/{}", project_id),
    })))
}

/// List available project templates
pub async fn list_templates(
    State(_state): State<Arc<CodegenApiState>>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(Json(serde_json::json!({
        "templates": {
            "web-app": {
                "name": "Web Application",
                "description": "Full-stack web application",
                "backends": ["express", "fastapi", "django", "axum"],
                "frontends": ["react", "vue", "angular", "svelte"],
                "databases": ["postgresql", "mysql", "mongodb", "sqlite"],
            },
            "api": {
                "name": "API Service",
                "description": "RESTful or GraphQL API",
                "frameworks": ["fastapi", "express", "gin", "axum"],
                "databases": ["postgresql", "mysql", "mongodb", "redis"],
            },
            "cli": {
                "name": "CLI Tool",
                "description": "Command-line application",
                "languages": ["rust", "go", "python", "node"],
            },
            "microservice": {
                "name": "Microservice",
                "description": "Cloud-native microservice",
                "languages": ["go", "rust", "python", "java"],
                "patterns": ["rest", "grpc", "graphql", "event-driven"],
            },
        }
    })))
}

/// Health check for code generation service
pub async fn codegen_health(
    State(_state): State<Arc<CodegenApiState>>,
) -> impl IntoResponse {
    // Check if code generation neurons are healthy
    let neurons = ["codegen-architect",
        "codegen-api-designer",
        "codegen-db-designer",
        "codegen-frontend-designer",
        "codegen-test-designer"];
    
    let total_count = neurons.len();
    
    // In production, check actual neuron health
    // For now, assume all healthy
    let healthy_count = total_count;
    
    Json(serde_json::json!({
        "status": if healthy_count == total_count { "healthy" } else { "degraded" },
        "neurons": {
            "healthy": healthy_count,
            "total": total_count,
        },
        "message": format!("{}/{} code generation neurons operational", healthy_count, total_count),
    }))
}