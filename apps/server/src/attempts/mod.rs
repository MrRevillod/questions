mod controller;
mod dtos;
mod entity;
mod errors;
mod policy;
mod repository;
mod service;

pub use dtos::*;
pub use entity::*;
pub use errors::*;
pub use policy::*;
pub use repository::*;
pub use service::*;

use controller::AttemptController;
use sword::prelude::*;

pub struct AttemptsModule;

impl Module for AttemptsModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<AttemptController>();
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<AttemptPolicy>();
        components.register::<AttemptRepository>();
        components.register::<AttemptService>();
    }
}
