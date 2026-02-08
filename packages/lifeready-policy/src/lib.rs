use axum::response::Response;
use lifeready_auth::{AuthError, RequestContext, RequestId};

pub use lifeready_auth::{Role, SensitivityTier};

#[derive(Debug, Clone)]
pub enum TierRequirement {
    Min(SensitivityTier),
    Allowlist(Vec<SensitivityTier>),
}

#[derive(Debug, Clone)]
pub enum PolicyError {
    Forbidden { detail: String },
}

impl PolicyError {
    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self::Forbidden {
            detail: detail.into(),
        }
    }

    pub fn into_response(self, request_id: Option<RequestId>) -> Response {
        match self {
            PolicyError::Forbidden { detail } => {
                AuthError::forbidden(detail).into_response(request_id)
            }
        }
    }
}

pub fn require_role(ctx: &RequestContext, allowed_roles: &[Role]) -> Result<(), PolicyError> {
    if allowed_roles.is_empty() {
        return Ok(());
    }

    if ctx.roles.iter().any(|role| allowed_roles.contains(role)) {
        Ok(())
    } else {
        Err(PolicyError::forbidden("role not permitted"))
    }
}

pub fn require_tier(ctx: &RequestContext, requirement: TierRequirement) -> Result<(), PolicyError> {
    match requirement {
        TierRequirement::Min(min_tier) => {
            let required_rank = tier_rank(min_tier);
            let max_rank = ctx
                .allowed_tiers
                .iter()
                .map(|tier| tier_rank(*tier))
                .max()
                .unwrap_or(0);
            if max_rank >= required_rank {
                Ok(())
            } else {
                Err(PolicyError::forbidden("insufficient tier access"))
            }
        }
        TierRequirement::Allowlist(allowed) => {
            if allowed.iter().any(|tier| ctx.allowed_tiers.contains(tier)) {
                Ok(())
            } else {
                Err(PolicyError::forbidden("tier not permitted"))
            }
        }
    }
}

pub fn require_scope(ctx: &RequestContext, required_scope: &str) -> Result<(), PolicyError> {
    if required_scope.trim().is_empty() {
        return Ok(());
    }

    if ctx.scopes.iter().any(|scope| scope == required_scope) {
        Ok(())
    } else {
        Err(PolicyError::forbidden("scope not permitted"))
    }
}

pub fn require_scope_any(
    ctx: &RequestContext,
    required_scopes: &[&str],
) -> Result<(), PolicyError> {
    if required_scopes.is_empty() {
        return Ok(());
    }

    if ctx
        .scopes
        .iter()
        .any(|scope| required_scopes.iter().any(|required| scope == required))
    {
        Ok(())
    } else {
        Err(PolicyError::forbidden("scope not permitted"))
    }
}

fn tier_rank(tier: SensitivityTier) -> u8 {
    match tier {
        SensitivityTier::Green => 0,
        SensitivityTier::Amber => 1,
        SensitivityTier::Red => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lifeready_auth::RequestId;
    use uuid::Uuid;

    fn ctx(roles: Vec<Role>, tiers: Vec<SensitivityTier>, scopes: Vec<&str>) -> RequestContext {
        RequestContext {
            request_id: RequestId(Uuid::new_v4()),
            principal_id: "user".into(),
            roles,
            allowed_tiers: tiers,
            scopes: scopes.into_iter().map(|s| s.to_string()).collect(),
            expires_at: chrono::Utc::now(),
            email: None,
        }
    }

    #[test]
    fn rbac_role_matrix() {
        let cases = vec![
            (
                vec![Role::Principal],
                vec![Role::Principal, Role::Proxy],
                true,
            ),
            (vec![Role::Proxy], vec![Role::Principal, Role::Proxy], true),
            (
                vec![Role::ExecutorNominee],
                vec![Role::Principal, Role::Proxy],
                false,
            ),
            (vec![Role::EmergencyContact], vec![Role::Principal], false),
        ];

        for (roles, allowed, ok) in cases {
            let ctx = ctx(roles, vec![SensitivityTier::Green], vec!["read:all"]);
            let result = require_role(&ctx, &allowed);
            assert_eq!(result.is_ok(), ok);
        }
    }

    #[test]
    fn rbac_tier_matrix() {
        let cases = vec![
            (vec![SensitivityTier::Green], SensitivityTier::Green, true),
            (vec![SensitivityTier::Amber], SensitivityTier::Green, true),
            (vec![SensitivityTier::Green], SensitivityTier::Amber, false),
            (vec![SensitivityTier::Amber], SensitivityTier::Red, false),
            (vec![SensitivityTier::Red], SensitivityTier::Red, true),
        ];

        for (tiers, required, ok) in cases {
            let ctx = ctx(vec![Role::Principal], tiers, vec!["read:all"]);
            let result = require_tier(&ctx, TierRequirement::Min(required));
            assert_eq!(result.is_ok(), ok);
        }
    }

    #[test]
    fn rbac_tier_allowlist_matrix() {
        let cases = vec![
            (
                vec![SensitivityTier::Green],
                vec![SensitivityTier::Green],
                true,
            ),
            (
                vec![SensitivityTier::Amber],
                vec![SensitivityTier::Green, SensitivityTier::Amber],
                true,
            ),
            (
                vec![SensitivityTier::Red],
                vec![SensitivityTier::Green, SensitivityTier::Amber],
                false,
            ),
            (
                vec![SensitivityTier::Green, SensitivityTier::Amber],
                vec![SensitivityTier::Red],
                false,
            ),
            (
                vec![SensitivityTier::Green, SensitivityTier::Red],
                vec![SensitivityTier::Red],
                true,
            ),
        ];

        for (tiers, allowlist, ok) in cases {
            let ctx = ctx(vec![Role::Principal], tiers, vec!["read:all"]);
            let result = require_tier(&ctx, TierRequirement::Allowlist(allowlist));
            assert_eq!(result.is_ok(), ok);
        }
    }

    #[test]
    fn rbac_scope_matrix() {
        let cases = vec![
            (vec!["read:all"], "read:all", true),
            (vec!["write:limited"], "read:all", false),
            (vec!["read:packs", "read:all"], "read:all", true),
        ];

        for (scopes, required, ok) in cases {
            let ctx = ctx(vec![Role::Principal], vec![SensitivityTier::Green], scopes);
            let result = require_scope(&ctx, required);
            assert_eq!(result.is_ok(), ok);
        }
    }

    #[test]
    fn rbac_scope_any_matrix() {
        let cases = vec![
            (vec!["read:all"], vec!["read:packs", "read:all"], true),
            (vec!["read:packs"], vec!["read:packs", "read:all"], true),
            (vec!["write:limited"], vec!["read:packs", "read:all"], false),
        ];

        for (scopes, required, ok) in cases {
            let ctx = ctx(vec![Role::Principal], vec![SensitivityTier::Green], scopes);
            let result = require_scope_any(&ctx, &required);
            assert_eq!(result.is_ok(), ok);
        }
    }
}
