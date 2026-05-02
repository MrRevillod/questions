use std::collections::HashMap;

use sword::prelude::*;
use uuid::Uuid;

use crate::{
    attempts::{AttemptAnswer, AttemptQuestionResultView},
    banks::Question,
    quizzes::{CertaintyLevel, CertaintyTable, QuizKind},
};

pub struct GradingOutput {
    pub score_points: i16,
    pub score_points_max: i16,
    pub grade: f64,
    pub questions: Vec<AttemptQuestionResultView>,
}

#[injectable]
pub struct GradingService;

impl GradingService {
    pub fn grade_attempt(
        &self,
        question_order: &[Uuid],
        questions_by_id: &HashMap<Uuid, Question>,
        answers: &[AttemptAnswer],
        quiz_kind: QuizKind,
        certainty_table: Option<CertaintyTable>,
    ) -> GradingOutput {
        let answers_by_question = answers
            .iter()
            .map(|answer| (answer.question_id, answer))
            .collect::<HashMap<_, _>>();

        let mut score_points = 0i16;
        let score_points_max = question_order.len() as i16;
        let mut question_results = Vec::with_capacity(question_order.len());

        for question_id in question_order {
            let Some(question) = questions_by_id.get(question_id) else {
                continue;
            };

            let answer = answers_by_question.get(question_id).copied();
            let selected_index = answer.map(|a| a.answer_index);
            let certainty_level = answer.and_then(|a| a.certainty_level.clone());

            let is_correct = selected_index
                .map(|idx| idx == question.answer_index)
                .unwrap_or(false);

            let awarded_points = match quiz_kind {
                QuizKind::Traditional => {
                    if is_correct {
                        1
                    } else {
                        0
                    }
                }
                QuizKind::Certainty => self.awarded_points_for_certainty(
                    is_correct,
                    certainty_level.clone(),
                    &certainty_table,
                ),
            };

            score_points += awarded_points;

            question_results.push(AttemptQuestionResultView {
                question_id: question.id,
                question: question.prompt.clone(),
                options: question.options.clone(),
                images: question.images.clone(),
                answer_index: selected_index,
                correct_answer_index: question.answer_index,
                certainty_level,
                is_correct,
                awarded_points,
            });
        }

        let grade = if score_points_max > 0 {
            score_points as f64 / score_points_max as f64
        } else {
            0.0
        };

        GradingOutput {
            score_points,
            score_points_max,
            grade,
            questions: question_results,
        }
    }

    fn awarded_points_for_certainty(
        &self,
        is_correct: bool,
        certainty_level: Option<CertaintyLevel>,
        table: &Option<CertaintyTable>,
    ) -> i16 {
        let Some(table) = table else {
            return if is_correct { 1 } else { 0 };
        };

        let Some(level) = certainty_level else {
            return 0;
        };

        match (level, is_correct) {
            (CertaintyLevel::Low, true) => table.low.correct,
            (CertaintyLevel::Low, false) => table.low.incorrect,
            (CertaintyLevel::Medium, true) => table.medium.correct,
            (CertaintyLevel::Medium, false) => table.medium.incorrect,
            (CertaintyLevel::High, true) => table.high.correct,
            (CertaintyLevel::High, false) => table.high.incorrect,
        }
    }
}
