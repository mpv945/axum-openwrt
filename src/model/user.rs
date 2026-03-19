/*use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub struct User {

    pub id:i64,
    pub name:String
}*/

use serde::{Serialize, Deserialize};

/*
CREATE TABLE users
(
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100),
    age INT
);
 */
// sqlx::FromRow 可以自动映射查询结果。
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {

    pub id: i64,

    pub name: String,

    pub age: i32,
}

#[derive(Deserialize)]
pub struct CreateUser {

    pub name: String,

    pub age: i32,
}