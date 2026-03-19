use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use serde::Serialize;
use serde_json::json;
use crate::service::user_service;
use crate::model::user::{CreateUser, User};
use crate::model::app_state::{AppState, CurrentUser};
use serde_json::{Map, Value};

pub async fn list_users_simple() -> Json<Vec<User>> {
    let users = user_service::find_all().await;
    Json(users)
}

use crate::{common::{error::AppError, response::ApiResponse, result::AppResult}, utils};
use crate::common::result::ApiResult;
use crate::utils::jwt::JwtUtil;

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: u64,
    pub username: String,
    pub app_name: String,
}

pub async fn profile(Extension(user): Extension<CurrentUser>,
                     State(state): State<Arc<AppState>>)
    -> AppResult<Json<ApiResponse<UserProfile>>> {
    let count = *state.counter.read().unwrap(); // 读取 counter
    println!("读写锁 count: {}", count);
    let user = UserProfile {
        id: 1,
        //username: "admin".to_string(),
        username: user.username.clone(),
        app_name: state.config.jwt_secret.clone(),
    };

    // 增加计数器
    state.increment_counter();
    println!("读写锁 增加计数后的 count: {}", count);
    Ok(Json(ApiResponse::success(user)))
}

pub async fn biz_error_demo() -> AppResult<Json<ApiResponse<()>>> {
    Err(AppError::BizError("用户名已存在".to_string()))
}

pub async fn bad_request_demo() -> AppResult<Json<ApiResponse<()>>> {
    Err(AppError::BadRequest("请求参数不合法".to_string()))
}

pub async fn internal_error_demo() -> AppResult<Json<ApiResponse<()>>> {
    Err(AppError::InternalWithMsg("数据库连接失败".to_string()))
}

pub async fn panic_error_demo() -> AppResult<Json<ApiResponse<()>>> {
    panic!("boom");
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<User>>> {
    let user = user_service::get_user(&state.db, id).await?;
    Ok(Json(ApiResponse::success(user)))
}

/*pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {

    user_service::delete_user(&state.db, id).await?;

    Ok(Json(ApiResponse::ok()))
}*/
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> ApiResult<()> {
    user_service::delete_user(&state.db, id).await?;
    Ok(Json(ApiResponse::ok()))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> AppResult<Json<ApiResponse<User>>> {

    let user =
        user_service::create_user(
            &state.db,
            payload
        )
            .await?;
    Ok(Json(ApiResponse::success(user)))
}

#[derive(Serialize)]
struct Query {
    page: u32,
    size: u32,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<ApiResponse<Vec<User>>>> {

    let users =
        user_service::list_users(&state.db)
            .await?;

    // 发送stream消息
    state.redis
        .stream_add("orders", "data", "hello")
        .await
        .unwrap();

    // 发送普通
    state.redis
        .publish("events", "hello from axum")
        .await
        .unwrap();

    state.redis.set("rust:test","test").await.unwrap();

    // TTL
    state.redis.set_ex("k1", "v1", 60).await.unwrap();

    // 分布式锁
    let ok = state.redis.lock("lock:test", "uuid123", 10).await.unwrap();
    if ok {
        println!("lock acquired");
        state.redis.unlock("lock:test", "uuid123").await.unwrap();
    }

    // Hash
    state.redis.h_set("user:1", "name", "tom").await.unwrap();
    let name = state.redis.h_get("user:1", "name").await.unwrap();

    // Set
    state.redis.s_add("tags", "rust").await.unwrap();
    let tags = state.redis.s_members("tags").await.unwrap();

    // jwt
    // 生成 JWT
    let token = JwtUtil::generate("user123", 3600).unwrap();
    // 校验 JWT: async fn verify_jwt(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    //let token = body["token"].as_str().unwrap();
    let claims = JwtUtil::verify(token.as_str()).unwrap();
    println!("令牌数据 : {}",serde_json::json!(claims));

    // 加密处理
    let hash = utils::crypt_util::CryptUtil::crypt("123456", "6", "salt123").unwrap();
    println!("hash = {}", hash);
    let ok = utils::crypt_util::CryptUtil::verify("123456", &hash);
    println!("verify = {}", ok);

    // http 请求
    let payload = json!({
    "user": {
        "id": 1001,
        "name": "Alice",
        "roles": ["admin", "user"],
        "profile": {
            "age": 30,
            "email": "alice@example.com"
        }
    },
    "items": [
        {
            "id": 1,
            "name": "item1",
            "tags": ["a", "b"]
        },
        {
            "id": 2,
            "name": "item2",
            "tags": ["c", "d"]
        }
    ],
    "meta": {
        "timestamp": 1710000000,
        "source": "system"
    }
    });
    let url = "https://httpbin.org/post";
    let result: serde_json::Value = state.http.post_json(url, &payload).await.unwrap();

    /*let params = [
        ("page", "1"),
        ("size", "10"),
        ("keyword", "rust"),
    ];
    let resp11 =  state.http
        .get_json("https://httpbin.org/get")
        //.query(&params)
        //.send()
        .await.unwrap();*/
    let q = Query { page: 1, size: 10 };

    let result: serde_json::Value =  state.http
        .get_with_query("https://httpbin.org/get", &q)

        .await.unwrap();


    // 完全动态构建（Map + Value）
    let mut root = Map::new();

    // 基本字段
    root.insert("name".to_string(), Value::from("Alice"));
    root.insert("age".to_string(), Value::from(30));

    // 嵌套 JSON
    let mut profile = Map::new();
    profile.insert("email".to_string(), Value::from("alice@example.com"));

    root.insert("profile".to_string(), Value::Object(profile));

    // 数组
    let arr = vec![
        Value::from("admin"),
        Value::from("user"),
    ];

    root.insert("roles".to_string(), Value::Array(arr));

    let json = Value::Object(root);

    println!("{}", json.to_string());

    // 动态 JSON + 数组嵌套（复杂结构）
    let mut items = vec![];

    for i in 1..=3 {
        items.push(json!({
            "id": i,
            "name": format!("item-{}", i)
        }));
    }

    let result = json!({
        "list": items,
        "total": 3
    });
    println!("{}", result.to_string());

    Ok(Json(ApiResponse::success(users)))
}

