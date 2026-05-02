mod codegen;
mod policy;

pub use codegen::*;
pub use policy::*;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::attempts::{AttemptResultView, AttemptsService};
use crate::banks::QuestionBankRepository;
use crate::courses::CourseRepository;
use crate::quizzes::*;
use crate::shared::{AppResult, TransactionManager};
use crate::snapshots::SnapshotService;
use crate::users::{User, UserRole};

use chrono::{DateTime, Utc};
use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct QuizService {
    codegen: Arc<QuizCodeGenerator>,
    policy: Arc<QuizPolicy>,
    repository: Arc<QuizRepository>,
    courses: Arc<CourseRepository>,
    snapshots: Arc<SnapshotService>,
    tx: Arc<TransactionManager>,
    banks: Arc<QuestionBankRepository>,
    attempts: Arc<AttemptsService>,
}

impl QuizService {
    pub async fn get_one(&self, current_user: &User, quiz_id: &QuizId) -> AppResult<QuizView> {
        let quiz = self
            .policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;

        let course = self
            .courses
            .find_by_id(&quiz.course_id)
            .await?
            .ok_or_else(|| QuizError::NotFound(quiz.course_id.to_string()))?;

        Ok(QuizView::from((quiz, course)))
    }

    pub async fn list_managed_by_user(&self, current_user: &User) -> AppResult<Vec<QuizView>> {
        let quizzes = self
            .repository
            .list_managed_by_user(&current_user.id)
            .await?;

        let course_ids = quizzes
            .iter()
            .map(|quiz| quiz.course_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let courses_by_id = self
            .courses
            .find_by_ids(&course_ids)
            .await?
            .into_iter()
            .map(|course| (course.id, course))
            .collect::<HashMap<_, _>>();

        let views = quizzes
            .into_iter()
            .filter_map(|quiz| {
                let course = courses_by_id.get(&quiz.course_id)?.clone();
                Some(QuizView::from((quiz, course)))
            })
            .collect::<Vec<_>>();

        Ok(views)
    }

    pub async fn get_join_preview(&self, code: &str) -> AppResult<JoinQuizPreviewView> {
        let Some(quiz) = self.repository.find_by_code(code).await? else {
            return Err(QuizError::NotFound(code.to_string()))?;
        };

        if quiz.closed_at.is_some() {
            return Err(QuizError::Closed)?;
        }

        Ok(JoinQuizPreviewView::from(&quiz))
    }

    pub async fn get_my_result_by_join_code(
        &self,
        current_user: &User,
        code: &str,
    ) -> AppResult<AttemptResultView> {
        let Some(quiz) = self.repository.find_by_code(code).await? else {
            return Err(QuizError::NotFound(code.to_string()))?;
        };

        if current_user.role != UserRole::Admin
            && !self
                .courses
                .is_member(&quiz.course_id, &current_user.id)
                .await?
        {
            return Err(QuizError::Forbidden)?;
        }

        let attempt = self
            .attempts
            .get_attempt_for_quiz_and_student(&quiz.id, &current_user.id)
            .await?;

        self.attempts
            .view_results(attempt.id, current_user.id)
            .await
    }

    pub async fn create(&self, current_user: &User, input: CreateQuizDto) -> AppResult<QuizView> {
        self.policy
            .check_can_create_quiz(current_user, &input.course_id)
            .await?;

        if !self
            .banks
            .are_banks_in_course(&input.bank_ids, &input.course_id)
            .await?
        {
            return Err(QuizError::InvalidBanksForCourse)?;
        }

        let questions = self
            .banks
            .list_questions_by_bank_ids(&input.bank_ids)
            .await?;

        if input.question_count as usize > questions.len() {
            return Err(QuizError::InvalidQuestionCount)?;
        }

        let starts_at = DateTime::parse_from_rfc3339(&input.starts_at)
            .map_err(|_| QuizError::InvalidStartTime)?
            .with_timezone(&Utc);

        let snapshot_id = Uuid::new_v4();

        let quiz = Quiz::builder()
            .course_id(input.course_id)
            .snapshot_id(snapshot_id)
            .title(input.title)
            .kind(input.kind)
            .join_code(self.codegen.generate_unique_join_code().await?)
            .question_count(input.question_count)
            .maybe_certainty_table(input.certainty_config.map(CertaintyTable::from))
            .attempt_duration_minutes(input.attempt_duration_minutes)
            .starts_at(starts_at)
            .created_at(Utc::now())
            .build();

        let mut tx = self.tx.begin().await?;

        self.snapshots
            .create_snapshot(&mut tx, snapshot_id, &questions)
            .await?;

        let quiz = self.repository.save(&mut tx, &quiz).await?;

        self.repository
            .set_bank_links(&mut tx, &quiz.id, &input.bank_ids)
            .await?;

        tx.commit().await?;

        let course = self
            .courses
            .find_by_id(&quiz.course_id)
            .await?
            .ok_or_else(|| QuizError::NotFound(quiz.course_id.to_string()))?;

        Ok(QuizView::from((quiz, course)))
    }

    pub async fn delete_quiz(&self, current_user: &User, quiz_id: &QuizId) -> AppResult<()> {
        let quiz = self
            .policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;

        if !self.repository.delete_by_id(&quiz.id).await? {
            return Err(QuizError::NotFound(quiz_id.to_string()))?;
        }

        Ok(())
    }
}
