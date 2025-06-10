//! API client for HAL9 code generation service

use anyhow::{Context, Result};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Code generation API client
pub struct CodegenClient {
    client: Client,
    base_url: String,
    _api_key: Option<String>,
}

impl CodegenClient {
    /// Create a new API client
    pub fn new(base_url: &str, api_key: Option<String>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(ref key) = api_key {
            headers.insert(
                header::HeaderName::from_static("x-api-key"),
                header::HeaderValue::from_str(key)?,
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(300)) // 5 minutes for large generations
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            _api_key: api_key,
        })
    }

    /// Generate a new project
    #[allow(clippy::too_many_arguments)]
    pub async fn generate_project(
        &self,
        description: &str,
        project_type: &str,
        backend: Option<String>,
        frontend: Option<String>,
        database: Option<String>,
        testing: bool,
        docker: bool,
        ci_cd: bool,
    ) -> Result<GenerateProjectResponse> {
        let request = GenerateProjectRequest {
            description: description.to_string(),
            project_type: match project_type {
                "web-app" => ProjectType::WebApp,
                "api" => ProjectType::Api,
                "cli" => ProjectType::Cli,
                "library" => ProjectType::Library,
                "microservice" => ProjectType::Microservice,
                _ => return Err(anyhow::anyhow!("Invalid project type")),
            },
            preferences: ProjectPreferences {
                backend,
                frontend,
                database,
                testing,
                docker,
                ci_cd,
            },
        };

        let response = self
            .client
            .post(format!("{}/api/v1/codegen/project", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send project generation request")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("API error: {}", error_text));
        }

        response
            .json::<GenerateProjectResponse>()
            .await
            .context("Failed to parse project generation response")
    }

    /// Get project generation status
    #[allow(dead_code)]
    pub async fn get_project_status(&self, project_id: &str) -> Result<ProjectStatus> {
        let response = self
            .client
            .get(format!(
                "{}/api/v1/codegen/project/{}",
                self.base_url, project_id
            ))
            .send()
            .await
            .context("Failed to get project status")?;

        response
            .json::<ProjectStatus>()
            .await
            .context("Failed to parse project status")
    }

    /// Get code completion suggestions
    #[allow(dead_code)]
    pub async fn complete_code(
        &self,
        file_path: String,
        cursor_position: usize,
        context: String,
        language: Option<String>,
    ) -> Result<CodeCompletionResponse> {
        let request = CodeCompletionRequest {
            file_path,
            cursor_position,
            context,
            language,
        };

        let response = self
            .client
            .post(format!("{}/api/v1/codegen/complete", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to get code completion")?;

        response
            .json::<CodeCompletionResponse>()
            .await
            .context("Failed to parse completion response")
    }

    /// Review code
    pub async fn review_code(
        &self,
        file_path: String,
        content: String,
        focus: Vec<String>,
    ) -> Result<CodeReviewResponse> {
        let focus_areas = focus
            .iter()
            .map(|f| match f.as_str() {
                "security" => ReviewFocus::Security,
                "performance" => ReviewFocus::Performance,
                "best-practices" => ReviewFocus::BestPractices,
                "bugs" => ReviewFocus::Bugs,
                "style" => ReviewFocus::Style,
                _ => ReviewFocus::BestPractices,
            })
            .collect();

        let request = CodeReviewRequest {
            file_path,
            content,
            focus: focus_areas,
        };

        let response = self
            .client
            .post(format!("{}/api/v1/codegen/review", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to review code")?;

        response
            .json::<CodeReviewResponse>()
            .await
            .context("Failed to parse review response")
    }

    /// Refactor code
    pub async fn refactor_code(
        &self,
        file_path: String,
        refactor_type: String,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> Result<RefactorResponse> {
        let refactor_type = match refactor_type.as_str() {
            "extract-method" => RefactorType::ExtractMethod,
            "extract-variable" => RefactorType::ExtractVariable,
            "inline-variable" => RefactorType::InlineVariable,
            "rename-symbol" => RefactorType::RenameSymbol,
            "optimize-imports" => RefactorType::OptimizeImports,
            "format-code" => RefactorType::FormatCode,
            _ => return Err(anyhow::anyhow!("Invalid refactor type")),
        };

        let selection = if let (Some(start), Some(end)) = (start_line, end_line) {
            Some(CodeSelection {
                start_line: start,
                end_line: end,
                start_column: None,
                end_column: None,
            })
        } else {
            None
        };

        let request = RefactorRequest {
            file_path,
            refactor_type,
            selection,
        };

        let response = self
            .client
            .post(format!("{}/api/v1/codegen/refactor", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to refactor code")?;

        response
            .json::<RefactorResponse>()
            .await
            .context("Failed to parse refactor response")
    }

    /// Get available templates
    #[allow(dead_code)]
    pub async fn get_templates(&self) -> Result<TemplatesResponse> {
        let response = self
            .client
            .get(format!("{}/api/v1/codegen/templates", self.base_url))
            .send()
            .await
            .context("Failed to get templates")?;

        response
            .json::<TemplatesResponse>()
            .await
            .context("Failed to parse templates")
    }
}

// Request/Response types

#[derive(Debug, Serialize)]
struct GenerateProjectRequest {
    description: String,
    project_type: ProjectType,
    preferences: ProjectPreferences,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProjectType {
    WebApp,
    Api,
    Cli,
    Library,
    Microservice,
}

#[derive(Debug, Serialize)]
struct ProjectPreferences {
    backend: Option<String>,
    frontend: Option<String>,
    database: Option<String>,
    testing: bool,
    docker: bool,
    ci_cd: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GenerateProjectResponse {
    pub project_id: String,
    pub status: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ProjectStatus {
    pub project_id: String,
    pub status: String,
    pub progress: u32,
    pub files_generated: u32,
    pub location: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct CodeCompletionRequest {
    file_path: String,
    cursor_position: usize,
    context: String,
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CodeCompletionResponse {
    pub _suggestions: Vec<CodeSuggestion>,
}

#[derive(Debug, Deserialize)]
pub struct CodeSuggestion {
    pub text: String,
    pub description: String,
    pub _confidence: f32,
}

#[derive(Debug, Serialize)]
struct CodeReviewRequest {
    file_path: String,
    content: String,
    focus: Vec<ReviewFocus>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ReviewFocus {
    Security,
    Performance,
    BestPractices,
    Bugs,
    Style,
}

#[derive(Debug, Deserialize)]
pub struct CodeReviewResponse {
    pub issues: Vec<CodeIssue>,
    pub _suggestions: Vec<CodeSuggestion>,
    pub overall_score: f32,
}

#[derive(Debug, Deserialize)]
pub struct CodeIssue {
    pub severity: String,
    pub line: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize)]
struct RefactorRequest {
    file_path: String,
    refactor_type: RefactorType,
    selection: Option<CodeSelection>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RefactorType {
    ExtractMethod,
    ExtractVariable,
    InlineVariable,
    RenameSymbol,
    OptimizeImports,
    FormatCode,
}

#[derive(Debug, Serialize)]
struct CodeSelection {
    start_line: usize,
    end_line: usize,
    start_column: Option<usize>,
    end_column: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RefactorResponse {
    pub success: bool,
    pub original_code: String,
    pub refactored_code: String,
    pub changes: Vec<CodeChange>,
}

#[derive(Debug, Deserialize)]
pub struct CodeChange {
    pub line: usize,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct TemplatesResponse {
    pub _templates: serde_json::Value,
}
