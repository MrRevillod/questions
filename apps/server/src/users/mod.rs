mod controller;
mod dtos;
mod entity;
mod errors;
mod policy;
mod repository;
mod service;

use controller::UsersController;
use sword::prelude::*;

pub use dtos::{SearchUsersQuery, UpdateUserRoleRequest};
pub use entity::{User, UserRole};
pub use errors::UsersError;
pub use policy::UserPolicy;
pub use repository::UserRepository;
pub use service::UsersService;

pub struct UsersModule;

impl Module for UsersModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<UserPolicy>();
        components.register::<UserRepository>();
        components.register::<UsersService>();
    }
}
