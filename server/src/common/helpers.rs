use uuid::Uuid;

use super::error::{AppError, AppResult};

/// Check that the resource belongs to the authenticated user.
/// Returns NotFound error (not Forbidden) to avoid leaking existence of resources.
pub fn check_ownership(
    resource_user_id: Uuid,
    auth_user_id: Uuid,
    resource_name: &str,
    resource_id: Uuid,
) -> AppResult<()> {
    if resource_user_id != auth_user_id {
        Err(AppError::NotFound(format!(
            "{resource_name} {resource_id} not found"
        )))
    } else {
        Ok(())
    }
}
