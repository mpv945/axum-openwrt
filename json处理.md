下面给你一套**生产可用的 JSON ↔ String ↔ BTreeMap 工具类封装**，完全适配你当前的 Rust 技术栈（serde + axum + reqwest）。

---

# 一、Cargo.toml

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
```

---

# 二、工具类实现（核心）

## json_util.rs

```rust
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, Map};
use std::collections::BTreeMap;

pub struct JsonUtil;

impl JsonUtil {
    // =========================
    // JSON <-> String
    // =========================

    /// struct / json -> string
    pub fn to_string<T: Serialize>(data: &T) -> Result<String> {
        Ok(serde_json::to_string(data)?)
    }

    /// string -> struct
    pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
        Ok(serde_json::from_str(s)?)
    }

    /// string -> Value
    pub fn str_to_value(s: &str) -> Result<Value> {
        Ok(serde_json::from_str(s)?)
    }

    /// Value -> string（pretty）
    pub fn to_pretty_string(value: &Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(value)?)
    }

    // =========================
    // BTreeMap <-> JSON
    // =========================

    /// BTreeMap -> JSON Value
    pub fn map_to_json(map: &BTreeMap<String, String>) -> Value {
        let mut json_map = Map::new();

        for (k, v) in map {
            json_map.insert(k.clone(), Value::String(v.clone()));
        }

        Value::Object(json_map)
    }

    /// JSON Value -> BTreeMap<String, String>
    /// ⚠️ 只适用于扁平 JSON（value 必须是 string）
    pub fn json_to_map(value: &Value) -> Result<BTreeMap<String, String>> {
        let mut map = BTreeMap::new();

        if let Value::Object(obj) = value {
            for (k, v) in obj {
                let val = match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "unsupported value type for key: {}",
                            k
                        ))
                    }
                };
                map.insert(k.clone(), val);
            }
        } else {
            return Err(anyhow::anyhow!("not a json object"));
        }

        Ok(map)
    }

    // =========================
    // String <-> BTreeMap
    // =========================

    /// JSON字符串 -> BTreeMap
    pub fn str_to_map(s: &str) -> Result<BTreeMap<String, String>> {
        let value: Value = serde_json::from_str(s)?;
        Self::json_to_map(&value)
    }

    /// BTreeMap -> JSON字符串
    pub fn map_to_string(map: &BTreeMap<String, String>) -> Result<String> {
        let value = Self::map_to_json(map);
        Self::to_string(&value)
    }
}
```

---

# 三、使用示例

---

## 1️⃣ JSON ↔ String

```rust
use serde_json::json;

let data = json!({
    "name": "Alice",
    "age": 30
});

// JSON -> String
let s = JsonUtil::to_string(&data)?;

// String -> JSON
let v = JsonUtil::str_to_value(&s)?;
```

---

## 2️⃣ struct ↔ JSON String

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    age: u32,
}

let user = User {
    name: "Alice".into(),
    age: 30,
};

let s = JsonUtil::to_string(&user)?;

let u: User = JsonUtil::from_str(&s)?;
```

---

## 3️⃣ BTreeMap ↔ JSON

```rust
use std::collections::BTreeMap;

let mut map = BTreeMap::new();
map.insert("name".to_string(), "Alice".to_string());
map.insert("age".to_string(), "30".to_string());

// map -> json
let json = JsonUtil::map_to_json(&map);

// json -> map
let map2 = JsonUtil::json_to_map(&json)?;
```

---

## 4️⃣ JSON String ↔ Map

```rust
let s = r#"{"name":"Alice","age":30}"#;

// string -> map
let map = JsonUtil::str_to_map(s)?;

// map -> string
let s2 = JsonUtil::map_to_string(&map)?;
```

---

# 四、⚠️ 重要限制（必须知道）

## ❗1：json_to_map 只支持扁平结构

```json
{
  "a": 1,
  "b": "x"
}
```

✔ 支持

---

```json
{
  "user": {
    "name": "Alice"
  }
}
```

❌ 不支持（嵌套）

---

# 五、进阶（支持嵌套 JSON → Map）

如果你需要“展开”嵌套 JSON 👇

```rust
pub fn flatten_json(
    value: &Value,
    prefix: String,
    map: &mut BTreeMap<String, String>,
) {
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let new_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(v, new_key, map);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_key = format!("{}[{}]", prefix, i);
                flatten_json(v, new_key, map);
            }
        }
        _ => {
            map.insert(prefix, value.to_string());
        }
    }
}
```

---

# 六、生产最佳实践（结合你场景）

你现在可以形成标准：

### ✔ API 请求

* body → struct / json
* query → BTreeMap
* sign → BTreeMap 排序

---

### ✔ JSON 处理策略

| 场景  | 推荐       |
| --- | -------- |
| 强类型 | struct   |
| 动态  | Value    |
| 签名  | BTreeMap |

---

# 七、总结一句话

👉 这套工具帮你统一了：

* JSON ⇄ String
* JSON ⇄ Map
* String ⇄ Map

---

# 如果你下一步要做（我强烈建议）

我可以帮你升级成：

✅ 类似 Java Hutool `JSONUtil` + `MapUtil`
✅ 自动：

* null 过滤
* key 排序
* snake_case ↔ camelCase
* 签名专用 canonical JSON

直接说：

👉 **“做 Rust 版 Hutool 工具库”**
