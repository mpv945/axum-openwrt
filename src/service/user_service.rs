use sqlx::MySqlPool;
use crate::{
    repository::user_repo,
    model::user::{User,CreateUser},
    common::result::AppResult,
};

pub async fn find_all() -> Vec<User> {
    user_repo::find_all().await
}

pub async fn create_user(
    pool: &MySqlPool,
    input: CreateUser,
) -> AppResult<User> {

    let id =
        user_repo::create_user(
            pool,
            &input.name,
            input.age
        ).await?;

    let user =
        user_repo::get_user(pool, id as i64)
            .await?;

    Ok(user)
}

pub async fn list_users(
    pool: &MySqlPool,
) -> AppResult<Vec<User>> {

    let users = user_repo::list_users(pool)
        .await?;

    Ok(users)
}

pub async fn get_user(
    pool: &MySqlPool,
    id: i64,
) -> AppResult<User> {

    let user = user_repo::get_user(pool, id)
        .await?;

    Ok(user)
}

pub async fn delete_user(
    pool: &MySqlPool,
    id: i64,
) -> AppResult<()> {

    user_repo::delete_user(pool, id).await?;

    Ok(())
}