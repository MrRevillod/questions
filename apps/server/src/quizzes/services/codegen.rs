use rand::{RngExt, distr::Alphanumeric};
use sword::prelude::*;

use crate::{
    quizzes::{QuizError, QuizRepository},
    shared::AppResult,
};

const JOIN_CODE_LENGTH: usize = 8;
const JOIN_CODE_MAX_ATTEMPTS: usize = 10;

#[injectable]
pub struct QuizCodeGenerator {
    repository: QuizRepository,
}

impl QuizCodeGenerator {
    pub async fn generate_unique_join_code(&self) -> AppResult<String> {
        for _ in 0..JOIN_CODE_MAX_ATTEMPTS {
            let join_code = self.generate_join_code_candidate();

            if self.repository.find_by_code(&join_code).await?.is_none() {
                return Ok(join_code);
            }
        }

        Err(QuizError::InvalidCode)?
    }

    fn generate_join_code_candidate(&self) -> String {
        rand::rng()
            .sample_iter(Alphanumeric)
            .take(JOIN_CODE_LENGTH)
            .map(char::from)
            .collect::<String>()
            .to_uppercase()
    }
}
