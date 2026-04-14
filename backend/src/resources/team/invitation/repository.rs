use async_trait::async_trait;
use surrealdb::sql::Thing;

use crate::error::AppError;

use super::model::{InvitationAcceptRow, InvitationRow};

/// Pure invitation data access — no authorization. Service layer does all ACL checks.
#[async_trait]
pub trait TeamInvitationRepository: Send + Sync {
    /// Insert a new invitation record.
    async fn create_invitation(
        &self,
        team: Thing,
        created_by: Thing,
        inv_id: &str,
    ) -> Result<(), AppError>;

    /// List all invitations for a team (ordered by created_at ASC, FETCH created_by).
    async fn list_invitations(&self, team: Thing) -> Result<Vec<InvitationRow>, AppError>;

    /// Get a single invitation with created_by fetched.
    async fn get_invitation(&self, inv_id: &str) -> Result<Option<InvitationRow>, AppError>;

    /// Delete an invitation and return whether it existed.
    async fn delete_invitation(&self, inv_id: &str) -> Result<bool, AppError>;

    /// Fetch an invitation with the team record fully FETCHed (for accept flow).
    async fn get_invitation_with_team(
        &self,
        inv_id: &str,
    ) -> Result<Option<InvitationAcceptRow>, AppError>;
}
