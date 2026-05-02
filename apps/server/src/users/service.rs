use crate::{shared::AppResult, users::*};
use sword::prelude::*;

#[injectable]
pub struct UsersService {
    policy: UserPolicy,
    users: UserRepository,
}

impl UsersService {
    pub async fn list_users(&self, query: SearchUsersQuery) -> AppResult<Vec<User>> {
        self.users.list_users(UserFilter::from(query)).await
    }

    pub async fn list_collaborator_candidates(
        &self,
        query: SearchUsersQuery,
    ) -> AppResult<Vec<User>> {
        let filter = UserFilter::from(query);

        if let Some(roles) = &filter.roles
            && roles.contains(&UserRole::Student)
        {
            return Err(UsersError::InvalidUserRole)?;
        }

        self.users.list_users(filter).await
    }

    pub async fn update_role(
        &self,
        user_id: &UserId,
        input: UpdateUserRoleRequest,
    ) -> AppResult<User> {
        let mut target = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| UsersError::NotFound(user_id.to_string()))?;

        self.policy.can_assign_assistant_role(&target)?;

        target.role = UserRole::from(input.role);

        self.users.save(&target).await
    }
}
