use std::sync::Arc;

use crate::infrastructure::repositories::DatabaseTransformationRepository;

pub struct DatabaseTransformationUseCase {
    repository: Arc<DatabaseTransformationRepository>,
}

impl DatabaseTransformationUseCase {
    pub fn new(repository: Arc<DatabaseTransformationRepository>) -> Self {
        Self { repository }
    }
}
