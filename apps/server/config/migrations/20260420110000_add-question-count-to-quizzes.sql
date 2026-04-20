ALTER TABLE quizzes
ADD COLUMN question_count INTEGER;

UPDATE quizzes
SET question_count = CARDINALITY(questions)
WHERE question_count IS NULL;

ALTER TABLE quizzes
ALTER COLUMN question_count SET NOT NULL;

ALTER TABLE quizzes
ADD CONSTRAINT quizzes_question_count_positive CHECK (question_count > 0),
ADD CONSTRAINT quizzes_question_count_within_bank CHECK (question_count <= CARDINALITY(questions));
