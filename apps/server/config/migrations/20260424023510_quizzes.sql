CREATE TYPE quiz_kind AS ENUM ('traditional', 'certainty');
CREATE TYPE certainty_level AS ENUM ('low', 'medium', 'high');

CREATE TYPE certainty_score AS (
    correct SMALLINT,
    incorrect SMALLINT
);

CREATE TYPE certainty_table AS (
    low certainty_score,
    medium certainty_score,
    high certainty_score
);

CREATE TABLE quizzes (
	id UUID PRIMARY KEY,
	course_id UUID NOT NULL REFERENCES courses(id),
	title TEXT NOT NULL,
	kind quiz_kind NOT NULL,
	join_code TEXT NOT NULL UNIQUE,
	question_count SMALLINT NOT NULL,
	certainty_table certainty_table,
	attempt_duration_minutes SMALLINT NOT NULL,
	starts_at TIMESTAMPTZ NOT NULL,
	closed_at TIMESTAMPTZ,
	created_at TIMESTAMPTZ NOT NULL,
	deleted_at TIMESTAMPTZ
);

CREATE TABLE quiz_question_banks (
	quiz_id UUID NOT NULL REFERENCES quizzes(id),
	question_bank_id UUID NOT NULL REFERENCES question_banks(id),
	PRIMARY KEY (quiz_id, question_bank_id)
);

CREATE TABLE question_bank_snapshots (
	id UUID PRIMARY KEY,
	quiz_id UUID NOT NULL REFERENCES quizzes(id),
	questions question[] NOT NULL,
	deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_quizzes_kind ON quizzes(kind);
CREATE INDEX idx_quizzes_course_id ON quizzes(course_id);
CREATE INDEX idx_quizzes_join_code ON quizzes(join_code);
CREATE INDEX idx_quizzes_starts_at ON quizzes(starts_at);
CREATE INDEX idx_quizzes_closed_at ON quizzes(closed_at);
CREATE INDEX idx_quizzes_deleted_at ON quizzes(deleted_at);

CREATE INDEX idx_quiz_question_banks_quiz_id ON quiz_question_banks(quiz_id);
CREATE INDEX idx_quiz_question_banks_question_bank_id ON quiz_question_banks(question_bank_id);
CREATE INDEX idx_question_bank_snapshots_quiz_id ON question_bank_snapshots(quiz_id);
CREATE INDEX idx_question_bank_snapshots_deleted_at ON question_bank_snapshots(deleted_at);
