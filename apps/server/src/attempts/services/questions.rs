use rand::seq::SliceRandom;
use std::{collections::HashMap, sync::Arc};
use sword::prelude::*;
use uuid::Uuid;

use crate::{
    attempts::{AttemptError, AttemptRepository},
    banks::{Question, QuestionView},
    quizzes::{QuizError, QuizId},
    shared::AppResult,
    snapshots::SnapshotRepository,
};

#[injectable]
pub struct QuestionService {
    snapshots: Arc<SnapshotRepository>,
    repository: Arc<AttemptRepository>,
}

impl QuestionService {
    pub async fn get_questions_by_ids(
        &self,
        quiz_id: &QuizId,
        question_ids: &[Uuid],
    ) -> AppResult<HashMap<Uuid, Question>> {
        let questions = self.snapshots.list_questions_for_quiz(quiz_id).await?;

        let question_map = questions
            .into_iter()
            .map(|q| (q.id, q))
            .collect::<HashMap<Uuid, Question>>();

        for id in question_ids {
            if !question_map.contains_key(id) {
                return Err(QuizError::QuestionNotFound(id.to_string()))?;
            };
        }

        Ok(question_map)
    }

    pub async fn get_question_views_by_ids(
        &self,
        quiz_id: &QuizId,
        question_ids: &[Uuid],
    ) -> AppResult<HashMap<Uuid, QuestionView>> {
        let questions = self.get_questions_by_ids(quiz_id, question_ids).await?;

        let views = questions
            .into_iter()
            .map(|(k, v)| (k, QuestionView::from(v)))
            .collect::<HashMap<Uuid, QuestionView>>();

        Ok(views)
    }

    pub async fn get_ordered_question_views(
        &self,
        quiz_id: &QuizId,
        question_ids: &[Uuid],
    ) -> AppResult<Vec<QuestionView>> {
        let views_by_id = self
            .get_question_views_by_ids(quiz_id, question_ids)
            .await?;

        let mut ordered = Vec::with_capacity(question_ids.len());

        for question_id in question_ids {
            let Some(question) = views_by_id.get(question_id) else {
                return Err(QuizError::QuestionNotFound(question_id.to_string()))?;
            };

            ordered.push(question.clone());
        }

        Ok(ordered)
    }

    pub async fn get_question_ids_for_attempt(
        &self,
        quiz_id: &QuizId,
        question_number: usize,
    ) -> AppResult<Vec<Uuid>> {
        let questions = self.snapshots.list_questions_for_quiz(quiz_id).await?;

        if questions.len() < question_number {
            return Err(QuizError::InvalidQuestionCount)?;
        }

        let mut question_ids = questions.into_iter().map(|q| q.id).collect::<Vec<Uuid>>();

        question_ids.shuffle(&mut rand::rng());

        let selected_ids = question_ids
            .into_iter()
            .take(question_number)
            .collect::<Vec<Uuid>>();

        Ok(selected_ids)
    }

    pub fn ensure_question_belongs_to_attempt(
        &self,
        question_order: &[Uuid],
        question_id: Uuid,
    ) -> AppResult<()> {
        if !question_order.contains(&question_id) {
            return Err(AttemptError::QuestionNotInAttempt(question_id))?;
        }

        Ok(())
    }

    pub fn ensure_valid_answer_index(
        &self,
        question: &Question,
        answer_index: i16,
    ) -> AppResult<()> {
        let max = question.options.len() as i16;

        if answer_index < 0 || answer_index >= max {
            return Err(AttemptError::InvalidAnswerIndex)?;
        }

        Ok(())
    }
}
