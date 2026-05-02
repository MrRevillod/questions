mod controller;
mod dtos;
mod entity;
mod errors;
mod repository;
mod services;

use sword::prelude::*;

pub use controller::AttemptsController;
pub use dtos::*;
pub use entity::*;
pub use errors::AttemptError;
pub use repository::AttemptRepository;
pub use services::*;

pub struct AttemptsModule;

impl Module for AttemptsModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<AttemptRepository>();
        components.register::<AttemptsService>();
        components.register::<QuestionService>();
        components.register::<GradingService>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<AttemptsController>();
    }
}
