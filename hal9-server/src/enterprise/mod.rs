//! Enterprise features for B2B deployments

pub mod organization;
pub mod team;
pub mod sso;
pub mod rbac;
pub mod audit;
pub mod compliance;
pub mod database_runtime;

pub use organization::{Organization, OrganizationManager, SubscriptionTier};
pub use team::{Team, TeamManager, TeamRole};
pub use sso::{SsoProvider, SsoManager};
pub use rbac::{Role, Permission, RbacManager};
pub use audit::{AuditEvent, AuditLogger};
pub use compliance::{ComplianceManager, CompliancePolicy};

#[cfg(test)]
mod tests;