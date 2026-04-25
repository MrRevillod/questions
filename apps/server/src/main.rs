mod attempts;
mod auth;
mod authz;
mod banks;
mod courses;
mod logger;
mod quizzes;
mod shared;
mod users;

use attempts::AttemptsModule;
use auth::AuthModule;
use authz::AuthzModule;
use banks::QuestionBankModule;
use courses::CoursesModule;
use quizzes::QuizzesModule;
use shared::SharedModule;
use users::UsersModule;

use logger::LoggerLayer;
use sword::prelude::*;

#[sword::main]
async fn main() {
    let app = Application::builder()
        .with_module::<AuthModule>()
        .with_module::<AuthzModule>()
        .with_module::<QuestionBankModule>()
        .with_module::<CoursesModule>()
        .with_module::<AttemptsModule>()
        .with_module::<UsersModule>()
        .with_module::<QuizzesModule>()
        .with_module::<SharedModule>()
        .with_layer(LoggerLayer())
        .build();

    app.run().await;
}
