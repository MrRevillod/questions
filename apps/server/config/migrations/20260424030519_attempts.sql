CREATE TABLE attempts (
	id UUID PRIMARY KEY,
	student_id UUID NOT NULL REFERENCES users(id),
	quiz_id UUID NOT NULL REFERENCES quizzes(id),
	score SMALLINT,
	grade DOUBLE PRECISION,
	question_order UUID[] NOT NULL,
	started_at TIMESTAMPTZ NOT NULL,
	expires_at TIMESTAMPTZ NOT NULL,
	submitted_at TIMESTAMPTZ,
	results_viewed_at TIMESTAMPTZ,
	deleted_at TIMESTAMPTZ
);

CREATE TABLE attempt_answers (
	attempt_id UUID NOT NULL REFERENCES attempts(id),
	question_id UUID NOT NULL,
	answer_index SMALLINT NOT NULL,
	certainty_level certainty_level,
	PRIMARY KEY (attempt_id, question_id)
);

CREATE INDEX idx_attempt_answers_attempt_id ON attempt_answers(attempt_id);
CREATE INDEX idx_attempt_answers_question_id ON attempt_answers(question_id);
CREATE INDEX idx_attempts_deleted_at ON attempts(deleted_at);
