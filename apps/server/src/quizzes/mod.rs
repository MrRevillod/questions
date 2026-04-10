mod controller;
mod dtos;
mod entity;
mod errors;
mod policy;
mod repository;
mod service;
mod views;

pub use dtos::*;
pub use entity::*;
pub use errors::*;
pub use policy::*;
pub use repository::*;
pub use service::*;
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
    }
}
