//! Single Sign-On (SSO) integration for enterprise deployments

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use async_trait::async_trait;
use sha2::{Sha256, Digest};

/// SSO provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SsoProvider {
    Saml(SamlConfig),
    Oidc(OidcConfig),
    Custom(CustomSsoConfig),
}

/// SAML 2.0 configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamlConfig {
    /// Identity Provider metadata URL
    pub idp_metadata_url: String,
    
    /// Service Provider entity ID
    pub sp_entity_id: String,
    
    /// Assertion Consumer Service URL
    pub acs_url: String,
    
    /// Single Logout Service URL
    pub sls_url: Option<String>,
    
    /// X.509 certificate for signing
    pub sp_certificate: String,
    
    /// Private key for decryption
    pub sp_private_key: String,
    
    /// Attribute mappings
    pub attribute_mapping: AttributeMapping,
    
    /// Additional settings
    pub settings: SamlSettings,
}

/// SAML settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamlSettings {
    pub force_authn: bool,
    pub sign_requests: bool,
    pub want_assertions_signed: bool,
    pub want_assertions_encrypted: bool,
    pub name_id_format: String,
    pub authn_context_class_ref: Option<String>,
}

impl Default for SamlSettings {
    fn default() -> Self {
        Self {
            force_authn: false,
            sign_requests: true,
            want_assertions_signed: true,
            want_assertions_encrypted: false,
            name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".to_string(),
            authn_context_class_ref: None,
        }
    }
}

/// OAuth2/OIDC configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OidcConfig {
    /// Provider configuration URL
    pub discovery_url: String,
    
    /// Client ID
    pub client_id: String,
    
    /// Client secret
    pub client_secret: String,
    
    /// Redirect URI
    pub redirect_uri: String,
    
    /// Requested scopes
    pub scopes: Vec<String>,
    
    /// Attribute mappings
    pub attribute_mapping: AttributeMapping,
    
    /// Additional settings
    pub settings: OidcSettings,
}

/// OIDC settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OidcSettings {
    pub use_pkce: bool,
    pub prompt: Option<String>,
    pub max_age: Option<u32>,
    pub ui_locales: Option<Vec<String>>,
    pub acr_values: Option<Vec<String>>,
}

impl Default for OidcSettings {
    fn default() -> Self {
        Self {
            use_pkce: true,
            prompt: None,
            max_age: None,
            ui_locales: None,
            acr_values: None,
        }
    }
}

/// Custom SSO provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomSsoConfig {
    pub provider_name: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub attribute_mapping: AttributeMapping,
}

/// Attribute mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttributeMapping {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub groups: Option<String>,
    pub custom_attributes: HashMap<String, String>,
}

impl Default for AttributeMapping {
    fn default() -> Self {
        Self {
            user_id: "sub".to_string(),
            email: "email".to_string(),
            name: "name".to_string(),
            groups: None,
            custom_attributes: HashMap::new(),
        }
    }
}

/// SAML authentication request
#[derive(Debug, Clone)]
pub struct SamlAuthRequest {
    pub id: String,
    pub issue_instant: DateTime<Utc>,
    pub destination: String,
    pub assertion_consumer_service_url: String,
    pub xml: String,
}

/// SAML assertion
#[derive(Debug, Clone)]
pub struct SamlAssertion {
    pub subject: String,
    pub attributes: HashMap<String, Vec<String>>,
    pub session_index: Option<String>,
    pub authn_instant: DateTime<Utc>,
    pub not_before: DateTime<Utc>,
    pub not_on_or_after: DateTime<Utc>,
}

/// OAuth token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

/// User info from OAuth provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
    pub groups: Option<Vec<String>>,
    pub custom_claims: HashMap<String, serde_json::Value>,
}

/// SSO user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub groups: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub provider: String,
    pub session_index: Option<String>,
}

/// SSO handler trait
#[async_trait]
pub trait SsoHandler: Send + Sync {
    /// Get authentication URL
    async fn get_auth_url(&self, state: &str) -> Result<String>;
    
    /// Process authentication response
    async fn process_response(&self, params: HashMap<String, String>) -> Result<SsoUser>;
    
    /// Logout user
    async fn logout(&self, user: &SsoUser) -> Result<Option<String>>;
    
    /// Refresh user information
    async fn refresh_user(&self, user: &SsoUser) -> Result<SsoUser>;
}

/// SAML handler implementation
pub struct SamlHandler {
    config: SamlConfig,
}

impl SamlHandler {
    pub fn new(config: SamlConfig) -> Self {
        Self { config }
    }
    
    /// Create SAML authentication request
    fn create_auth_request_xml(&self, request_id: &str) -> String {
        let issue_instant = Utc::now().to_rfc3339();
        
        format!(
            r#"<samlp:AuthnRequest 
                xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                ID="{}"
                Version="2.0"
                IssueInstant="{}"
                Destination="{}"
                AssertionConsumerServiceURL="{}"
                ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
                <saml:Issuer>{}</saml:Issuer>
                <samlp:NameIDPolicy Format="{}" AllowCreate="true"/>
            </samlp:AuthnRequest>"#,
            request_id,
            issue_instant,
            self.config.idp_metadata_url,
            self.config.acs_url,
            self.config.sp_entity_id,
            self.config.settings.name_id_format
        )
    }
    
    /// Parse SAML response
    fn parse_saml_response(&self, _response: &str) -> Result<SamlAssertion> {
        // TODO: Implement actual SAML response parsing
        // This would use a proper SAML library
        Ok(SamlAssertion {
            subject: "user@example.com".to_string(),
            attributes: HashMap::new(),
            session_index: Some("session123".to_string()),
            authn_instant: Utc::now(),
            not_before: Utc::now(),
            not_on_or_after: Utc::now() + chrono::Duration::hours(8),
        })
    }
}

#[async_trait]
impl SsoHandler for SamlHandler {
    async fn get_auth_url(&self, state: &str) -> Result<String> {
        let request_id = format!("_{}", Uuid::new_v4());
        let auth_request = self.create_auth_request_xml(&request_id);
        
        // Base64 encode and URL encode the request
        let encoded = base64::encode(&auth_request);
        let url_encoded = urlencoding::encode(&encoded);
        
        Ok(format!(
            "{}?SAMLRequest={}&RelayState={}",
            self.config.idp_metadata_url,
            url_encoded,
            state
        ))
    }
    
    async fn process_response(&self, params: HashMap<String, String>) -> Result<SsoUser> {
        let saml_response = params.get("SAMLResponse")
            .ok_or_else(|| anyhow::anyhow!("Missing SAMLResponse"))?;
        
        let decoded = base64::decode(saml_response)?;
        let response_xml = String::from_utf8(decoded)?;
        
        let assertion = self.parse_saml_response(&response_xml)?;
        
        Ok(SsoUser {
            id: assertion.subject.clone(),
            email: assertion.subject.clone(),
            name: assertion.attributes.get("name")
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_else(|| assertion.subject.clone()),
            groups: assertion.attributes.get("groups")
                .cloned()
                .unwrap_or_default(),
            attributes: assertion.attributes.into_iter()
                .filter_map(|(k, v)| v.first().map(|val| (k, val.clone())))
                .collect(),
            provider: "saml".to_string(),
            session_index: assertion.session_index,
        })
    }
    
    async fn logout(&self, user: &SsoUser) -> Result<Option<String>> {
        if let Some(sls_url) = &self.config.sls_url {
            let logout_request = format!(
                r#"<samlp:LogoutRequest
                    xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                    ID="{}"
                    Version="2.0"
                    IssueInstant="{}">
                    <saml:Issuer>{}</saml:Issuer>
                    <saml:NameID>{}</saml:NameID>
                    <samlp:SessionIndex>{}</samlp:SessionIndex>
                </samlp:LogoutRequest>"#,
                Uuid::new_v4(),
                Utc::now().to_rfc3339(),
                self.config.sp_entity_id,
                user.id,
                user.session_index.as_deref().unwrap_or("")
            );
            
            let encoded = base64::encode(&logout_request);
            let url_encoded = urlencoding::encode(&encoded);
            
            Ok(Some(format!("{}?SAMLRequest={}", sls_url, url_encoded)))
        } else {
            Ok(None)
        }
    }
    
    async fn refresh_user(&self, user: &SsoUser) -> Result<SsoUser> {
        // SAML doesn't support refresh, return existing user
        Ok(user.clone())
    }
}

/// OAuth/OIDC handler implementation
pub struct OidcHandler {
    config: OidcConfig,
    discovery: Option<OidcDiscovery>,
}

#[derive(Debug, Clone, Deserialize)]
struct OidcDiscovery {
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    jwks_uri: String,
    issuer: String,
}

impl OidcHandler {
    pub async fn new(config: OidcConfig) -> Result<Self> {
        let discovery = Self::discover(&config.discovery_url).await?;
        Ok(Self {
            config,
            discovery: Some(discovery),
        })
    }
    
    async fn discover(url: &str) -> Result<OidcDiscovery> {
        // TODO: Implement actual discovery
        Ok(OidcDiscovery {
            authorization_endpoint: format!("{}/authorize", url),
            token_endpoint: format!("{}/token", url),
            userinfo_endpoint: format!("{}/userinfo", url),
            jwks_uri: format!("{}/jwks", url),
            issuer: url.to_string(),
        })
    }
    
    fn generate_pkce_challenge() -> (String, String) {
        let verifier = base64::encode(&Uuid::new_v4().as_bytes());
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge = base64::encode(&hasher.finalize());
        (verifier, challenge)
    }
}

#[async_trait]
impl SsoHandler for OidcHandler {
    async fn get_auth_url(&self, state: &str) -> Result<String> {
        let discovery = self.discovery.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OIDC discovery not initialized"))?;
        
        let mut params = vec![
            ("client_id", self.config.client_id.clone()),
            ("redirect_uri", self.config.redirect_uri.clone()),
            ("response_type", "code".to_string()),
            ("scope", self.config.scopes.join(" ")),
            ("state", state.to_string()),
        ];
        
        if self.config.settings.use_pkce {
            let (_verifier, challenge) = Self::generate_pkce_challenge();
            params.push(("code_challenge", challenge));
            params.push(("code_challenge_method", "S256".to_string()));
        }
        
        let query = params.into_iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(&v)))
            .collect::<Vec<_>>()
            .join("&");
        
        Ok(format!("{}?{}", discovery.authorization_endpoint, query))
    }
    
    async fn process_response(&self, params: HashMap<String, String>) -> Result<SsoUser> {
        let code = params.get("code")
            .ok_or_else(|| anyhow::anyhow!("Missing authorization code"))?;
        
        // Exchange code for tokens
        let token_response = self.exchange_code(code).await?;
        
        // Get user info
        let user_info = self.get_user_info(&token_response.access_token).await?;
        
        Ok(SsoUser {
            id: user_info.sub.clone(),
            email: user_info.email.unwrap_or_else(|| user_info.sub.clone()),
            name: user_info.name.unwrap_or_else(|| user_info.sub.clone()),
            groups: user_info.groups.unwrap_or_default(),
            attributes: user_info.custom_claims.into_iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect(),
            provider: "oidc".to_string(),
            session_index: None,
        })
    }
    
    async fn logout(&self, _user: &SsoUser) -> Result<Option<String>> {
        // TODO: Implement OIDC logout
        Ok(None)
    }
    
    async fn refresh_user(&self, user: &SsoUser) -> Result<SsoUser> {
        // TODO: Implement token refresh
        Ok(user.clone())
    }
}

impl OidcHandler {
    async fn exchange_code(&self, _code: &str) -> Result<TokenResponse> {
        // TODO: Implement actual token exchange
        Ok(TokenResponse {
            access_token: "mock_access_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("mock_refresh_token".to_string()),
            id_token: Some("mock_id_token".to_string()),
            scope: Some(self.config.scopes.join(" ")),
        })
    }
    
    async fn get_user_info(&self, _token: &str) -> Result<OAuthUserInfo> {
        // TODO: Implement actual user info retrieval
        Ok(OAuthUserInfo {
            sub: "user123".to_string(),
            email: Some("user@example.com".to_string()),
            email_verified: Some(true),
            name: Some("Test User".to_string()),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            picture: None,
            locale: Some("en-US".to_string()),
            groups: Some(vec!["users".to_string(), "developers".to_string()]),
            custom_claims: HashMap::new(),
        })
    }
}

/// SSO manager
pub struct SsoManager {
    handlers: HashMap<String, Box<dyn SsoHandler>>,
}

impl SsoManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register SSO provider
    pub fn register_provider(&mut self, name: String, handler: Box<dyn SsoHandler>) {
        self.handlers.insert(name, handler);
    }
    
    /// Get authentication URL for provider
    pub async fn get_auth_url(&self, provider: &str, state: &str) -> Result<String> {
        let handler = self.handlers.get(provider)
            .ok_or_else(|| anyhow::anyhow!("Unknown SSO provider: {}", provider))?;
        
        handler.get_auth_url(state).await
    }
    
    /// Process authentication response
    pub async fn process_response(
        &self,
        provider: &str,
        params: HashMap<String, String>
    ) -> Result<SsoUser> {
        let handler = self.handlers.get(provider)
            .ok_or_else(|| anyhow::anyhow!("Unknown SSO provider: {}", provider))?;
        
        handler.process_response(params).await
    }
    
    /// Logout user
    pub async fn logout(&self, provider: &str, user: &SsoUser) -> Result<Option<String>> {
        let handler = self.handlers.get(provider)
            .ok_or_else(|| anyhow::anyhow!("Unknown SSO provider: {}", provider))?;
        
        handler.logout(user).await
    }
}

/// SSO configuration manager
pub struct SsoConfigManager {
    configs: HashMap<Uuid, SsoConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub provider: SsoProvider,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SsoConfigManager {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }
    
    /// Add SSO configuration
    pub fn add_config(&mut self, config: SsoConfig) {
        self.configs.insert(config.organization_id, config);
    }
    
    /// Get SSO configuration for organization
    pub fn get_config(&self, org_id: &Uuid) -> Option<&SsoConfig> {
        self.configs.get(org_id)
    }
    
    /// Update SSO configuration
    pub fn update_config(&mut self, config: SsoConfig) {
        self.configs.insert(config.organization_id, config);
    }
    
    /// Remove SSO configuration
    pub fn remove_config(&mut self, org_id: &Uuid) -> Option<SsoConfig> {
        self.configs.remove(org_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saml_auth_request() {
        let config = SamlConfig {
            idp_metadata_url: "https://idp.example.com/metadata".to_string(),
            sp_entity_id: "https://sp.example.com".to_string(),
            acs_url: "https://sp.example.com/acs".to_string(),
            sls_url: None,
            sp_certificate: "cert".to_string(),
            sp_private_key: "key".to_string(),
            attribute_mapping: AttributeMapping::default(),
            settings: SamlSettings::default(),
        };
        
        let handler = SamlHandler::new(config);
        let xml = handler.create_auth_request_xml("test123");
        
        assert!(xml.contains("ID=\"test123\""));
        assert!(xml.contains("https://idp.example.com/metadata"));
        assert!(xml.contains("https://sp.example.com"));
    }
    
    #[test]
    fn test_pkce_challenge() {
        let (verifier, challenge) = OidcHandler::generate_pkce_challenge();
        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());
        assert_ne!(verifier, challenge);
    }
    
    #[test]
    fn test_sso_manager() {
        let mut manager = SsoManager::new();
        let handler = Box::new(SamlHandler::new(SamlConfig {
            idp_metadata_url: "https://idp.example.com".to_string(),
            sp_entity_id: "sp".to_string(),
            acs_url: "acs".to_string(),
            sls_url: None,
            sp_certificate: "cert".to_string(),
            sp_private_key: "key".to_string(),
            attribute_mapping: AttributeMapping::default(),
            settings: SamlSettings::default(),
        }));
        
        manager.register_provider("saml".to_string(), handler);
        assert!(manager.handlers.contains_key("saml"));
    }
}