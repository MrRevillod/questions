mod database;
mod errors;
mod jsonwebtoken;

use database::DatabaseConfig;
use sword::prelude::*;

pub use database::Database;
pub use errors::*;
pub use jsonwebtoken::JsonWebTokenService;

pub struct SharedModule;

impl Module for SharedModule {
    async fn register_providers(config: &Config, providers: &ProviderRegistry) {
        let db_config = config.expect::<DatabaseConfig>();
        let database = Database::new(db_config).await;

        providers.register(database);
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<JsonWebTokenService>();
    }
}
