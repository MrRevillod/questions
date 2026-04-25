mod controller;
mod dtos;
mod entity;
mod errors;
mod repository;
mod services;
mod views;

pub use dtos::*;
pub use entity::*;
pub use errors::*;
pub use repository::*;
pub use services::*;
pub use views::*;

use controller::QuizController;
use sword::prelude::*;

pub struct QuizzesModule;

impl Module for QuizzesModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<QuizController>();
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<QuizPolicy>();
        components.register::<QuizRepository>();
        components.register::<QuizService>();
        components.register::<QuizCodeGenerator>();
    }
}
