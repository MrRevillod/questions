CREATE OR REPLACE FUNCTION soft_delete_course_children()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NULL OR OLD.deleted_at IS NOT NULL THEN
        RETURN NEW;
    END IF;

    UPDATE quizzes
    SET deleted_at = NEW.deleted_at
    WHERE course_id = NEW.id
      AND deleted_at IS NULL;

    UPDATE question_banks
    SET deleted_at = NEW.deleted_at
    WHERE course_id = NEW.id
      AND deleted_at IS NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_soft_delete_course_children
AFTER UPDATE OF deleted_at ON courses
FOR EACH ROW
EXECUTE FUNCTION soft_delete_course_children();

CREATE OR REPLACE FUNCTION soft_delete_quiz_children()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NULL OR OLD.deleted_at IS NOT NULL THEN
        RETURN NEW;
    END IF;

    UPDATE attempts
    SET deleted_at = NEW.deleted_at
    WHERE quiz_id = NEW.id
      AND deleted_at IS NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_soft_delete_quiz_children
AFTER UPDATE OF deleted_at ON quizzes
FOR EACH ROW
EXECUTE FUNCTION soft_delete_quiz_children();

CREATE OR REPLACE FUNCTION soft_delete_course_on_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE courses
    SET deleted_at = NOW()
    WHERE id = OLD.id
      AND deleted_at IS NULL;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_soft_delete_course_on_delete
BEFORE DELETE ON courses
FOR EACH ROW
EXECUTE FUNCTION soft_delete_course_on_delete();

CREATE OR REPLACE FUNCTION soft_delete_quiz_on_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE quizzes
    SET deleted_at = NOW()
    WHERE id = OLD.id
      AND deleted_at IS NULL;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_soft_delete_quiz_on_delete
BEFORE DELETE ON quizzes
FOR EACH ROW
EXECUTE FUNCTION soft_delete_quiz_on_delete();

CREATE OR REPLACE FUNCTION soft_delete_question_bank_on_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE question_banks
    SET deleted_at = NOW()
    WHERE id = OLD.id
      AND deleted_at IS NULL;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_soft_delete_question_bank_on_delete
BEFORE DELETE ON question_banks
FOR EACH ROW
EXECUTE FUNCTION soft_delete_question_bank_on_delete();

CREATE OR REPLACE FUNCTION soft_delete_attempt_on_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE attempts
    SET deleted_at = NOW()
    WHERE id = OLD.id
      AND deleted_at IS NULL;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_soft_delete_attempt_on_delete
BEFORE DELETE ON attempts
FOR EACH ROW
EXECUTE FUNCTION soft_delete_attempt_on_delete();

CREATE OR REPLACE FUNCTION prevent_question_bank_snapshot_delete()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'question_bank_snapshots are immutable and cannot be deleted';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_question_bank_snapshot_delete
BEFORE DELETE ON question_bank_snapshots
FOR EACH ROW
EXECUTE FUNCTION prevent_question_bank_snapshot_delete();

CREATE OR REPLACE FUNCTION prevent_question_bank_snapshot_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS DISTINCT FROM OLD.deleted_at THEN
        RAISE EXCEPTION 'question_bank_snapshots cannot be soft-deleted';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_question_bank_snapshot_soft_delete
BEFORE UPDATE OF deleted_at ON question_bank_snapshots
FOR EACH ROW
EXECUTE FUNCTION prevent_question_bank_snapshot_soft_delete();
