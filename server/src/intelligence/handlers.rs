use axum::extract::{Json, Path, Query, State};
use axum::routing::{delete, get, post, put};
use axum::Router;
use uuid::Uuid;

use crate::common::{ApiResponse, AppError, AppResult, AppState, AuthUser};

use super::models::{
    CreateReminderRequest, ReminderQuery, ReminderResponse, UpdateReminderRequest,
};
use super::repo;

/// GET /api/reminders
async fn list_reminders_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ReminderQuery>,
) -> AppResult<ApiResponse<Vec<ReminderResponse>>> {
    let reminders = repo::list_reminders(&state.pool, auth.user_id, query.status).await?;
    let responses: Vec<ReminderResponse> = reminders.into_iter().map(Into::into).collect();
    Ok(ApiResponse::success(responses))
}

/// GET /api/reminders/:id
async fn get_reminder_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<ReminderResponse>> {
    let reminder = repo::find_reminder_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("reminder {id} not found")))?;

    if reminder.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("reminder {id} not found")));
    }

    Ok(ApiResponse::success(ReminderResponse::from(reminder)))
}

/// POST /api/reminders
async fn create_reminder_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReminderRequest>,
) -> AppResult<ApiResponse<ReminderResponse>> {
    let reminder = repo::create_reminder(
        &state.pool,
        auth.user_id,
        body.data_object_id,
        &body.title,
        body.description.as_deref(),
        body.trigger_at,
        body.repeat_rule.as_ref(),
    )
    .await?;

    Ok(ApiResponse::success(ReminderResponse::from(reminder)))
}

/// PUT /api/reminders/:id
async fn update_reminder_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateReminderRequest>,
) -> AppResult<ApiResponse<ReminderResponse>> {
    // Verify ownership before updating.
    let existing = repo::find_reminder_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("reminder {id} not found")))?;

    if existing.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("reminder {id} not found")));
    }

    let reminder = repo::update_reminder(
        &state.pool,
        id,
        body.title.as_deref(),
        body.description.as_deref(),
        body.trigger_at,
        body.repeat_rule.as_ref(),
    )
    .await?;

    Ok(ApiResponse::success(ReminderResponse::from(reminder)))
}

/// DELETE /api/reminders/:id
async fn delete_reminder_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<()>> {
    // Verify ownership before deleting.
    let existing = repo::find_reminder_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("reminder {id} not found")))?;

    if existing.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("reminder {id} not found")));
    }

    repo::delete_reminder(&state.pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// POST /api/reminders/:id/dismiss
async fn dismiss_reminder_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<ReminderResponse>> {
    // Verify ownership before dismissing.
    let existing = repo::find_reminder_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("reminder {id} not found")))?;

    if existing.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("reminder {id} not found")));
    }

    let reminder = repo::dismiss_reminder(&state.pool, id).await?;
    Ok(ApiResponse::success(ReminderResponse::from(reminder)))
}

/// Build the router subtree for the intelligence module.
///
/// Mount points:
///   - `/api/reminders/*` — CRUD + dismiss
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/reminders", get(list_reminders_handler).post(create_reminder_handler))
        .route(
            "/api/reminders/{id}",
            get(get_reminder_handler)
                .put(update_reminder_handler)
                .delete(delete_reminder_handler),
        )
        .route("/api/reminders/{id}/dismiss", post(dismiss_reminder_handler))
}
