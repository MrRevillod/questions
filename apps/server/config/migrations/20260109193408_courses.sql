
CREATE TABLE courses (
	id UUID PRIMARY KEY,
	name TEXT NOT NULL,
	code TEXT NOT NULL,
	year SMALLINT NOT NULL,
	deleted_at TIMESTAMPTZ
);

CREATE TABLE course_members (
	id UUID PRIMARY KEY,
	course_id UUID NOT NULL REFERENCES courses(id),
	user_id UUID NOT NULL REFERENCES users(id),
	role user_role NOT NULL,
	UNIQUE (course_id, user_id)
);

CREATE INDEX idx_course_members_course_id ON course_members(course_id);
CREATE INDEX idx_course_members_user_id ON course_members(user_id);
CREATE INDEX idx_course_members_role ON course_members(role);
CREATE INDEX idx_course_members_course_user ON course_members(course_id, user_id);
CREATE INDEX idx_courses_deleted_at ON courses(deleted_at);
CREATE UNIQUE INDEX idx_courses_code_active_unique ON courses(code) WHERE deleted_at IS NULL;
