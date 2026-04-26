mod entity;
mod repository;
mod service;

use sword::prelude::*;

pub use repository::*;
pub use service::*;

pub struct SnapshotsModule;

impl Module for SnapshotsModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<SnapshotRepository>();
        components.register::<SnapshotService>();
    }
}
