use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
pub use models::JoinCheckData;
use sqlx::SqlitePool;
use teloxide::{
    prelude::Requester,
    types::{ChatId, MessageId},
    Bot,
};

mod models;

#[derive(Clone)]
pub(super) struct ChatStorage {
    pool: sqlx::SqlitePool,
    chat_id: ChatId,
}

impl ChatStorage {
    pub(super) fn new(pool: sqlx::SqlitePool, chat_id: ChatId) -> Self {
        Self { pool, chat_id }
    }

    pub async fn add(&self, data: JoinCheckData) -> sqlx::Result<bool> {
        let res = sqlx::query(
            r#"
            INSERT INTO join_storage
            (
                chat_id,
                message_id,
                user_id,
                is_passed,
                question_lhs,
                question_operator,
                question_rhs,
                expires_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(data.chat_id.0)
        .bind(data.message_id.0)
        .bind(data.user_id.0 as i64)
        .bind(data.is_passed as i8)
        .bind(data.question.lhs)
        .bind(data.question.operator.into_number())
        .bind(data.question.rhs)
        .bind(data.expires_at.timestamp_nanos_opt())
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn get(&self, message_id: MessageId) -> sqlx::Result<Option<JoinCheckData>> {
        sqlx::query_as(
            r#"
            SELECT
                chat_id,
                message_id,
                user_id,
                is_passed,
                question_lhs,
                question_operator,
                question_rhs,
                expires_at
            FROM join_storage
            WHERE chat_id = ? AND message_id = ?
            "#,
        )
        .bind(self.chat_id.0)
        .bind(message_id.0)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_and_delete(
        &self,
        message_id: MessageId,
    ) -> sqlx::Result<Option<JoinCheckData>> {
        sqlx::query_as(
            r#"
            DELETE FROM join_storage
            WHERE chat_id = ? AND message_id = ?
            RETURNING
                chat_id,
                message_id,
                user_id,
                is_passed,
                question_lhs,
                question_operator,
                question_rhs,
                expires_at
            "#,
        )
        .bind(self.chat_id.0)
        .bind(message_id.0)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn mark_as_passed(&self, message_id: MessageId) -> sqlx::Result<bool> {
        let res = sqlx::query(
            "UPDATE join_storage SET is_passed = TRUE WHERE chat_id = ? AND message_id = ?",
        )
        .bind(self.chat_id.0)
        .bind(message_id.0)
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }
}

// we have this task to remove checks that the bot missed for any reason
pub async fn delete_expired_task(bot: Bot, pool: sqlx::SqlitePool) -> ! {
    let time = Duration::minutes(5);
    loop {
        // we delete expired join checks after 5 minutes
        let expired = match get_and_delete_expired(&pool, Utc::now() - time).await {
            Ok(res) => res,
            Err(e) => {
                log::error!("failed to get expired check data from database: {e}");
                tokio::time::sleep(StdDuration::from_secs(5 * 60)).await;
                continue;
            }
        };

        for data in expired {
            if let Err(e) = bot.delete_message(data.chat_id, data.message_id).await {
                log::warn!("failed to delete expired join check message: {e}");
            }

            log::error!(
                "the state of user {} is unknown, manual intervention required",
                data.user_id
            );

            tokio::time::sleep(StdDuration::from_secs(5)).await;
        }

        tokio::time::sleep(StdDuration::from_secs(5 * 60)).await;
    }
}

async fn get_and_delete_expired(
    pool: &SqlitePool,
    from: DateTime<Utc>,
) -> sqlx::Result<Vec<JoinCheckData>> {
    sqlx::query_as(
        r#"
        DELETE FROM join_storage
        WHERE expires_at <= ?
        RETURNING
            chat_id,
            message_id,
            user_id,
            is_passed,
            question_lhs,
            question_operator,
            question_rhs,
            expires_at
        "#,
    )
    .bind(from.timestamp_nanos_opt())
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod test {
    use crate::join_check::{MathQuestion, Operator};

    use super::*;
    use sqlx::SqlitePool;
    use teloxide::types::UserId;

    #[tokio::test]
    async fn test_insert_and_get() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to connect to database");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("failed to migrate database");

        let storage = ChatStorage::new(pool, ChatId(1));

        let mut check_data = JoinCheckData {
            user_id: UserId(1),
            chat_id: ChatId(1),
            message_id: MessageId(1),
            question: MathQuestion {
                lhs: 1,
                operator: Operator::Add,
                rhs: 1,
            },
            is_passed: false,
            expires_at: Utc::now() + Duration::minutes(5),
        };

        storage.add(check_data.clone()).await.unwrap();

        assert!(storage.add(check_data.clone()).await.is_err());

        assert_eq!(
            storage.get(check_data.message_id).await.unwrap(),
            Some(check_data.clone())
        );

        assert!(storage.mark_as_passed(check_data.message_id).await.unwrap());

        check_data.is_passed = true;

        assert_eq!(
            storage.get_and_delete(check_data.message_id).await.unwrap(),
            Some(check_data.clone())
        );

        assert!(storage
            .get_and_delete(check_data.message_id)
            .await
            .unwrap()
            .is_none());

        assert!(storage.get(check_data.message_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_expired() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to connect to database");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("failed to migrate database");

        let storage = ChatStorage::new(pool.clone(), ChatId(1));

        let check_data = JoinCheckData {
            user_id: UserId(1),
            chat_id: ChatId(1),
            message_id: MessageId(1),
            question: MathQuestion {
                lhs: 1,
                operator: Operator::Add,
                rhs: 1,
            },
            is_passed: false,
            expires_at: Utc::now() - Duration::minutes(5),
        };

        storage.add(check_data.clone()).await.unwrap();

        assert_eq!(
            get_and_delete_expired(&pool, Utc::now() - Duration::minutes(5))
                .await
                .unwrap(),
            vec![check_data.clone()]
        );

        assert_eq!(
            storage.get_and_delete(check_data.message_id).await.unwrap(),
            None
        );

        assert_eq!(
            get_and_delete_expired(&pool, Utc::now() - Duration::minutes(5))
                .await
                .unwrap(),
            vec![]
        );
    }
}
