use sqlx::MySqlPool;
use crate::model::user::User;

pub async fn find_all() -> Vec<User> {

    vec![
        User{id:1,name:"Alice".into(),age:23},
        User{id:2,name:"Bob".into(),age:29}
    ]
}

// 创建
pub async fn create_user(
    pool: &MySqlPool,
    name: &str,
    age: i32,
) -> Result<u64, sqlx::Error> {

    let result =
        sqlx::query(
            "INSERT INTO users_rust(name,age) VALUES (?,?)"
        )
            .bind(name)
            .bind(age)
            .execute(pool)
            .await?;

    Ok(result.last_insert_id())
}

pub async fn get_user(
    pool: &MySqlPool,
    id: i64,
) -> Result<User, sqlx::Error> {

    let user =
        sqlx::query_as::<_, User>(
            "SELECT id,name,age FROM users_rust WHERE id=?"
        )
            .bind(id)
            .fetch_one(pool)
            .await?;

    Ok(user)
}

pub async fn list_users(
    pool: &MySqlPool,
) -> Result<Vec<User>, sqlx::Error> {

    let users =
        sqlx::query_as::<_, User>(
            "SELECT id,name,age FROM users_rust"
        )
            .fetch_all(pool)
            .await?;

    Ok(users)
}

pub async fn update_user(
    pool: &MySqlPool,
    id: i64,
    name: &str,
    age: i32,
) -> Result<u64, sqlx::Error> {

    // 增加事务支持
    /*let mut tx = pool.begin().await?;

    sqlx::query("INSERT ...")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;*/

    let result =
        sqlx::query(
            "UPDATE users_rust SET name=?,age=? WHERE id=?"
        )
            .bind(name)
            .bind(age)
            .bind(id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected())
}

pub async fn delete_user(
    pool: &MySqlPool,
    id: i64,
) -> Result<u64, sqlx::Error> {

    let result =
        sqlx::query(
            "DELETE FROM users_rust WHERE id=?"
        )
            .bind(id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected())
}