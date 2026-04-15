//! MLS (RFC 9420) delivery service — ciphertext fan-out only.
//!
//! The backend sees ONLY ciphertext. No endpoint here should ever deserialize
//! plaintext message content; if we add one, we've broken the core claim of
//! the messaging feature.
//!
//! See `docs/architecture/simplex_messaging_spike.md` §§13–15 for the design.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::entities::{
    enums::NotificationKind, mls_group, mls_group_member, mls_keypackage, mls_message, mls_welcome,
    notification, user,
};

// ---- Routes ----------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/keypackages",
            post(publish_keypackages)
                .get(list_my_keypackages)
                .delete(purge_my_keypackages),
        )
        .route("/keypackages/:user_id/consume", post(consume_keypackage))
        .route("/groups", post(create_group).get(list_my_groups))
        .route("/groups/:id/members", post(add_members))
        .route(
            "/groups/:id/messages",
            post(post_message).get(list_messages),
        )
        .route("/groups/:id", delete(leave_group))
        .route("/welcomes", get(list_welcomes))
        .route("/welcomes/:id", delete(ack_welcome))
}

// ---- DTOs ------------------------------------------------------------------

/// Bytes are shipped as base64 strings over the JSON wire. The server never
/// interprets them — they're opaque to us.
fn encode_bytes(b: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(b)
}

fn decode_bytes(s: &str) -> Result<Vec<u8>, (StatusCode, Json<serde_json::Value>)> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.decode(s).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": { "code": "BAD_BASE64", "message": "Invalid base64 encoding" } })),
        )
    })
}

#[derive(Debug, Deserialize)]
pub struct PublishKeyPackagesRequest {
    /// Base64-encoded serialized `KeyPackage`s from the client.
    pub key_packages: Vec<String>,
    /// How many days until these expire. Clamped to [1, 90].
    #[serde(default = "default_ttl_days")]
    pub ttl_days: i64,
}

fn default_ttl_days() -> i64 {
    30
}

#[derive(Debug, Serialize)]
pub struct KeyPackageSummary {
    pub id: Uuid,
    pub consumed: bool,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct KeyPackageBundle {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_package_b64: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    /// Base64-encoded MLS `GroupID` chosen by the client.
    pub mls_group_id_b64: String,
    pub name: Option<String>,
    /// Users to add on creation. Their KeyPackages must have been
    /// independently consumed by the creator — the Welcomes below carry the
    /// cryptographic proof.
    pub initial_members: Vec<Uuid>,
    /// One Welcome per initial member (order-matched).
    pub welcomes_b64: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupSummary {
    pub id: Uuid,
    pub mls_group_id_b64: String,
    pub name: Option<String>,
    pub created_by: Uuid,
    pub created_at: String,
    pub members: Vec<MemberSummary>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MemberSummary {
    pub user_id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMembersRequest {
    pub new_members: Vec<Uuid>,
    pub welcomes_b64: Vec<String>,
    /// The Commit that adds the members, which existing members need to apply.
    pub commit_b64: String,
}

#[derive(Debug, Deserialize)]
pub struct PostMessageRequest {
    /// Base64-encoded MLS `MlsMessageOut` — can be an Application message or
    /// a Commit. The server does not distinguish.
    pub ciphertext_b64: String,
}

#[derive(Debug, Serialize)]
pub struct MessageEnvelope {
    pub id: i64,
    pub group_id: Uuid,
    pub sender_user_id: Uuid,
    pub ciphertext_b64: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct WelcomeEnvelope {
    pub id: Uuid,
    pub ciphertext_b64: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    /// Return messages with id > `since` (strict — monotonic sequence).
    #[serde(default)]
    pub since: i64,
    /// Max messages to return per call. Clamped to [1, 500].
    pub limit: Option<u64>,
}

fn err(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({
            "error": { "code": code, "message": message }
        })),
    )
}

// ---- KeyPackage endpoints --------------------------------------------------

async fn publish_keypackages(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PublishKeyPackagesRequest>,
) -> impl IntoResponse {
    if body.key_packages.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "EMPTY",
            "At least one KeyPackage required",
        )
        .into_response();
    }
    if body.key_packages.len() > 50 {
        return err(
            StatusCode::BAD_REQUEST,
            "TOO_MANY",
            "At most 50 KeyPackages per request",
        )
        .into_response();
    }

    let ttl_days = body.ttl_days.clamp(1, 90);
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::days(ttl_days);

    let db = state.db.connection();
    let mut inserted: Vec<Uuid> = Vec::with_capacity(body.key_packages.len());

    for kp_b64 in &body.key_packages {
        let bytes = match decode_bytes(kp_b64) {
            Ok(b) => b,
            Err(e) => return e.into_response(),
        };
        // Cap per-KeyPackage size to prevent abuse. KeyPackages are normally
        // a few hundred bytes; 32 KiB is generous.
        if bytes.is_empty() || bytes.len() > 32 * 1024 {
            return err(
                StatusCode::BAD_REQUEST,
                "BAD_KEYPACKAGE",
                "KeyPackage size out of range (1..=32KiB)",
            )
            .into_response();
        }

        let id = Uuid::new_v4();
        let model = mls_keypackage::ActiveModel {
            id: Set(id),
            user_id: Set(auth.user_id),
            key_package: Set(bytes),
            consumed_at: Set(None),
            created_at: Set(now.fixed_offset()),
            expires_at: Set(expires.fixed_offset()),
        };
        if let Err(e) = model.insert(db).await {
            tracing::error!("Failed to insert KeyPackage: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to store KeyPackage",
            )
            .into_response();
        }
        inserted.push(id);
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "ids": inserted })),
    )
        .into_response()
}

/// Delete every unconsumed KeyPackage belonging to the authenticated user.
/// Used for key rotation (client generates fresh ones and purges the old
/// batch) and for test-state hygiene. Consumed KeyPackages are kept for audit.
async fn purge_my_keypackages(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    let db = state.db.connection();
    match mls_keypackage::Entity::delete_many()
        .filter(mls_keypackage::Column::UserId.eq(auth.user_id))
        .filter(mls_keypackage::Column::ConsumedAt.is_null())
        .exec(db)
        .await
    {
        Ok(res) => (
            StatusCode::OK,
            Json(serde_json::json!({ "deleted": res.rows_affected })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to purge KeyPackages: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Purge failed",
            )
            .into_response()
        }
    }
}

async fn list_my_keypackages(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    let db = state.db.connection();
    match mls_keypackage::Entity::find()
        .filter(mls_keypackage::Column::UserId.eq(auth.user_id))
        .order_by_desc(mls_keypackage::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(items) => {
            let summaries: Vec<KeyPackageSummary> = items
                .into_iter()
                .map(|kp| KeyPackageSummary {
                    id: kp.id,
                    consumed: kp.consumed_at.is_some(),
                    created_at: kp.created_at.to_rfc3339(),
                    expires_at: kp.expires_at.to_rfc3339(),
                })
                .collect();
            Json(serde_json::json!({ "data": summaries })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list KeyPackages: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to list KeyPackages",
            )
            .into_response()
        }
    }
}

/// Consume one unconsumed KeyPackage for `user_id` atomically. Any user can
/// consume another user's KeyPackage — that's how MLS invitations bootstrap.
async fn consume_keypackage(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let db = state.db.connection();
    let txn = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin txn: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Transaction failed",
            )
            .into_response();
        }
    };

    let now = chrono::Utc::now();
    let candidate = match mls_keypackage::Entity::find()
        .filter(mls_keypackage::Column::UserId.eq(user_id))
        .filter(mls_keypackage::Column::ConsumedAt.is_null())
        .filter(mls_keypackage::Column::ExpiresAt.gt(now.fixed_offset()))
        .order_by_asc(mls_keypackage::Column::CreatedAt)
        .one(&txn)
        .await
    {
        Ok(Some(kp)) => kp,
        Ok(None) => {
            let _ = txn.rollback().await;
            return err(
                StatusCode::NOT_FOUND,
                "NO_KEYPACKAGES",
                "No available KeyPackages for that user",
            )
            .into_response();
        }
        Err(e) => {
            tracing::error!("KeyPackage query failed: {e}");
            let _ = txn.rollback().await;
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Query failed",
            )
            .into_response();
        }
    };

    let id = candidate.id;
    let kp_bytes = candidate.key_package.clone();
    let mut active: mls_keypackage::ActiveModel = candidate.into();
    active.consumed_at = Set(Some(now.fixed_offset()));
    if let Err(e) = active.update(&txn).await {
        tracing::error!("Failed to mark KeyPackage consumed: {e}");
        let _ = txn.rollback().await;
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Consume failed",
        )
        .into_response();
    }

    if let Err(e) = txn.commit().await {
        tracing::error!("Commit failed: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Commit failed",
        )
        .into_response();
    }

    Json(KeyPackageBundle {
        id,
        user_id,
        key_package_b64: encode_bytes(&kp_bytes),
    })
    .into_response()
}

// ---- Group endpoints -------------------------------------------------------

async fn create_group(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    if body.initial_members.len() != body.welcomes_b64.len() {
        return err(
            StatusCode::BAD_REQUEST,
            "LENGTH_MISMATCH",
            "initial_members and welcomes_b64 must have the same length",
        )
        .into_response();
    }
    if body.initial_members.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "EMPTY_GROUP",
            "A group needs at least one other member",
        )
        .into_response();
    }

    let mls_gid = match decode_bytes(&body.mls_group_id_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };

    let db = state.db.connection();
    let txn = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("txn begin failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Transaction failed",
            )
            .into_response();
        }
    };

    let group_id = Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();

    let group = mls_group::ActiveModel {
        id: Set(group_id),
        mls_group_id: Set(mls_gid),
        name: Set(body.name.clone()),
        created_by: Set(auth.user_id),
        created_at: Set(now),
    };
    if let Err(e) = group.insert(&txn).await {
        let _ = txn.rollback().await;
        tracing::error!("Group insert failed: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to create group",
        )
        .into_response();
    }

    // Creator is a member.
    let creator_member = mls_group_member::ActiveModel {
        group_id: Set(group_id),
        user_id: Set(auth.user_id),
        joined_at: Set(now),
    };
    if let Err(e) = creator_member.insert(&txn).await {
        let _ = txn.rollback().await;
        tracing::error!("Creator membership insert failed: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to add creator",
        )
        .into_response();
    }

    // Each initial member gets a membership row + a Welcome to fetch on their
    // next poll.
    for (member_id, welcome_b64) in body.initial_members.iter().zip(&body.welcomes_b64) {
        let welcome_bytes = match decode_bytes(welcome_b64) {
            Ok(b) => b,
            Err(e) => {
                let _ = txn.rollback().await;
                return e.into_response();
            }
        };
        let member = mls_group_member::ActiveModel {
            group_id: Set(group_id),
            user_id: Set(*member_id),
            joined_at: Set(now),
        };
        if let Err(e) = member.insert(&txn).await {
            let _ = txn.rollback().await;
            tracing::error!("Member insert failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to add member",
            )
            .into_response();
        }

        let welcome = mls_welcome::ActiveModel {
            id: Set(Uuid::new_v4()),
            recipient_user_id: Set(*member_id),
            ciphertext: Set(welcome_bytes),
            delivered_at: Set(None),
            created_at: Set(now),
        };
        if let Err(e) = welcome.insert(&txn).await {
            let _ = txn.rollback().await;
            tracing::error!("Welcome insert failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to queue Welcome",
            )
            .into_response();
        }
    }

    if let Err(e) = txn.commit().await {
        tracing::error!("Commit failed: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Commit failed",
        )
        .into_response();
    }

    // Bell notifications for the invitees so they see a red dot without
    // having to open the inbox.
    let sender_name = display_name_for(db, auth.user_id).await;
    let single_title = format!("{sender_name} started a private conversation");
    notify_recipients(
        db,
        &body.initial_members,
        auth.user_id,
        &sender_name,
        &single_title,
        "/messages".to_string(),
    )
    .await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": group_id })),
    )
        .into_response()
}

async fn list_my_groups(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    use ideaforge_db::entities::user;

    let db = state.db.connection();
    let my_memberships = match mls_group_member::Entity::find()
        .filter(mls_group_member::Column::UserId.eq(auth.user_id))
        .all(db)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Member list failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to list groups",
            )
            .into_response();
        }
    };

    let ids: Vec<Uuid> = my_memberships.iter().map(|m| m.group_id).collect();
    if ids.is_empty() {
        let empty: Vec<GroupSummary> = Vec::new();
        return Json(serde_json::json!({ "data": empty })).into_response();
    }

    let groups = match mls_group::Entity::find()
        .filter(mls_group::Column::Id.is_in(ids.clone()))
        .order_by_desc(mls_group::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Group list failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch groups",
            )
            .into_response();
        }
    };

    // Fan out one query for all members of all my groups.
    let all_members = match mls_group_member::Entity::find()
        .filter(mls_group_member::Column::GroupId.is_in(ids))
        .all(db)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Member fan-out failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch members",
            )
            .into_response();
        }
    };

    // Batch-fetch display names for every user referenced.
    let user_ids: std::collections::HashSet<Uuid> = all_members.iter().map(|m| m.user_id).collect();
    let users = if user_ids.is_empty() {
        Vec::new()
    } else {
        match user::Entity::find()
            .filter(user::Column::Id.is_in(user_ids.into_iter().collect::<Vec<_>>()))
            .all(db)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("User lookup failed: {e}");
                return err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DB_ERROR",
                    "Failed to fetch users",
                )
                .into_response();
            }
        }
    };
    let name_map: std::collections::HashMap<Uuid, String> =
        users.into_iter().map(|u| (u.id, u.display_name)).collect();

    // Group memberships by group_id so we can attach them per row.
    let mut members_by_group: std::collections::HashMap<Uuid, Vec<MemberSummary>> =
        std::collections::HashMap::new();
    for m in all_members {
        members_by_group
            .entry(m.group_id)
            .or_default()
            .push(MemberSummary {
                user_id: m.user_id,
                display_name: name_map
                    .get(&m.user_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
            });
    }

    let summaries: Vec<GroupSummary> = groups
        .into_iter()
        .map(|g| GroupSummary {
            members: members_by_group.remove(&g.id).unwrap_or_default(),
            id: g.id,
            mls_group_id_b64: encode_bytes(&g.mls_group_id),
            name: g.name,
            created_by: g.created_by,
            created_at: g.created_at.to_rfc3339(),
        })
        .collect();

    Json(serde_json::json!({ "data": summaries })).into_response()
}

async fn add_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(group_id): Path<Uuid>,
    Json(body): Json<AddMembersRequest>,
) -> impl IntoResponse {
    if body.new_members.len() != body.welcomes_b64.len() {
        return err(
            StatusCode::BAD_REQUEST,
            "LENGTH_MISMATCH",
            "new_members and welcomes_b64 must have the same length",
        )
        .into_response();
    }

    let db = state.db.connection();
    if !is_group_member(db, group_id, auth.user_id).await {
        return err(
            StatusCode::FORBIDDEN,
            "NOT_MEMBER",
            "Not a member of this group",
        )
        .into_response();
    }

    let commit_bytes = match decode_bytes(&body.commit_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };

    let txn = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("txn begin failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Transaction failed",
            )
            .into_response();
        }
    };

    let now = chrono::Utc::now().fixed_offset();

    // Post the Commit to the group stream so existing members apply it.
    let commit_msg = mls_message::ActiveModel {
        id: sea_orm::NotSet,
        group_id: Set(group_id),
        sender_user_id: Set(auth.user_id),
        ciphertext: Set(commit_bytes),
        created_at: Set(now),
    };
    if let Err(e) = commit_msg.insert(&txn).await {
        let _ = txn.rollback().await;
        tracing::error!("Commit insert failed: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to post commit",
        )
        .into_response();
    }

    // Add membership rows + queue Welcomes.
    for (member_id, welcome_b64) in body.new_members.iter().zip(&body.welcomes_b64) {
        let welcome_bytes = match decode_bytes(welcome_b64) {
            Ok(b) => b,
            Err(e) => {
                let _ = txn.rollback().await;
                return e.into_response();
            }
        };
        let member = mls_group_member::ActiveModel {
            group_id: Set(group_id),
            user_id: Set(*member_id),
            joined_at: Set(now),
        };
        // Use insert with on-conflict-do-nothing semantics by ignoring
        // duplicate-key errors — members that were already in the group via
        // a rejoin are fine.
        if let Err(e) = member.insert(&txn).await {
            let msg = e.to_string();
            if !msg.contains("duplicate key") {
                let _ = txn.rollback().await;
                tracing::error!("Member insert failed: {e}");
                return err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DB_ERROR",
                    "Failed to add member",
                )
                .into_response();
            }
        }

        let welcome = mls_welcome::ActiveModel {
            id: Set(Uuid::new_v4()),
            recipient_user_id: Set(*member_id),
            ciphertext: Set(welcome_bytes),
            delivered_at: Set(None),
            created_at: Set(now),
        };
        if let Err(e) = welcome.insert(&txn).await {
            let _ = txn.rollback().await;
            tracing::error!("Welcome insert failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to queue Welcome",
            )
            .into_response();
        }
    }

    if let Err(e) = txn.commit().await {
        tracing::error!("Commit failed: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Commit failed",
        )
        .into_response();
    }

    let sender_name = display_name_for(db, auth.user_id).await;
    let single_title = format!("{sender_name} added you to a private conversation");
    notify_recipients(
        db,
        &body.new_members,
        auth.user_id,
        &sender_name,
        &single_title,
        "/messages".to_string(),
    )
    .await;

    StatusCode::NO_CONTENT.into_response()
}

/// Leave (and, if last member, tear down) a group.
///
/// The per-user `DELETE /groups/:id` is semantically "remove me from this
/// conversation." If I was the only remaining member, nothing else is going
/// to apply Commits or read messages, so we also cascade the group row +
/// message history. Welcomes aren't keyed by group_id so they persist
/// (harmlessly) on recipient queues.
async fn leave_group(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    use sea_orm::PaginatorTrait;

    let db = state.db.connection();

    if let Err(e) = mls_group_member::Entity::delete_many()
        .filter(mls_group_member::Column::GroupId.eq(group_id))
        .filter(mls_group_member::Column::UserId.eq(auth.user_id))
        .exec(db)
        .await
    {
        tracing::error!("Leave failed: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to leave group",
        )
        .into_response();
    }

    let remaining = mls_group_member::Entity::find()
        .filter(mls_group_member::Column::GroupId.eq(group_id))
        .count(db)
        .await
        .unwrap_or(0);

    if remaining == 0 {
        let _ = mls_message::Entity::delete_many()
            .filter(mls_message::Column::GroupId.eq(group_id))
            .exec(db)
            .await;
        let _ = mls_group::Entity::delete_many()
            .filter(mls_group::Column::Id.eq(group_id))
            .exec(db)
            .await;
    }

    StatusCode::NO_CONTENT.into_response()
}

// ---- Message endpoints -----------------------------------------------------

async fn post_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(group_id): Path<Uuid>,
    Json(body): Json<PostMessageRequest>,
) -> impl IntoResponse {
    let db = state.db.connection();
    if !is_group_member(db, group_id, auth.user_id).await {
        return err(
            StatusCode::FORBIDDEN,
            "NOT_MEMBER",
            "Not a member of this group",
        )
        .into_response();
    }

    let ciphertext = match decode_bytes(&body.ciphertext_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    // 1 MiB cap — generous for text, tight enough to prevent abuse.
    if ciphertext.is_empty() || ciphertext.len() > 1024 * 1024 {
        return err(
            StatusCode::BAD_REQUEST,
            "BAD_SIZE",
            "Ciphertext must be 1..=1048576 bytes",
        )
        .into_response();
    }

    let now = chrono::Utc::now().fixed_offset();
    let msg = mls_message::ActiveModel {
        id: sea_orm::NotSet,
        group_id: Set(group_id),
        sender_user_id: Set(auth.user_id),
        ciphertext: Set(ciphertext),
        created_at: Set(now),
    };
    match msg.insert(db).await {
        Ok(inserted) => {
            // Fan out bell notifications to every group member except the sender.
            let recipients: Vec<Uuid> = mls_group_member::Entity::find()
                .filter(mls_group_member::Column::GroupId.eq(group_id))
                .filter(mls_group_member::Column::UserId.ne(auth.user_id))
                .all(db)
                .await
                .map(|rows| rows.into_iter().map(|r| r.user_id).collect())
                .unwrap_or_default();
            if !recipients.is_empty() {
                let sender_name = display_name_for(db, auth.user_id).await;
                let single_title = format!("New message from {sender_name}");
                notify_recipients(
                    db,
                    &recipients,
                    auth.user_id,
                    &sender_name,
                    &single_title,
                    "/messages".to_string(),
                )
                .await;
            }
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "id": inserted.id })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Message insert failed: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to post message",
            )
            .into_response()
        }
    }
}

async fn list_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(group_id): Path<Uuid>,
    Query(params): Query<ListMessagesQuery>,
) -> impl IntoResponse {
    let db = state.db.connection();
    if !is_group_member(db, group_id, auth.user_id).await {
        return err(
            StatusCode::FORBIDDEN,
            "NOT_MEMBER",
            "Not a member of this group",
        )
        .into_response();
    }

    let limit = params.limit.unwrap_or(100).clamp(1, 500);
    match mls_message::Entity::find()
        .filter(mls_message::Column::GroupId.eq(group_id))
        .filter(mls_message::Column::Id.gt(params.since))
        .order_by_asc(mls_message::Column::Id)
        .limit(limit)
        .all(db)
        .await
    {
        Ok(messages) => {
            let envelopes: Vec<MessageEnvelope> = messages
                .into_iter()
                .map(|m| MessageEnvelope {
                    id: m.id,
                    group_id: m.group_id,
                    sender_user_id: m.sender_user_id,
                    ciphertext_b64: encode_bytes(&m.ciphertext),
                    created_at: m.created_at.to_rfc3339(),
                })
                .collect();
            Json(serde_json::json!({ "data": envelopes })).into_response()
        }
        Err(e) => {
            tracing::error!("Message list failed: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to list messages",
            )
            .into_response()
        }
    }
}

// ---- Welcome endpoints -----------------------------------------------------

async fn list_welcomes(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    let db = state.db.connection();
    match mls_welcome::Entity::find()
        .filter(mls_welcome::Column::RecipientUserId.eq(auth.user_id))
        .filter(mls_welcome::Column::DeliveredAt.is_null())
        .order_by_asc(mls_welcome::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(welcomes) => {
            let envelopes: Vec<WelcomeEnvelope> = welcomes
                .into_iter()
                .map(|w| WelcomeEnvelope {
                    id: w.id,
                    ciphertext_b64: encode_bytes(&w.ciphertext),
                    created_at: w.created_at.to_rfc3339(),
                })
                .collect();
            Json(serde_json::json!({ "data": envelopes })).into_response()
        }
        Err(e) => {
            tracing::error!("Welcome list failed: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to list Welcomes",
            )
            .into_response()
        }
    }
}

/// Client calls this after successfully importing a Welcome into its keystore.
/// Marks the welcome as delivered so it isn't re-served on the next poll.
async fn ack_welcome(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let db = state.db.connection();
    let welcome = match mls_welcome::Entity::find_by_id(id).one(db).await {
        Ok(Some(w)) => w,
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Welcome not found").into_response();
        }
        Err(e) => {
            tracing::error!("Welcome lookup failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Lookup failed",
            )
            .into_response();
        }
    };

    if welcome.recipient_user_id != auth.user_id {
        return err(StatusCode::FORBIDDEN, "FORBIDDEN", "Not your welcome").into_response();
    }

    let mut active: mls_welcome::ActiveModel = welcome.into();
    active.delivered_at = Set(Some(chrono::Utc::now().fixed_offset()));
    match active.update(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Welcome ack failed: {e}");
            err(StatusCode::INTERNAL_SERVER_ERROR, "DB_ERROR", "Ack failed").into_response()
        }
    }
}

// ---- Helpers ---------------------------------------------------------------

async fn is_group_member(db: &sea_orm::DatabaseConnection, group_id: Uuid, user_id: Uuid) -> bool {
    mls_group_member::Entity::find()
        .filter(mls_group_member::Column::GroupId.eq(group_id))
        .filter(mls_group_member::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map(|r| r.is_some())
        .unwrap_or(false)
}

async fn display_name_for(db: &sea_orm::DatabaseConnection, user_id: Uuid) -> String {
    user::Entity::find_by_id(user_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|u| u.display_name)
        .unwrap_or_else(|| "Someone".to_string())
}

/// Insert or coalesce a "message" notification per recipient. If the user
/// already has an UNREAD notification of the same kind from the same sender,
/// we bump its count + title ("6 new messages from X") and created_at
/// instead of inserting a new row. That way the notifications page doesn't
/// explode into dozens of rows during an active conversation.
///
/// Best-effort on errors — the core message flow has already succeeded.
async fn notify_recipients(
    db: &sea_orm::DatabaseConnection,
    recipients: &[Uuid],
    sender_user_id: Uuid,
    sender_name: &str,
    single_title: &str,
    link_url: String,
) {
    let now = chrono::Utc::now().fixed_offset();
    for rid in recipients {
        let existing = notification::Entity::find()
            .filter(notification::Column::UserId.eq(*rid))
            .filter(notification::Column::Kind.eq(NotificationKind::Message))
            .filter(notification::Column::RelatedUserId.eq(sender_user_id))
            .filter(notification::Column::ReadAt.is_null())
            .one(db)
            .await;

        match existing {
            Ok(Some(row)) => {
                let new_count = row.count + 1;
                let mut active: notification::ActiveModel = row.into();
                active.count = Set(new_count);
                active.title = Set(format!("{new_count} new messages from {sender_name}"));
                active.created_at = Set(now);
                if let Err(e) = active.update(db).await {
                    tracing::warn!(
                        "coalesce notification failed for {rid} from {sender_user_id}: {e}"
                    );
                }
            }
            Ok(None) => {
                let note = notification::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    user_id: Set(*rid),
                    kind: Set(NotificationKind::Message),
                    title: Set(single_title.to_string()),
                    message: Set(format!("From {sender_name}")),
                    link_url: Set(Some(link_url.clone())),
                    read_at: Set(None),
                    created_at: Set(now),
                    related_user_id: Set(Some(sender_user_id)),
                    count: Set(1),
                };
                if let Err(e) = note.insert(db).await {
                    tracing::warn!("message notification insert failed for {rid}: {e}");
                }
            }
            Err(e) => {
                tracing::warn!("notification coalesce lookup failed: {e}");
            }
        }
    }
}
