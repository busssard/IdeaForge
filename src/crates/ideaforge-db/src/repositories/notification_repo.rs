use sea_orm::prelude::Expr;
use sea_orm::*;
use uuid::Uuid;

use crate::entities::enums::NotificationKind;
use crate::entities::notification;

pub struct NotificationRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> NotificationRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new notification.
    pub async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        kind: NotificationKind,
        title: &str,
        message: &str,
        link_url: Option<&str>,
    ) -> Result<notification::Model, DbErr> {
        let model = notification::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            kind: Set(kind),
            title: Set(title.to_string()),
            message: Set(message.to_string()),
            link_url: Set(link_url.map(|s| s.to_string())),
            read_at: Set(None),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            related_user_id: Set(None),
            count: Set(1),
        };
        model.insert(self.db).await
    }

    /// List notifications for a user (paginated, optionally unread only).
    pub async fn list_for_user(
        &self,
        user_id: Uuid,
        unread_only: bool,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<notification::Model>, u64), DbErr> {
        let mut query =
            notification::Entity::find().filter(notification::Column::UserId.eq(user_id));

        if unread_only {
            query = query.filter(notification::Column::ReadAt.is_null());
        }

        query = query.order_by_desc(notification::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }

    /// Mark a single notification as read.
    pub async fn mark_read(
        &self,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<notification::Model>, DbErr> {
        let model = notification::Entity::find_by_id(notification_id)
            .filter(notification::Column::UserId.eq(user_id))
            .one(self.db)
            .await?;

        match model {
            Some(n) => {
                let mut active: notification::ActiveModel = n.into();
                active.read_at = Set(Some(chrono::Utc::now().fixed_offset()));
                let updated = active.update(self.db).await?;
                Ok(Some(updated))
            }
            None => Ok(None),
        }
    }

    /// Mark all notifications as read for a user.
    pub async fn mark_all_read(&self, user_id: Uuid) -> Result<u64, DbErr> {
        let result = notification::Entity::update_many()
            .col_expr(
                notification::Column::ReadAt,
                Expr::value(chrono::Utc::now().fixed_offset()),
            )
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::ReadAt.is_null())
            .exec(self.db)
            .await?;
        Ok(result.rows_affected)
    }

    /// Count unread notifications for a user.
    pub async fn count_unread(&self, user_id: Uuid) -> Result<u64, DbErr> {
        notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::ReadAt.is_null())
            .count(self.db)
            .await
    }
}
