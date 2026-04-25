use crate::banks::*;
use crate::shared::{AppResult, TransactionManager};
use crate::users::User;

use chrono::Utc;
use std::sync::Arc;
use sword::prelude::*;

#[injectable]
pub struct QuestionBankService {
    policy: Arc<QuestionBankPolicy>,
    repository: Arc<QuestionBankRepository>,
    tx: Arc<TransactionManager>,
}

impl QuestionBankService {
    pub async fn list_for_course(
        &self,
        current_user: &User,
        course_id: &crate::courses::CourseId,
    ) -> AppResult<Vec<QuestionBankView>> {
        self.policy
            .require_accessible_course(current_user, course_id)
            .await?;

        let banks = self.repository.list_by_course(course_id).await?;

        Ok(banks.into_iter().map(QuestionBankView::from).collect())
    }

    pub async fn get_one(
        &self,
        current_user: &User,
        bank_id: &QuestionBankId,
    ) -> AppResult<QuestionBankView> {
        let bank = self
            .policy
            .require_accessible_bank(current_user, bank_id)
            .await?;

        Ok(QuestionBankView::from(bank))
    }

    pub async fn create(&self, current_user: &User, input: CreateQuestionBankDto) -> AppResult<()> {
        self.policy
            .require_accessible_course(current_user, &input.course_id)
            .await?;

        let bank = QuestionBank::builder()
            .course_id(input.course_id)
            .name(input.name)
            .questions(
                input
                    .questions
                    .iter()
                    .map(QuestionBankQuestion::from)
                    .collect(),
            )
            .created_at(Utc::now())
            .build();

        let mut tx = self.tx.begin().await?;
        self.repository.create(&mut tx, &bank).await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn update(
        &self,
        current_user: &User,
        bank_id: &QuestionBankId,
        input: UpdateQuestionBankDto,
    ) -> AppResult<()> {
        let mut bank = self
            .policy
            .require_accessible_bank(current_user, bank_id)
            .await?;

        if let Some(name) = input.name {
            bank.name = name;
        }

        if let Some(questions) = input.questions {
            bank.questions = questions.iter().map(QuestionBankQuestion::from).collect();
        }

        let linked_quizzes = self.repository.list_linked_quizzes(&bank.id).await?;
        self.ensure_not_linked_to_running_quiz(&linked_quizzes)?;

        let mut tx = self.tx.begin().await?;
        self.repository.update(&mut tx, &bank).await?;
        self.sync_not_started_snapshots(&mut tx, &linked_quizzes)
            .await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn soft_delete(
        &self,
        current_user: &User,
        bank_id: &QuestionBankId,
    ) -> AppResult<()> {
        let bank = self
            .policy
            .require_accessible_bank(current_user, bank_id)
            .await?;

        let linked_quizzes = self.repository.list_linked_quizzes(&bank.id).await?;
        self.ensure_not_linked_to_running_quiz(&linked_quizzes)?;

        let mut tx = self.tx.begin().await?;

        if !self.repository.soft_delete(&mut tx, &bank.id).await? {
            return Err(QuestionBankError::NotFound(bank.id.to_string()))?;
        }

        self.sync_not_started_snapshots(&mut tx, &linked_quizzes)
            .await?;
        tx.commit().await?;

        Ok(())
    }

    fn ensure_not_linked_to_running_quiz(&self, quizzes: &[LinkedQuiz]) -> AppResult<()> {
        let now = Utc::now();

        if quizzes
            .iter()
            .any(|quiz| quiz.closed_at.is_none() && quiz.starts_at <= now)
        {
            return Err(QuestionBankError::LockedByRunningQuiz)?;
        }

        Ok(())
    }

    async fn sync_not_started_snapshots(
        &self,
        tx: &mut crate::shared::Tx<'_>,
        quizzes: &[LinkedQuiz],
    ) -> AppResult<()> {
        let now = Utc::now();

        for quiz in quizzes {
            if quiz.closed_at.is_some() || quiz.starts_at <= now {
                continue;
            }

            let questions = self.repository.list_questions_for_quiz(&quiz.id).await?;

            if quiz.question_count as usize > questions.len() {
                return Err(QuestionBankError::InvalidQuestionCountAfterBankUpdate)?;
            }

            if !self
                .repository
                .update_snapshot_questions(tx, &quiz.id, &questions)
                .await?
            {
                return Err(QuestionBankError::SnapshotNotFound)?;
            }
        }

        Ok(())
    }
}
