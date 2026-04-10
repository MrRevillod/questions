CREATE TYPE quiz_kind AS ENUM ('Traditional', 'Certainly');

CREATE TYPE certainly_level AS (
    correct SMALLINT,
    incorrect SMALLINT
);

CREATE TYPE certainly_table AS (
    low certainly_level,
    medium certainly_level,
    high certainly_level
);

CREATE TYPE question AS (
    id UUID,
    question TEXT,
    options TEXT[],
    answer SMALLINT,
    images TEXT[]
);

CREATE TYPE attempt_certainty_level AS ENUM ('low', 'medium', 'high');

CREATE TABLE quizzes (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    kind quiz_kind NOT NULL,
    join_code TEXT UNIQUE NOT NULL,
    questions question[] NOT NULL,
    certainly_table certainly_table NULL,
    start_time TIMESTAMPTZ NOT NULL,
    attempt_duration_minutes INTEGER NOT NULL,
    closed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (attempt_duration_minutes > 0)
);

CREATE TABLE quiz_collaborators (
    quiz_id UUID NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (quiz_id, user_id)
);

CREATE TABLE quiz_attempts (
    id UUID PRIMARY KEY,
    quiz_id UUID NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    submitted_at TIMESTAMPTZ NULL,
    question_order UUID[] NOT NULL,
    score_points DOUBLE PRECISION NULL,
    score_points_max DOUBLE PRECISION NULL,
    grade DOUBLE PRECISION NULL,
    evaluated_at TIMESTAMPTZ NULL,
    evaluated_by UUID NULL REFERENCES users(id),
    results_released_at TIMESTAMPTZ NULL,
    results_viewed_at TIMESTAMPTZ NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CHECK (expires_at > started_at),
    CHECK (submitted_at IS NULL OR submitted_at >= started_at)
);

CREATE TABLE quiz_answers (
    attempt_id UUID NOT NULL REFERENCES quiz_attempts(id) ON DELETE CASCADE,
    question_id UUID NOT NULL,
    answer_index SMALLINT NOT NULL,
    certainty_level attempt_certainty_level NULL,
    PRIMARY KEY (attempt_id, question_id)
);

CREATE INDEX idx_quizzes_owner_id ON quizzes(owner_id);
CREATE INDEX idx_quizzes_created_at ON quizzes(created_at);
CREATE INDEX idx_quizzes_join_code ON quizzes(join_code);
CREATE INDEX idx_quizzes_closed_at ON quizzes(closed_at);

CREATE INDEX idx_quiz_collaborators_user_id ON quiz_collaborators(user_id);

CREATE INDEX idx_quiz_attempts_student_id ON quiz_attempts(student_id);
CREATE INDEX idx_quiz_attempts_quiz_id ON quiz_attempts(quiz_id);
CREATE INDEX idx_quiz_attempts_results_released_at ON quiz_attempts(results_released_at);
CREATE UNIQUE INDEX idx_quiz_attempts_one_per_student
    ON quiz_attempts(quiz_id, student_id);

CREATE INDEX idx_quiz_answers_question_id ON quiz_answers(question_id);
