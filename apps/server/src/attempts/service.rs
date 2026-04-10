use crate::attempts::*;
use crate::quizzes::*;
use crate::shared::AppError;
use crate::shared::AppResult;
use crate::users::User;

use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct AttemptService {
    attempts: AttemptRepository,
    policy: AttemptPolicy,
    quizzes: QuizRepository,
    quiz_policy: QuizPolicy,
}

impl AttemptService {
    pub async fn start_attempt(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
    ) -> AppResult<AttemptSnapshotView> {
        self.policy.can_start_attempt(current_user)?;

        let quiz = self.require_quiz(quiz_id).await?;

        if quiz.closed_at.is_some() {
            return Err(QuizError::Closed.into());
        }

        if Utc::now() < quiz.start_time {
            return Err(AttemptError::NotStarted)?;
        }

        if let Some(attempt) = self
            .attempts
            .find_by_quiz_and_student(quiz_id, &current_user.id)
            .await?
        {
            if attempt.submitted_at.is_some() {
                return Err(AttemptError::AlreadySubmitted)?;
            }

            let answers = self.attempts.list_answers(&attempt.id).await?;

            return Ok(self.build_snapshot(&quiz, attempt, answers));
        }

        let question_order = shuffled_question_order(&quiz);

        let started_at = Utc::now();
        let expires_at = started_at + Duration::minutes(i64::from(quiz.attempt_duration_minutes));

        let attempt = AttemptEntity {
            id: Uuid::new_v4(),
            quiz_id: *quiz_id,
            student_id: current_user.id,
            started_at,
            expires_at,
            submitted_at: None,
            question_order,
            score_points: None,
            score_points_max: None,
            grade: None,
            evaluated_at: None,
            evaluated_by: None,
            results_released_at: None,
            results_viewed_at: None,
            updated_at: started_at,
        };

        let attempt = self.attempts.create(attempt).await?;

        Ok(self.build_snapshot(&quiz, attempt, Vec::new()))
    }

    pub async fn get_active_attempt_for_quiz(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
    ) -> AppResult<AttemptSnapshotView> {
        self.policy.can_start_attempt(current_user)?;

        let Some(attempt) = self
            .attempts
            .find_active_for_quiz(quiz_id, &current_user.id)
            .await?
        else {
            return Err(AttemptError::ActiveAttemptNotFound(quiz_id.to_string()).into());
        };

        let quiz = self.require_quiz(&attempt.quiz_id).await?;
        let answers = self.attempts.list_answers(&attempt.id).await?;

        Ok(self.build_snapshot(&quiz, attempt, answers))
    }

    pub async fn save_answer(
        &self,
        current_user: &User,
        command: SaveAnswerCommand,
    ) -> AppResult<AttemptAnswerView> {
        let attempt = self
            .require_attempt_owner(current_user, &command.attempt_id)
            .await?;

        let quiz = self.require_quiz(&attempt.quiz_id).await?;

        self.policy.can_save_answer(current_user, &attempt)?;

        if !attempt.question_order.contains(&command.question_id) {
            return Err(AttemptError::QuestionNotInAttempt.into());
        }

        let Some(question) = quiz
            .questions
            .iter()
            .find(|question| question.id == command.question_id)
        else {
            return Err(AttemptError::QuestionNotInAttempt.into());
        };

        let is_invalid_answer = match usize::try_from(command.answer_index) {
            Ok(answer_index) => answer_index >= question.options.len(),
            Err(_) => true,
        };

        if is_invalid_answer {
            return Err(AttemptError::InvalidAnswerIndex.into());
        }

        let answer = AttemptAnswerEntity {
            attempt_id: attempt.id,
            question_id: command.question_id,
            answer_index: command.answer_index,
            certainty_level: command.certainty_level,
        };

        let answer = self.attempts.upsert_answer(answer).await?;

        Ok(AttemptAnswerView::from(answer))
    }

    pub async fn submit_attempt(
        &self,
        current_user: &User,
        attempt_id: &Uuid,
    ) -> AppResult<AttemptSnapshotView> {
        let attempt = self.require_attempt_owner(current_user, attempt_id).await?;
        self.policy.can_submit_attempt(current_user, &attempt)?;

        let attempt = self.attempts.submit(attempt_id).await.map_err(|error| {
            if let AppError::Database(sqlx::Error::RowNotFound) = error {
                AppError::Attempt(AttemptError::AlreadySubmitted)
            } else {
                error
            }
        })?;
        let quiz = self.require_quiz(&attempt.quiz_id).await?;
        let answers = self.attempts.list_answers(attempt_id).await?;

        let evaluation = self.evaluate_attempt(&quiz, &attempt, &answers);

        self.attempts
            .evaluate(
                attempt_id,
                evaluation.score_points,
                evaluation.score_points_max,
                evaluation.grade,
                Some(current_user.id),
            )
            .await?;

        Ok(self.build_snapshot(&quiz, attempt, answers))
    }

    pub async fn finalize_and_publish_quiz(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
    ) -> AppResult<FinalizeAndPublishSummaryView> {
        let quiz = self
            .quiz_policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;

        let in_progress_attempts = self.attempts.list_in_progress_for_quiz(quiz_id).await?;
        let mut finalized_attempts = 0usize;

        for attempt in in_progress_attempts {
            let submitted_attempt = self.attempts.submit(&attempt.id).await.map_err(|error| {
                if let AppError::Database(sqlx::Error::RowNotFound) = error {
                    AppError::Attempt(AttemptError::AlreadySubmitted)
                } else {
                    error
                }
            })?;

            let answers = self.attempts.list_answers(&attempt.id).await?;
            let evaluation = self.evaluate_attempt(&quiz, &submitted_attempt, &answers);

            self.attempts
                .evaluate(
                    &submitted_attempt.id,
                    evaluation.score_points,
                    evaluation.score_points_max,
                    evaluation.grade,
                    Some(current_user.id),
                )
                .await?;

            finalized_attempts += 1;
        }

        let submitted_attempts = self.attempts.list_submitted_for_quiz(quiz_id).await?;
        for attempt in submitted_attempts {
            if attempt.grade.is_some()
                && attempt.score_points.is_some()
                && attempt.score_points_max.is_some()
                && attempt.evaluated_at.is_some()
            {
                continue;
            }

            let answers = self.attempts.list_answers(&attempt.id).await?;
            let evaluation = self.evaluate_attempt(&quiz, &attempt, &answers);

            self.attempts
                .evaluate(
                    &attempt.id,
                    evaluation.score_points,
                    evaluation.score_points_max,
                    evaluation.grade,
                    Some(current_user.id),
                )
                .await?;
        }

        let published_attempts = self.attempts.release_results_for_quiz(quiz_id).await? as usize;
        self.quizzes.close_quiz(quiz_id).await?;

        Ok(FinalizeAndPublishSummaryView {
            quiz_id: *quiz_id,
            finalized_attempts,
            published_attempts,
        })
    }

    pub async fn list_managed_quiz_attempts(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
    ) -> AppResult<Vec<ManagedAttemptSummaryView>> {
        self.quiz_policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;

        self.attempts.list_for_quiz(quiz_id).await
    }

    pub async fn get_result_for_student(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
    ) -> AppResult<AttemptResultView> {
        self.policy.can_start_attempt(current_user)?;

        let Some(attempt) = self
            .attempts
            .find_by_quiz_and_student(quiz_id, &current_user.id)
            .await?
        else {
            return Err(AttemptError::NotFound(quiz_id.to_string()).into());
        };

        if attempt.submitted_at.is_none() {
            return Err(AttemptError::NotSubmitted.into());
        }

        let quiz = self.require_quiz(quiz_id).await?;
        if attempt.results_released_at.is_none() {
            return Err(AttemptError::ResultNotAvailable.into());
        }

        if attempt.results_viewed_at.is_some() {
            return Err(AttemptError::ResultAlreadyViewed.into());
        }

        let Some(consumed_attempt) = self.attempts.mark_results_viewed_once(&attempt.id).await?
        else {
            let Some(recheck_attempt) = self.attempts.find_by_id(&attempt.id).await? else {
                return Err(AttemptError::NotFound(attempt.id.to_string()).into());
            };

            if recheck_attempt.results_released_at.is_none() {
                return Err(AttemptError::ResultNotAvailable.into());
            }

            return Err(AttemptError::ResultAlreadyViewed.into());
        };

        let answers = self.attempts.list_answers(&consumed_attempt.id).await?;
        let evaluation = self.evaluate_attempt(&quiz, &consumed_attempt, &answers);

        let evaluated_attempt = if consumed_attempt.grade.is_none()
            || consumed_attempt.score_points.is_none()
            || consumed_attempt.score_points_max.is_none()
            || consumed_attempt.evaluated_at.is_none()
        {
            self.attempts
                .evaluate(
                    &consumed_attempt.id,
                    evaluation.score_points,
                    evaluation.score_points_max,
                    evaluation.grade,
                    consumed_attempt.evaluated_by,
                )
                .await?
        } else {
            consumed_attempt
        };

        Ok(self.build_result_view(&quiz, &evaluated_attempt, &evaluation))
    }

    pub async fn get_result_for_attempt_owner(
        &self,
        current_user: &User,
        attempt_id: &Uuid,
    ) -> AppResult<AttemptResultView> {
        let attempt = self.require_attempt_owner(current_user, attempt_id).await?;

        if attempt.submitted_at.is_none() {
            return Err(AttemptError::NotSubmitted.into());
        }

        if attempt.results_released_at.is_none() {
            return Err(AttemptError::ResultNotAvailable.into());
        }

        if attempt.results_viewed_at.is_some() {
            return Err(AttemptError::ResultAlreadyViewed.into());
        }

        let Some(consumed_attempt) = self.attempts.mark_results_viewed_once(&attempt.id).await?
        else {
            let Some(recheck_attempt) = self.attempts.find_by_id(&attempt.id).await? else {
                return Err(AttemptError::NotFound(attempt.id.to_string()).into());
            };

            if recheck_attempt.results_released_at.is_none() {
                return Err(AttemptError::ResultNotAvailable.into());
            }

            return Err(AttemptError::ResultAlreadyViewed.into());
        };

        let quiz = self.require_quiz(&consumed_attempt.quiz_id).await?;
        let answers = self.attempts.list_answers(&consumed_attempt.id).await?;
        let evaluation = self.evaluate_attempt(&quiz, &consumed_attempt, &answers);

        let evaluated_attempt = if consumed_attempt.grade.is_none()
            || consumed_attempt.score_points.is_none()
            || consumed_attempt.score_points_max.is_none()
            || consumed_attempt.evaluated_at.is_none()
        {
            self.attempts
                .evaluate(
                    &consumed_attempt.id,
                    evaluation.score_points,
                    evaluation.score_points_max,
                    evaluation.grade,
                    consumed_attempt.evaluated_by,
                )
                .await?
        } else {
            consumed_attempt
        };

        Ok(self.build_result_view(&quiz, &evaluated_attempt, &evaluation))
    }

    async fn require_quiz(&self, quiz_id: &Uuid) -> AppResult<QuizEntity> {
        let Some(quiz) = self.quizzes.find_by_id(quiz_id).await? else {
            return Err(AttemptError::NotFound(quiz_id.to_string()).into());
        };

        Ok(quiz)
    }

    async fn require_attempt_owner(
        &self,
        current_user: &User,
        attempt_id: &Uuid,
    ) -> AppResult<AttemptEntity> {
        let Some(attempt) = self.attempts.find_by_id(attempt_id).await? else {
            return Err(AttemptError::NotFound(attempt_id.to_string()).into());
        };

        self.policy.can_access_attempt(current_user, &attempt)?;

        Ok(attempt)
    }

    fn build_snapshot(
        &self,
        quiz: &QuizEntity,
        attempt: AttemptEntity,
        answers: Vec<AttemptAnswerEntity>,
    ) -> AttemptSnapshotView {
        let mut ordered_questions = Vec::with_capacity(attempt.question_order.len());

        for question_id in &attempt.question_order {
            if let Some(question) = quiz
                .questions
                .iter()
                .find(|question| question.id == *question_id)
            {
                ordered_questions.push(QuizQuestionView::from(question.clone()));
            }
        }

        let quiz_view = QuizParticipantView {
            id: quiz.id,
            title: quiz.title.clone(),
            kind: quiz.kind.clone(),
            questions: ordered_questions,
            certainly_table: quiz.certainly_table.clone(),
            start_time: quiz.start_time,
            attempt_duration_minutes: quiz.attempt_duration_minutes,
            closed_at: quiz.closed_at,
        };

        AttemptSnapshotView {
            attempt_id: attempt.id,
            quiz_id: attempt.quiz_id,
            started_at: attempt.started_at,
            expires_at: attempt.expires_at,
            status: if attempt.submitted_at.is_some() {
                AttemptStatus::Submitted
            } else {
                AttemptStatus::InProgress
            },
            quiz: quiz_view,
            answers: answers.into_iter().map(AttemptAnswerView::from).collect(),
        }
    }

    fn evaluate_attempt(
        &self,
        quiz: &QuizEntity,
        attempt: &AttemptEntity,
        answers: &[AttemptAnswerEntity],
    ) -> AttemptEvaluation {
        let answer_map = answers
            .iter()
            .map(|answer| (answer.question_id, answer))
            .collect::<HashMap<_, _>>();

        let certainty_max_correct = quiz
            .certainly_table
            .as_ref()
            .map(|table| {
                i16::max(
                    table.low.correct,
                    i16::max(table.medium.correct, table.high.correct),
                ) as f64
            })
            .unwrap_or(1.0);

        let max_points_per_question = if certainty_max_correct > 0.0 {
            certainty_max_correct
        } else {
            1.0
        };

        let uses_certainty_table = quiz.certainly_table.is_some();
        let score_points_max = if uses_certainty_table {
            (attempt.question_order.len() as f64) * max_points_per_question
        } else {
            attempt.question_order.len() as f64
        };

        let mut question_results = Vec::with_capacity(attempt.question_order.len());
        let mut score_points = 0.0;

        for question_id in &attempt.question_order {
            let Some(question) = quiz
                .questions
                .iter()
                .find(|question| question.id == *question_id)
            else {
                continue;
            };

            let answer = answer_map.get(question_id).copied();
            let answer_index = answer.map(|value| value.answer_index);
            let certainty_level = answer.and_then(|value| value.certainty_level.clone());
            let is_correct = answer
                .map(|value| value.answer_index == question.answer)
                .unwrap_or(false);

            let awarded_points = self.compute_awarded_points(
                uses_certainty_table,
                quiz.certainly_table.as_ref(),
                is_correct,
                certainty_level.as_ref(),
            );

            score_points += awarded_points;

            question_results.push(AttemptQuestionResultView {
                question_id: question.id,
                question: question.question.clone(),
                options: question.options.clone(),
                images: question.images.clone(),
                answer_index,
                correct_answer_index: question.answer,
                certainty_level,
                is_correct,
                awarded_points,
            });
        }

        let grade = if score_points <= 0.0 || score_points_max <= 0.0 {
            1.0
        } else {
            round_two_decimals((score_points / score_points_max) * 6.0 + 1.0)
        };

        AttemptEvaluation {
            score_points,
            score_points_max,
            grade,
            questions: question_results,
        }
    }

    fn compute_awarded_points(
        &self,
        uses_certainty_table: bool,
        certainty_table: Option<&CertainlyTable>,
        is_correct: bool,
        certainty_level: Option<&AttemptCertaintyLevel>,
    ) -> f64 {
        if !uses_certainty_table {
            return if is_correct { 1.0 } else { 0.0 };
        }

        let Some(table) = certainty_table else {
            return if is_correct { 1.0 } else { 0.0 };
        };

        let Some(level) = certainty_level else {
            return 0.0;
        };

        let value = match level {
            AttemptCertaintyLevel::Low => &table.low,
            AttemptCertaintyLevel::Medium => &table.medium,
            AttemptCertaintyLevel::High => &table.high,
        };

        if is_correct {
            value.correct as f64
        } else {
            value.incorrect as f64
        }
    }

    fn build_result_view(
        &self,
        quiz: &QuizEntity,
        attempt: &AttemptEntity,
        evaluation: &AttemptEvaluation,
    ) -> AttemptResultView {
        let submitted_at = attempt.submitted_at.unwrap_or(attempt.updated_at);
        let evaluated_at = attempt.evaluated_at.unwrap_or(attempt.updated_at);
        let results_released_at = attempt.results_released_at.unwrap_or(evaluated_at);

        AttemptResultView {
            attempt_id: attempt.id,
            quiz_id: quiz.id,
            status: if attempt.submitted_at.is_some() {
                AttemptStatus::Submitted
            } else {
                AttemptStatus::InProgress
            },
            submitted_at,
            evaluated_at,
            score_points: attempt.score_points.unwrap_or(evaluation.score_points),
            score_points_max: attempt
                .score_points_max
                .unwrap_or(evaluation.score_points_max),
            grade: attempt.grade.unwrap_or(evaluation.grade),
            results_released_at,
            results_viewed_at: attempt.results_viewed_at,
            questions: evaluation.questions.clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct AttemptEvaluation {
    score_points: f64,
    score_points_max: f64,
    grade: f64,
    questions: Vec<AttemptQuestionResultView>,
}

fn round_two_decimals(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn shuffled_question_order(quiz: &QuizEntity) -> Vec<Uuid> {
    let mut question_order = quiz
        .questions
        .iter()
        .map(|question| question.id)
        .collect::<Vec<_>>();
    question_order.shuffle(&mut rand::rng());
    question_order
}
