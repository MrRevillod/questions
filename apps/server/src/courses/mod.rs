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

use controller::CoursesController;
use sword::prelude::*;

pub struct CoursesModule;

impl Module for CoursesModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<CoursesController>();
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<CoursePolicy>();
        components.register::<CourseRepository>();
        components.register::<CoursesService>();
    }
}
