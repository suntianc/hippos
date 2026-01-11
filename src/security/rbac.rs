//! Role-Based Access Control (RBAC) Module
//!
//! Provides authorization through role-based permissions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Role enumeration for access control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    /// Full system administrator
    Admin,
    /// Tenant-level administrator
    TenantAdmin,
    /// Regular user
    User,
    /// Read-only access
    ReadOnly,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::TenantAdmin => write!(f, "tenant_admin"),
            Role::User => write!(f, "user"),
            Role::ReadOnly => write!(f, "read_only"),
        }
    }
}

impl Role {
    /// Convert from string to Role
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "tenant_admin" | "tenantadmin" => Role::TenantAdmin,
            "user" => Role::User,
            "read_only" | "readonly" | "read" => Role::ReadOnly,
            _ => Role::User,
        }
    }

    /// Check if role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if role has elevated privileges
    pub fn is_elevated(&self) -> bool {
        matches!(self, Role::Admin | Role::TenantAdmin)
    }
}

/// Resource types that can be protected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Session resources
    Session,
    /// Turn/conversation resources
    Turn,
    /// Index/search resources
    Index,
    /// System/configuration resources
    System,
    /// User management resources
    User,
    /// All resources wildcard
    All,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Session => write!(f, "session"),
            ResourceType::Turn => write!(f, "turn"),
            ResourceType::Index => write!(f, "index"),
            ResourceType::System => write!(f, "system"),
            ResourceType::User => write!(f, "user"),
            ResourceType::All => write!(f, "all"),
        }
    }
}

/// Action types that can be performed on resources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    /// Create new resources
    Create,
    /// Read/view resources
    Read,
    /// Update/modify resources
    Update,
    /// Delete resources
    Delete,
    /// Search/query resources
    Search,
    /// Manage resources (admin action)
    Manage,
    /// All actions wildcard
    All,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::Create => write!(f, "create"),
            ActionType::Read => write!(f, "read"),
            ActionType::Update => write!(f, "update"),
            ActionType::Delete => write!(f, "delete"),
            ActionType::Search => write!(f, "search"),
            ActionType::Manage => write!(f, "manage"),
            ActionType::All => write!(f, "all"),
        }
    }
}

/// Permission definition combining resource and action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type
    pub resource: ResourceType,
    /// Action type
    pub action: ActionType,
    /// Optional: specific resource ID constraint
    pub resource_id: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: ResourceType, action: ActionType) -> Self {
        Self {
            resource,
            action,
            resource_id: None,
        }
    }

    /// Create a permission with specific resource ID
    pub fn new_with_id(resource: ResourceType, action: ActionType, resource_id: String) -> Self {
        Self {
            resource,
            action,
            resource_id: Some(resource_id),
        }
    }

    /// Check if this permission matches another (allowing wildcards)
    pub fn matches(&self, other: &Permission) -> bool {
        let resource_match = self.resource == ResourceType::All
            || other.resource == ResourceType::All
            || self.resource == other.resource;

        let action_match = self.action == ActionType::All
            || other.action == ActionType::All
            || self.action == other.action;

        let id_match = match (&self.resource_id, &other.resource_id) {
            (Some(_), Some(_)) => self.resource_id == other.resource_id,
            (None, _) => true,
            (_, None) => true,
        };

        resource_match && action_match && id_match
    }
}

/// Default permissions for each role
pub fn get_default_permissions(role: &Role) -> Vec<Permission> {
    match role {
        Role::Admin => vec![
            // All permissions on all resources
            Permission::new(ResourceType::All, ActionType::All),
        ],
        Role::TenantAdmin => vec![
            // Full access to tenant resources
            Permission::new(ResourceType::Session, ActionType::All),
            Permission::new(ResourceType::Turn, ActionType::All),
            Permission::new(ResourceType::Index, ActionType::All),
            Permission::new(ResourceType::User, ActionType::All),
            // Read-only on system
            Permission::new(ResourceType::System, ActionType::Read),
        ],
        Role::User => vec![
            // CRUD on own sessions and turns
            Permission::new(ResourceType::Session, ActionType::Create),
            Permission::new(ResourceType::Session, ActionType::Read),
            Permission::new(ResourceType::Session, ActionType::Update),
            Permission::new(ResourceType::Session, ActionType::Delete),
            Permission::new(ResourceType::Turn, ActionType::Create),
            Permission::new(ResourceType::Turn, ActionType::Read),
            Permission::new(ResourceType::Turn, ActionType::Update),
            Permission::new(ResourceType::Turn, ActionType::Delete),
            // Search permissions
            Permission::new(ResourceType::Index, ActionType::Search),
            Permission::new(ResourceType::Index, ActionType::Read),
            // No system access
        ],
        Role::ReadOnly => vec![
            // Read-only access
            Permission::new(ResourceType::Session, ActionType::Read),
            Permission::new(ResourceType::Turn, ActionType::Read),
            Permission::new(ResourceType::Index, ActionType::Read),
            Permission::new(ResourceType::Index, ActionType::Search),
        ],
    }
}

/// Authorizer trait for checking permissions
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if a subject has permission to perform an action
    async fn check_permission(&self, claims: &Claims, permission: &Permission) -> bool;
    /// Get all permissions for a role
    async fn get_role_permissions(&self, role: &Role) -> Vec<Permission>;
    /// Check if subject can access a specific resource
    async fn can_access_resource(
        &self,
        claims: &Claims,
        resource_type: ResourceType,
        action: ActionType,
        resource_id: Option<&str>,
    ) -> bool;
}

/// Claims structure for authorization (re-exported from auth module)
use crate::security::auth::Claims;

/// Simple in-memory authorizer implementation
#[derive(Debug, Clone)]
pub struct SimpleAuthorizer {
    /// Role permissions cache
    permissions: Vec<(Role, Vec<Permission>)>,
}

impl SimpleAuthorizer {
    /// Create new authorizer with default permissions
    pub fn new() -> Self {
        let permissions = vec![
            (Role::Admin, get_default_permissions(&Role::Admin)),
            (
                Role::TenantAdmin,
                get_default_permissions(&Role::TenantAdmin),
            ),
            (Role::User, get_default_permissions(&Role::User)),
            (Role::ReadOnly, get_default_permissions(&Role::ReadOnly)),
        ];

        Self { permissions }
    }

    /// Create development authorizer
    pub fn development() -> Self {
        Self::new()
    }

    /// Add custom permissions for a role
    pub fn with_permissions(mut self, role: Role, permissions: Vec<Permission>) -> Self {
        self.permissions.retain(|(r, _)| *r != role);
        self.permissions.push((role, permissions));
        self
    }
}

impl Default for SimpleAuthorizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Authorizer for SimpleAuthorizer {
    async fn check_permission(&self, claims: &Claims, permission: &Permission) -> bool {
        let role = Role::from_string(&claims.role);

        // Admin has all permissions
        if role.is_admin() {
            return true;
        }

        // Check role permissions
        if let Some((_, perms)) = self.permissions.iter().find(|(r, _)| *r == role) {
            for perm in perms {
                if perm.matches(permission) {
                    return true;
                }
            }
        }

        false
    }

    async fn get_role_permissions(&self, role: &Role) -> Vec<Permission> {
        if let Some((_, perms)) = self.permissions.iter().find(|(r, _)| *r == *role) {
            perms.clone()
        } else {
            get_default_permissions(role)
        }
    }

    async fn can_access_resource(
        &self,
        claims: &Claims,
        resource_type: ResourceType,
        action: ActionType,
        resource_id: Option<&str>,
    ) -> bool {
        let permission =
            Permission::new_with_id(resource_type, action, resource_id.unwrap_or("").to_string());
        self.check_permission(claims, &permission).await
    }
}

/// Claims extension trait for authorization helpers
pub trait ClaimsExt {
    /// Get the tenant ID from claims
    fn tenant_id(&self) -> &str;
    /// Get the role from claims
    fn role(&self) -> Role;
    /// Check if user is admin
    fn is_admin(&self) -> bool;
    /// Check if user can access tenant
    fn can_access_tenant(&self, tenant_id: &str) -> bool;
}

impl ClaimsExt for Claims {
    fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    fn role(&self) -> Role {
        Role::from_string(&self.role)
    }

    fn is_admin(&self) -> bool {
        self.role().is_admin()
    }

    fn can_access_tenant(&self, tenant_id: &str) -> bool {
        self.is_admin() || self.tenant_id() == tenant_id
    }
}
