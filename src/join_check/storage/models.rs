use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use teloxide::types::{ChatId, MessageId, UserId};

use crate::join_check::captcha::{MathQuestion, Operator};

pub struct JoinCheckData {
    pub user_id: UserId,
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub question: MathQuestion,
    pub is_passed: bool,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl FromRow<'_, SqliteRow> for JoinCheckData {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            user_id: UserId(row.try_get::<'_, i64, _>("user_id")? as u64),
            chat_id: ChatId(row.try_get("chat_id")?),
            message_id: MessageId(row.try_get("message_id")?),
            question: MathQuestion {
                lhs: row.try_get("question_lhs")?,
                operator: Operator::from_number(row.try_get("question_operator")?),
                rhs: row.try_get("question_rhs")?,
            },
            is_passed: row.try_get("is_passed")?,
            expires_at: chrono::DateTime::from_timestamp_nanos(row.try_get("expires_at")?),
        })
    }
}
