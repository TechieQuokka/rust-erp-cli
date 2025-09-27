use crate::core::database::models::UserRole;
use crate::utils::error::{ErpError, ErpResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: &str, action: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }

    pub fn from_string(permission: &str) -> ErpResult<Self> {
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() != 2 {
            return Err(ErpError::validation(
                "permission",
                format!(
                    "Invalid permission format: {}. Expected format: resource:action",
                    permission
                ),
            ));
        }

        Ok(Self {
            resource: parts[0].to_string(),
            action: parts[1].to_string(),
        })
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

#[derive(Debug, Clone)]
pub struct RbacService {
    role_permissions: HashMap<UserRole, HashSet<Permission>>,
    user_permissions: HashMap<Uuid, HashSet<Permission>>,
    resource_hierarchy: HashMap<String, Vec<String>>,
}

impl RbacService {
    pub fn new() -> Self {
        let mut service = Self {
            role_permissions: HashMap::new(),
            user_permissions: HashMap::new(),
            resource_hierarchy: HashMap::new(),
        };

        service.initialize_default_permissions();
        service.setup_resource_hierarchy();
        service
    }

    fn initialize_default_permissions(&mut self) {
        let admin_permissions = vec![
            Permission::new("users", "create"),
            Permission::new("users", "read"),
            Permission::new("users", "update"),
            Permission::new("users", "delete"),
            Permission::new("products", "create"),
            Permission::new("products", "read"),
            Permission::new("products", "update"),
            Permission::new("products", "delete"),
            Permission::new("customers", "create"),
            Permission::new("customers", "read"),
            Permission::new("customers", "update"),
            Permission::new("customers", "delete"),
            Permission::new("orders", "create"),
            Permission::new("orders", "read"),
            Permission::new("orders", "update"),
            Permission::new("orders", "delete"),
            Permission::new("orders", "cancel"),
            Permission::new("reports", "read"),
            Permission::new("reports", "export"),
            Permission::new("config", "read"),
            Permission::new("config", "update"),
            Permission::new("system", "admin"),
        ];

        let manager_permissions = vec![
            Permission::new("users", "read"),
            Permission::new("products", "create"),
            Permission::new("products", "read"),
            Permission::new("products", "update"),
            Permission::new("products", "delete"),
            Permission::new("customers", "create"),
            Permission::new("customers", "read"),
            Permission::new("customers", "update"),
            Permission::new("customers", "delete"),
            Permission::new("orders", "create"),
            Permission::new("orders", "read"),
            Permission::new("orders", "update"),
            Permission::new("orders", "delete"),
            Permission::new("orders", "cancel"),
            Permission::new("reports", "read"),
            Permission::new("reports", "export"),
        ];

        let employee_permissions = vec![
            Permission::new("products", "read"),
            Permission::new("products", "update"),
            Permission::new("customers", "create"),
            Permission::new("customers", "read"),
            Permission::new("customers", "update"),
            Permission::new("orders", "create"),
            Permission::new("orders", "read"),
            Permission::new("orders", "update"),
            Permission::new("reports", "read"),
        ];

        let viewer_permissions = vec![
            Permission::new("products", "read"),
            Permission::new("customers", "read"),
            Permission::new("orders", "read"),
            Permission::new("reports", "read"),
        ];

        self.role_permissions
            .insert(UserRole::Admin, admin_permissions.into_iter().collect());
        self.role_permissions
            .insert(UserRole::Manager, manager_permissions.into_iter().collect());
        self.role_permissions.insert(
            UserRole::Employee,
            employee_permissions.into_iter().collect(),
        );
        self.role_permissions
            .insert(UserRole::Viewer, viewer_permissions.into_iter().collect());
    }

    fn setup_resource_hierarchy(&mut self) {
        // Define resource hierarchy where parent resources grant access to child resources
        self.resource_hierarchy.insert(
            "system".to_string(),
            vec![
                "users".to_string(),
                "products".to_string(),
                "customers".to_string(),
                "orders".to_string(),
                "reports".to_string(),
                "config".to_string(),
            ],
        );

        self.resource_hierarchy.insert(
            "orders".to_string(),
            vec!["order_items".to_string(), "invoices".to_string()],
        );

        self.resource_hierarchy.insert(
            "products".to_string(),
            vec!["inventory".to_string(), "categories".to_string()],
        );

        self.resource_hierarchy.insert(
            "reports".to_string(),
            vec![
                "sales_reports".to_string(),
                "inventory_reports".to_string(),
                "customer_reports".to_string(),
                "financial_reports".to_string(),
            ],
        );
    }

    pub fn has_permission(
        &self,
        user_id: Uuid,
        role: &UserRole,
        permission: &str,
    ) -> ErpResult<bool> {
        let required_permission = Permission::from_string(permission)?;

        // Check role-based permissions
        if let Some(role_permissions) = self.role_permissions.get(role) {
            if self.check_permission_match(role_permissions, &required_permission) {
                return Ok(true);
            }
        }

        // Check user-specific permissions
        if let Some(user_permissions) = self.user_permissions.get(&user_id) {
            if self.check_permission_match(user_permissions, &required_permission) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn has_any_permission(
        &self,
        user_id: Uuid,
        role: &UserRole,
        permissions: &[String],
    ) -> ErpResult<bool> {
        for permission in permissions {
            if self.has_permission(user_id, role, permission)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn has_all_permissions(
        &self,
        user_id: Uuid,
        role: &UserRole,
        permissions: &[String],
    ) -> ErpResult<bool> {
        for permission in permissions {
            if !self.has_permission(user_id, role, permission)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn can_access_resource(
        &self,
        user_id: Uuid,
        role: &UserRole,
        resource: &str,
        action: &str,
    ) -> ErpResult<bool> {
        let permission_str = format!("{}:{}", resource, action);
        self.has_permission(user_id, role, &permission_str)
    }

    pub fn get_user_permissions(&self, user_id: Uuid, role: &UserRole) -> HashSet<String> {
        let mut permissions = HashSet::new();

        // Add role-based permissions
        if let Some(role_permissions) = self.role_permissions.get(role) {
            for permission in role_permissions {
                permissions.insert(permission.to_string());
            }
        }

        // Add user-specific permissions
        if let Some(user_permissions) = self.user_permissions.get(&user_id) {
            for permission in user_permissions {
                permissions.insert(permission.to_string());
            }
        }

        permissions
    }

    pub fn get_role_permissions(&self, role: &UserRole) -> Vec<String> {
        if let Some(permissions) = self.role_permissions.get(role) {
            permissions.iter().map(|p| p.to_string()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn grant_user_permission(&mut self, user_id: Uuid, permission: &str) -> ErpResult<()> {
        let permission = Permission::from_string(permission)?;

        self.user_permissions
            .entry(user_id)
            .or_default()
            .insert(permission);

        Ok(())
    }

    pub fn revoke_user_permission(&mut self, user_id: Uuid, permission: &str) -> ErpResult<()> {
        let permission = Permission::from_string(permission)?;

        if let Some(user_permissions) = self.user_permissions.get_mut(&user_id) {
            user_permissions.remove(&permission);
            if user_permissions.is_empty() {
                self.user_permissions.remove(&user_id);
            }
        }

        Ok(())
    }

    pub fn clear_user_permissions(&mut self, user_id: Uuid) {
        self.user_permissions.remove(&user_id);
    }

    pub fn add_role_permission(&mut self, role: UserRole, permission: &str) -> ErpResult<()> {
        let permission = Permission::from_string(permission)?;

        self.role_permissions
            .entry(role)
            .or_default()
            .insert(permission);

        Ok(())
    }

    pub fn remove_role_permission(&mut self, role: &UserRole, permission: &str) -> ErpResult<()> {
        let permission = Permission::from_string(permission)?;

        if let Some(role_permissions) = self.role_permissions.get_mut(role) {
            role_permissions.remove(&permission);
        }

        Ok(())
    }

    pub fn is_higher_role(&self, role: &UserRole, other_role: &UserRole) -> bool {
        let role_hierarchy = [
            UserRole::Viewer,
            UserRole::Employee,
            UserRole::Manager,
            UserRole::Admin,
        ];

        let role_level = role_hierarchy.iter().position(|r| r == role).unwrap_or(0);
        let other_level = role_hierarchy
            .iter()
            .position(|r| r == other_role)
            .unwrap_or(0);

        role_level > other_level
    }

    pub fn can_manage_user(&self, manager_role: &UserRole, target_role: &UserRole) -> bool {
        match manager_role {
            UserRole::Admin => true, // Admin can manage all users
            UserRole::Manager => !matches!(target_role, UserRole::Admin), // Manager cannot manage Admin
            _ => false, // Only Admin and Manager can manage users
        }
    }

    pub fn validate_permission_string(permission: &str) -> ErpResult<()> {
        Permission::from_string(permission)?;
        Ok(())
    }

    pub fn filter_accessible_resources(
        &self,
        user_id: Uuid,
        role: &UserRole,
        resources: Vec<String>,
    ) -> Vec<String> {
        resources
            .into_iter()
            .filter(|resource| {
                self.can_access_resource(user_id, role, resource, "read")
                    .unwrap_or(false)
            })
            .collect()
    }

    fn check_permission_match(
        &self,
        permissions: &HashSet<Permission>,
        required_permission: &Permission,
    ) -> bool {
        // Check for exact match
        if permissions.contains(required_permission) {
            return true;
        }

        // Check for wildcard permissions
        let wildcard_resource = Permission::new(&required_permission.resource, "*");
        let wildcard_action = Permission::new("*", &required_permission.action);
        let wildcard_all = Permission::new("*", "*");

        if permissions.contains(&wildcard_resource)
            || permissions.contains(&wildcard_action)
            || permissions.contains(&wildcard_all)
        {
            return true;
        }

        // Check resource hierarchy
        self.check_hierarchical_permission(permissions, required_permission)
    }

    fn check_hierarchical_permission(
        &self,
        permissions: &HashSet<Permission>,
        required_permission: &Permission,
    ) -> bool {
        for (parent_resource, child_resources) in &self.resource_hierarchy {
            if child_resources.contains(&required_permission.resource) {
                let parent_permission =
                    Permission::new(parent_resource, &required_permission.action);
                if permissions.contains(&parent_permission) {
                    return true;
                }

                // Check wildcard on parent resource
                let parent_wildcard = Permission::new(parent_resource, "*");
                if permissions.contains(&parent_wildcard) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for RbacService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RoleBasedGuard {
    rbac_service: RbacService,
}

impl RoleBasedGuard {
    pub fn new(rbac_service: RbacService) -> Self {
        Self { rbac_service }
    }

    pub fn check_permission(
        &self,
        user_id: Uuid,
        role: &UserRole,
        permission: &str,
    ) -> ErpResult<()> {
        if self
            .rbac_service
            .has_permission(user_id, role, permission)?
        {
            Ok(())
        } else {
            Err(ErpError::Authorization(format!(
                "Access denied. Required permission: {}",
                permission
            )))
        }
    }

    pub fn check_resource_access(
        &self,
        user_id: Uuid,
        role: &UserRole,
        resource: &str,
        action: &str,
    ) -> ErpResult<()> {
        if self
            .rbac_service
            .can_access_resource(user_id, role, resource, action)?
        {
            Ok(())
        } else {
            Err(ErpError::Authorization(format!(
                "Access denied. Cannot {} {}",
                action, resource
            )))
        }
    }

    pub fn check_role_hierarchy(
        &self,
        manager_role: &UserRole,
        target_role: &UserRole,
    ) -> ErpResult<()> {
        if self.rbac_service.can_manage_user(manager_role, target_role) {
            Ok(())
        } else {
            Err(ErpError::Authorization(format!(
                "Insufficient privileges to manage user with role: {}",
                target_role
            )))
        }
    }

    pub fn require_any_permission(
        &self,
        user_id: Uuid,
        role: &UserRole,
        permissions: &[String],
    ) -> ErpResult<()> {
        if self
            .rbac_service
            .has_any_permission(user_id, role, permissions)?
        {
            Ok(())
        } else {
            Err(ErpError::Authorization(format!(
                "Access denied. Required any of: {:?}",
                permissions
            )))
        }
    }

    pub fn require_all_permissions(
        &self,
        user_id: Uuid,
        role: &UserRole,
        permissions: &[String],
    ) -> ErpResult<()> {
        if self
            .rbac_service
            .has_all_permissions(user_id, role, permissions)?
        {
            Ok(())
        } else {
            Err(ErpError::Authorization(format!(
                "Access denied. Required all of: {:?}",
                permissions
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_parsing() {
        let permission = Permission::from_string("users:create").unwrap();
        assert_eq!(permission.resource, "users");
        assert_eq!(permission.action, "create");
        assert_eq!(permission.to_string(), "users:create");

        assert!(Permission::from_string("invalid_permission").is_err());
    }

    #[test]
    fn test_role_based_permissions() {
        let rbac = RbacService::new();
        let user_id = Uuid::new_v4();

        // Admin should have all permissions
        assert!(rbac
            .has_permission(user_id, &UserRole::Admin, "users:create")
            .unwrap());
        assert!(rbac
            .has_permission(user_id, &UserRole::Admin, "system:admin")
            .unwrap());

        // Employee should have limited permissions
        assert!(rbac
            .has_permission(user_id, &UserRole::Employee, "products:read")
            .unwrap());
        assert!(!rbac
            .has_permission(user_id, &UserRole::Employee, "users:create")
            .unwrap());

        // Viewer should only have read permissions
        assert!(rbac
            .has_permission(user_id, &UserRole::Viewer, "products:read")
            .unwrap());
        assert!(!rbac
            .has_permission(user_id, &UserRole::Viewer, "products:create")
            .unwrap());
    }

    #[test]
    fn test_user_specific_permissions() {
        let mut rbac = RbacService::new();
        let user_id = Uuid::new_v4();

        // Viewer normally can't create products
        assert!(!rbac
            .has_permission(user_id, &UserRole::Viewer, "products:create")
            .unwrap());

        // Grant specific permission
        rbac.grant_user_permission(user_id, "products:create")
            .unwrap();
        assert!(rbac
            .has_permission(user_id, &UserRole::Viewer, "products:create")
            .unwrap());

        // Revoke permission
        rbac.revoke_user_permission(user_id, "products:create")
            .unwrap();
        assert!(!rbac
            .has_permission(user_id, &UserRole::Viewer, "products:create")
            .unwrap());
    }

    #[test]
    fn test_role_hierarchy() {
        let rbac = RbacService::new();

        assert!(rbac.is_higher_role(&UserRole::Admin, &UserRole::Manager));
        assert!(rbac.is_higher_role(&UserRole::Manager, &UserRole::Employee));
        assert!(rbac.is_higher_role(&UserRole::Employee, &UserRole::Viewer));
        assert!(!rbac.is_higher_role(&UserRole::Viewer, &UserRole::Employee));

        assert!(rbac.can_manage_user(&UserRole::Admin, &UserRole::Manager));
        assert!(rbac.can_manage_user(&UserRole::Manager, &UserRole::Employee));
        assert!(!rbac.can_manage_user(&UserRole::Manager, &UserRole::Admin));
        assert!(!rbac.can_manage_user(&UserRole::Employee, &UserRole::Manager));
    }

    #[test]
    fn test_permission_collections() {
        let rbac = RbacService::new();
        let user_id = Uuid::new_v4();

        let permissions = vec![
            "products:read".to_string(),
            "products:create".to_string(),
            "users:create".to_string(),
        ];

        // Admin should have all permissions
        assert!(rbac
            .has_all_permissions(user_id, &UserRole::Admin, &permissions)
            .unwrap());

        // Employee should have some but not all
        assert!(rbac
            .has_any_permission(user_id, &UserRole::Employee, &permissions)
            .unwrap());
        assert!(!rbac
            .has_all_permissions(user_id, &UserRole::Employee, &permissions)
            .unwrap());

        // Viewer should have only read permission
        let read_permissions = vec!["products:read".to_string()];
        assert!(rbac
            .has_all_permissions(user_id, &UserRole::Viewer, &read_permissions)
            .unwrap());
    }

    #[test]
    fn test_resource_filtering() {
        let rbac = RbacService::new();
        let user_id = Uuid::new_v4();

        let all_resources = vec![
            "users".to_string(),
            "products".to_string(),
            "customers".to_string(),
            "orders".to_string(),
            "reports".to_string(),
            "config".to_string(),
        ];

        // Admin can access all resources
        let admin_accessible =
            rbac.filter_accessible_resources(user_id, &UserRole::Admin, all_resources.clone());
        assert_eq!(admin_accessible.len(), all_resources.len());

        // Viewer has limited access
        let viewer_accessible =
            rbac.filter_accessible_resources(user_id, &UserRole::Viewer, all_resources.clone());
        assert!(viewer_accessible.len() < all_resources.len());
        assert!(viewer_accessible.contains(&"products".to_string()));
        assert!(!viewer_accessible.contains(&"config".to_string()));
    }

    #[test]
    fn test_role_based_guard() {
        let rbac = RbacService::new();
        let guard = RoleBasedGuard::new(rbac);
        let user_id = Uuid::new_v4();

        // Admin should pass all checks
        assert!(guard
            .check_permission(user_id, &UserRole::Admin, "users:create")
            .is_ok());
        assert!(guard
            .check_resource_access(user_id, &UserRole::Admin, "users", "create")
            .is_ok());
        assert!(guard
            .check_role_hierarchy(&UserRole::Admin, &UserRole::Manager)
            .is_ok());

        // Employee should fail some checks
        assert!(guard
            .check_permission(user_id, &UserRole::Employee, "users:create")
            .is_err());
        assert!(guard
            .check_resource_access(user_id, &UserRole::Employee, "users", "create")
            .is_err());

        // Test multiple permissions
        let required_permissions = vec!["products:read".to_string(), "products:update".to_string()];
        assert!(guard
            .require_all_permissions(user_id, &UserRole::Employee, &required_permissions)
            .is_ok());

        let admin_permissions = vec!["users:create".to_string(), "config:update".to_string()];
        assert!(guard
            .require_any_permission(user_id, &UserRole::Employee, &admin_permissions)
            .is_err());
        assert!(guard
            .require_any_permission(user_id, &UserRole::Admin, &admin_permissions)
            .is_ok());
    }

    #[test]
    fn test_dynamic_role_permissions() {
        let mut rbac = RbacService::new();

        // Add new permission to a role
        rbac.add_role_permission(UserRole::Employee, "special:access")
            .unwrap();

        let user_id = Uuid::new_v4();
        assert!(rbac
            .has_permission(user_id, &UserRole::Employee, "special:access")
            .unwrap());

        // Remove the permission
        rbac.remove_role_permission(&UserRole::Employee, "special:access")
            .unwrap();
        assert!(!rbac
            .has_permission(user_id, &UserRole::Employee, "special:access")
            .unwrap());
    }
}
