mod grading;
mod questions;

use crate::{
    attempts::*,
    banks::Question,
    courses::CoursePolicy,
    quizzes::{QuizError, QuizId, QuizKind, QuizRepository},
    shared::AppResult,
    users::{User, UserId},
};

use chrono::{Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use sword::prelude::*;
use uuid::Uuid;

pub use grading::{GradingOutput, GradingService};
pub use questions::QuestionService;

use crate::banks::QuestionView;

#[injectable]
pub struct AttemptsService {
    repository: Arc<AttemptRepository>,
    quizzes: Arc<QuizRepository>,
    course_policy: Arc<CoursePolicy>,
    questions_service: Arc<QuestionService>,
    grading: Arc<GradingService>,
}

impl AttemptsService {
    pub async fn list_attempts(
        &self,
        current_user: &User,
        filter: AttemptFilter,
    ) -> AppResult<Vec<Attempt>> {
        self.course_policy
            .require_func_member(current_user, &filter.course_id)
            .await?;

        self.repository.list_attempts(filter).await
    }

    pub async fn get_attempt_for_user(
        &self,
        attempt_id: AttemptId,
        user_id: UserId,
    ) -> AppResult<Attempt> {
        let Some(attempt) = self.repository.find_by_id(&attempt_id).await? else {
            return Err(AttemptError::NotFound(attempt_id))?;
        };

        if attempt.student_id != user_id {
            return Err(AttemptError::Forbidden)?;
        }

        Ok(attempt)
    }

    pub async fn get_attempt_for_quiz_and_student(
        &self,
        quiz_id: &QuizId,
        student_id: &UserId,
    ) -> AppResult<Attempt> {
        self.repository
            .find_by_quiz_and_student(quiz_id, student_id)
            .await?
            .ok_or(AttemptError::NotFoundForQuiz.into())
    }

    pub async fn initialize_attempt(&self, quiz_id: QuizId, user_id: UserId) -> AppResult<Attempt> {
        let Some(quiz) = self.quizzes.find_by_id(&quiz_id).await? else {
            return Err(QuizError::NotFound(quiz_id.to_string()))?;
        };

        let now = Utc::now();

        if quiz.starts_at > now {
            return Err(AttemptError::QuizNotStarted)?;
        }

        if quiz.closed_at.is_some() {
            return Err(AttemptError::QuizEnded)?;
        }

        let filter = AttemptFilter {
            course_id: quiz.course_id,
            quiz_id: quiz.id,
            student_id: Some(user_id),
        };

        let attempts = self.repository.list_attempts(filter).await?;

        if !attempts.is_empty() {
            return Err(AttemptError::AlreadyAttempted)?;
        }

        let question_order = self
            .questions_service
            .get_question_ids_for_attempt(&quiz.id, quiz.question_count as usize)
            .await?;

        let expires_at = now + Duration::minutes(quiz.attempt_duration_minutes as i64);

        let attempt = Attempt::builder()
            .student_id(user_id)
            .quiz_id(quiz.id)
            .question_order(question_order)
            .started_at(now)
            .expires_at(expires_at)
            .build();

        self.repository.save(&attempt).await?;

        Ok(attempt)
    }

    pub async fn get_initialize_response(
        &self,
        attempt: &Attempt,
    ) -> AppResult<(AttemptSnapshotView, Vec<QuestionView>)> {
        let questions = self
            .questions_service
            .get_ordered_question_views(&attempt.quiz_id, &attempt.question_order)
            .await?;

        Ok((AttemptSnapshotView::from(attempt), questions))
    }

    pub async fn save_answer(
        &self,
        attempt_id: AttemptId,
        question_id: Uuid,
        user_id: UserId,
        input: SaveAttemptAnswerDto,
    ) -> AppResult<()> {
        let attempt = self.get_attempt_for_user(attempt_id, user_id).await?;

        let Some(quiz) = self.quizzes.find_by_id(&attempt.quiz_id).await? else {
            return Err(QuizError::NotFound(attempt.quiz_id.to_string()))?;
        };

        match quiz.kind {
            QuizKind::Certainty if input.certainty_level.is_none() => {
                return Err(AttemptError::CertaintyLevelRequired)?;
            }
            QuizKind::Traditional if input.certainty_level.is_some() => {
                return Err(AttemptError::CertaintyLevelNotAllowed)?;
            }
            _ => {}
        }

        self.ensure_attempt_is_answerable(&attempt)?;

        self.questions_service
            .ensure_question_belongs_to_attempt(&attempt.question_order, question_id)?;

        let questions_by_id = self
            .questions_service
            .get_questions_by_ids(&attempt.quiz_id, &attempt.question_order)
            .await?;

        let question = questions_by_id
            .get(&question_id)
            .ok_or_else(|| QuizError::QuestionNotFound(question_id.to_string()))?;

        self.questions_service
            .ensure_valid_answer_index(question, input.answer_index)?;

        let answer = AttemptAnswer::builder()
            .attempt_id(attempt.id)
            .question_id(question_id)
            .answer_index(input.answer_index)
            .maybe_certainty_level(input.certainty_level)
            .build();

        self.repository.upsert_attempt_answer(&answer).await?;

        Ok(())
    }

    pub async fn submit_attempt(
        &self,
        attempt_id: AttemptId,
        user_id: UserId,
    ) -> AppResult<AttemptSnapshotView> {
        let mut attempt = self.get_attempt_for_user(attempt_id, user_id).await?;

        self.ensure_attempt_is_answerable(&attempt)?;

        let Some(quiz) = self.quizzes.find_by_id(&attempt.quiz_id).await? else {
            return Err(QuizError::NotFound(attempt.quiz_id.to_string()))?;
        };

        let questions_by_id = self
            .questions_service
            .get_questions_by_ids(&attempt.quiz_id, &attempt.question_order)
            .await?;

        let answers = self.repository.list_attempt_answers(&attempt.id).await?;

        let grading = self.apply_grading(
            &attempt.question_order,
            &questions_by_id,
            &answers,
            quiz.kind,
            quiz.certainty_table,
        );

        attempt.score = Some(grading.score_points);
        attempt.grade = Some(grading.grade);
        attempt.submitted_at = Some(Utc::now());

        self.repository.save(&attempt).await?;

        Ok(AttemptSnapshotView::from(&attempt))
    }

    pub async fn view_results(
        &self,
        attempt_id: AttemptId,
        user_id: UserId,
    ) -> AppResult<AttemptResultView> {
        let mut attempt = self.get_attempt_for_user(attempt_id, user_id).await?;

        let submitted_at = attempt
            .submitted_at
            .ok_or(AttemptError::ResultsNotAvailable)?;

        let Some(quiz) = self.quizzes.find_by_id(&attempt.quiz_id).await? else {
            return Err(QuizError::NotFound(attempt.quiz_id.to_string()))?;
        };

        let score_points = attempt.score.ok_or(AttemptError::ResultsNotAvailable)?;
        let grade = attempt.grade.ok_or(AttemptError::ResultsNotAvailable)?;

        let questions_by_id = self
            .questions_service
            .get_questions_by_ids(&attempt.quiz_id, &attempt.question_order)
            .await?;

        let answers = self.repository.list_attempt_answers(&attempt.id).await?;

        let grading_details = self.apply_grading(
            &attempt.question_order,
            &questions_by_id,
            &answers,
            quiz.kind,
            quiz.certainty_table,
        );

        self.repository.mark_results_viewed(&attempt.id).await?;

        attempt.results_viewed_at = Some(Utc::now());

        Ok(AttemptResultView {
            attempt_id: attempt.id,
            quiz_id: attempt.quiz_id,
            status: AttemptStatus::Submitted,
            submitted_at,
            evaluated_at: submitted_at,
            score_points,
            score_points_max: grading_details.score_points_max,
            grade,
            results_viewed_at: attempt.results_viewed_at,
            questions: grading_details.questions,
        })
    }

    fn ensure_attempt_is_answerable(&self, attempt: &Attempt) -> AppResult<()> {
        let now = Utc::now();

        if attempt.expires_at + Duration::minutes(1) < now {
            return Err(AttemptError::Expired)?;
        }

        if attempt.submitted_at.is_some() {
            return Err(AttemptError::AlreadySubmitted)?;
        }

        Ok(())
    }

    fn apply_grading(
        &self,
        question_order: &[Uuid],
        questions_by_id: &HashMap<Uuid, Question>,
        answers: &[AttemptAnswer],
        quiz_kind: QuizKind,
        certainty_table: Option<crate::quizzes::CertaintyTable>,
    ) -> GradingOutput {
        self.grading.grade_attempt(
            question_order,
            questions_by_id,
            answers,
            quiz_kind,
            certainty_table,
        )
    }
}
