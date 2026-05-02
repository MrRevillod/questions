mod controller;
mod dtos;
mod entity;
mod errors;
mod policy;
mod repository;
mod service;

use sword::prelude::*;

pub use controller::*;
pub use dtos::*;
pub use entity::*;
pub use errors::*;
pub use policy::*;
pub use repository::*;
pub use service::*;

pub struct QuestionBankModule;

impl Module for QuestionBankModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<QuestionBankController>();
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<QuestionBankPolicy>();
        components.register::<QuestionBankRepository>();
        components.register::<QuestionBankService>();
    }
}
