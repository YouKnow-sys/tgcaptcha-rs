CREATE TABLE IF NOT EXISTS join_storage (
    chat_id             INTEGER NOT NULL,
    message_id          INTEGER NOT NULL,
    user_id             INTEGER NOT NULL,
    is_passed           INTEGER NOT NULL CHECK (is_passed IN (0, 1)),
    question_lhs        INTEGER NOT NULL,
    question_operator   INTEGER NOT NULL CHECK (question_operator IN (0, 1, 2)),
    question_rhs        INTEGER NOT NULL,
    expires_at          INTEGER NOT NULL,
    PRIMARY KEY (chat_id, message_id)
) STRICT;

CREATE INDEX idx_join_storage_expires_at ON join_storage (expires_at);
