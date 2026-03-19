明白了，你现在想做的优化目标是：

* **流处理**：不要一次性把整个 Excel 读进内存（尤其是百万级行数据），而是按行逐条处理。
* **减少 clone / heap 分配**：尽量使用 `&str` 或 `Cow<'_, str>`，只在必要时才生成 `String`。

下面我帮你整理一个 **生产级优化方案**，基于 `calamine` + Rust 流式处理理念。

---

# 一、核心思路

1. **Reader 按行迭代**
   calamine 的 `Range` 可以按行返回 `Rows` 迭代器，你可以直接 `for row in range.rows()` 逐行处理。

2. **使用 Cow<'_, str> 避免无意义 clone**

    * 单元格本身是 `Data::String(s)`，你可以直接返回 `&str`。
    * 数字、布尔值等才生成 `String`，返回 `Cow::Owned`。

3. **统一流式接口**
   用一个 **callback / iterator** 来处理每一行数据，避免一次性收集 `Vec<Vec<String>>`。

---

# 二、工具函数：单元格 → Cow<'_, str>

```rust
use calamine::Data;
use std::borrow::Cow;

#[inline]
fn cell_to_cow(cell: &Data) -> Cow<'_, str> {
    match cell {
        Data::Empty => Cow::Borrowed(""),
        Data::String(s) => Cow::Borrowed(s),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                Cow::Owned((*f as i64).to_string())
            } else {
                Cow::Owned(f.to_string())
            }
        }
        Data::Int(i) => Cow::Owned(i.to_string()),
        Data::Bool(b) => Cow::Owned(b.to_string()),
        Data::DateTime(f) => Cow::Owned(f.to_string()),
        Data::DateTimeIso(s) => Cow::Borrowed(s),
        Data::DurationIso(s) => Cow::Borrowed(s),
        Data::Error(_) => Cow::Borrowed(""),
        _ => Cow::Borrowed(""),
    }
}
```

---

# 三、流式读取接口（Callback 风格）

```rust
use calamine::{open_workbook_auto, Reader};
use crate::ExcelError;

pub struct ExcelReader {
    path: String,
}

impl ExcelReader {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// 流式读取，每行通过回调处理
    pub fn read_sheet_stream<F>(&self, sheet_name: &str, mut callback: F) -> Result<(), ExcelError>
    where
        F: FnMut(&[Cow<str>]),
    {
        let mut workbook = open_workbook_auto(&self.path)?;
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|_| ExcelError::SheetNotFound(sheet_name.to_string()))?;

        for row in range.rows() {
            let row_cow: Vec<Cow<str>> = row.iter().map(cell_to_cow).collect();
            callback(&row_cow);
        }

        Ok(())
    }
}
```

---

# 四、使用示例（流式处理）

```rust
fn main() -> Result<(), ExcelError> {
    let reader = ExcelReader::new("input.xlsx");

    // 按行流式处理
    reader.read_sheet_stream("Sheet1", |row| {
        // row: &[Cow<str>]，可以直接使用
        for cell in row {
            print!("{}\t", cell);
        }
        println!();
    })?;

    Ok(())
}
```

✅ 优点：

* 不一次性分配 `Vec<Vec<String>>`，节省内存
* 对百万行 Excel 也不会 OOM
* 使用 `Cow` 减少 clone / heap 分配

---

# 五、流式写入接口（Writer）

```rust
use rust_xlsxwriter::{Workbook, Worksheet};

pub struct ExcelWriter {
    workbook: Workbook,
}

impl ExcelWriter {
    pub fn new() -> Self {
        Self {
            workbook: Workbook::new(),
        }
    }

    pub fn write_sheet_stream<F>(mut self, sheet_name: &str, mut row_generator: F) -> Result<Self, ExcelError>
    where
        F: FnMut(&mut dyn FnMut(u32, u16, &str) -> Result<(), rust_xlsxwriter::XlsxError>) -> Result<(), ExcelError>,
    {
        let worksheet = self.workbook.add_worksheet();
        worksheet.set_name(sheet_name)?;

        // 通过闭包按行写
        row_generator(&mut |r, c, value| worksheet.write_string(r, c, value))?;

        Ok(self)
    }

    pub fn save(self, path: &str) -> Result<(), ExcelError> {
        self.workbook.save(path)?;
        Ok(())
    }
}
```

---

# 六、使用示例（流式写入）

```rust
let writer = ExcelWriter::new();

writer
    .write_sheet_stream("Result", |write_cell| {
        for row_idx in 0..1000000 {
            for col_idx in 0..10 {
                write_cell(row_idx, col_idx, "data")?;
            }
        }
        Ok(())
    })?
    .save("output.xlsx")?;
```

✅ 优点：

* 支持 **百万行流式写入**
* 不会一次性生成全部 `Vec<Vec<String>>`
* 可以配合生产级数据流（Kafka / Parquet）

---

# 七、总结优化要点

| 优化点      | 实现方式                                 |
| -------- | ------------------------------------ |
| 流式读取     | `for row in range.rows()` + callback |
| 流式写入     | 逐行闭包写入，不生成大数组                        |
| 减少 clone | `Cow<'_, str>`，仅必要时生成 `String`       |
| 类型安全     | 单元格到字符串统一封装                          |
| 大文件支持    | 不落内存，支持百万行 Excel                     |

---

如果你愿意，我可以帮你做一版 **完整企业级 Rust Excel 工具类模板**，对标 Java EasyExcel：

* 流式读写
* Row → Struct 自动映射
* 百万行零拷贝
* 支持 Kafka / Parquet 直接写入

这样就能直接在大数据生产环境用。

你希望我帮你做这个版本吗？
