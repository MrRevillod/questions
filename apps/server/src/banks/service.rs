use crate::banks::*;
use crate::courses::CourseId;
use crate::quizzes::Quiz;
use crate::shared::Tx;
use crate::shared::{AppResult, TransactionManager};
use crate::snapshots::SnapshotService;
use crate::users::User;

use chrono::Utc;
use std::sync::Arc;
use sword::prelude::*;

#[injectable]
pub struct QuestionBankService {
    policy: Arc<QuestionBankPolicy>,
    repository: Arc<QuestionBankRepository>,
    snapshots: Arc<SnapshotService>,
    tx: Arc<TransactionManager>,
}

impl QuestionBankService {
    pub async fn list_for_course(
        &self,
        current_user: &User,
        course_id: &CourseId,
    ) -> AppResult<Vec<QuestionBank>> {
        self.policy
            .require_accessible_course(current_user, course_id)
            .await?;

        let banks = self.repository.list_by_course(course_id).await?;

        Ok(banks)
    }

    pub async fn get_one(
        &self,
        current_user: &User,
        bank_id: &QuestionBankId,
    ) -> AppResult<QuestionBank> {
        let bank = self
            .policy
            .require_accessible_bank(current_user, bank_id)
            .await?;

        Ok(bank)
    }

    pub async fn create(&self, current_user: &User, input: CreateQuestionBankDto) -> AppResult<()> {
        self.policy
            .require_accessible_course(current_user, &input.course_id)
            .await?;

        let questions = input
            .questions
            .iter()
            .map(Question::from)
            .collect::<Vec<_>>();

        let bank = QuestionBank::builder()
            .course_id(input.course_id)
            .name(input.name)
            .questions(questions)
            .created_at(Utc::now())
            .build();

        let mut tx = self.tx.begin().await?;

        self.repository.save(&mut tx, &bank).await?;

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
            bank.questions = questions.iter().map(Question::from).collect();
        }

        let linked_quizzes = self.snapshots.list_linked_quizzes(&bank.id).await?;
        self.ensure_not_linked_to_running_quiz(&linked_quizzes)?;

        let mut tx = self.tx.begin().await?;

        self.repository.save(&mut tx, &bank).await?;
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

        let linked_quizzes = self.snapshots.list_linked_quizzes(&bank.id).await?;
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

    fn ensure_not_linked_to_running_quiz(&self, quizzes: &[Quiz]) -> AppResult<()> {
        let now = Utc::now();

        if quizzes
            .iter()
            .any(|quiz| quiz.closed_at.is_none() && quiz.starts_at <= now)
        {
            return Err(QuestionBankError::LockedByRunningQuiz)?;
        }

        Ok(())
    }

    async fn sync_not_started_snapshots(&self, tx: &mut Tx<'_>, quizzes: &[Quiz]) -> AppResult<()> {
        let now = Utc::now();

        for quiz in quizzes {
            if quiz.closed_at.is_some() || quiz.starts_at <= now {
                continue;
            }

            let questions = self
                .snapshots
                .list_questions_for_linked_banks(&quiz.id)
                .await?;

            if quiz.question_count as usize > questions.len() {
                return Err(QuestionBankError::InvalidQuestionCountAfterBankUpdate)?;
            }

            if !self
                .snapshots
                .update_questions(tx, quiz.snapshot_id, &questions)
                .await?
            {
                return Err(QuestionBankError::SnapshotNotFound)?;
            }
        }

        Ok(())
    }
}
