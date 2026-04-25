CREATE TYPE question AS (
    id UUID,
    prompt TEXT,
    options TEXT[],
    answer_index SMALLINT,
    images TEXT[]
);

CREATE TABLE question_banks (
	id UUID PRIMARY KEY,
	course_id UUID NOT NULL REFERENCES courses(id),
	name TEXT NOT NULL,
	questions question[] NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_question_banks_course_id ON question_banks(course_id);
CREATE INDEX idx_question_banks_created_at ON question_banks(created_at);
CREATE INDEX idx_question_banks_deleted_at ON question_banks(deleted_at);
