use axum::{
    extract::{self, Form},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AddUser {
    pub username: String,
    pub email: String,
}

/// Add a user to the database taking a form as input
#[tracing::instrument(
    name = "Adding a new user",
    skip(input, connection),
    fields(
        user_username = %input.username,
        user_email = %input.email
    )
)]
pub async fn add_user(
    Form(input): Form<AddUser>,
    connection: extract::Extension<PgPool>,
) -> StatusCode {
    match insert_user(&connection.0, &input).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tracing::instrument(name = "Saving new user details in the database", skip(form, pool))]
pub async fn insert_user(pool: &PgPool, form: &AddUser) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into axumstarter.user (id, username, email)
        values ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        form.username,
        form.email
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
