use calamine::{open_workbook_auto, Data, Reader, Xlsx};
use crate::excel::excel_error::ExcelError;
use std::borrow::Cow;
use std::io::Cursor;
use rust_embed::EmbeddedFile;
use crate::r#static::embed::Templates;

pub struct ExcelReader {
    path: String,
}

impl ExcelReader {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// 流式读取，每行通过回调处理
    /*
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
     */
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
    pub fn read_sheet(
        &self,
        sheet_name: &str,
    ) -> Result<Vec<Vec<String>>, ExcelError> {
        let mut workbook = open_workbook_auto(&self.path)?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| {
                ExcelError::InvalidData(format!(
                    "Failed to read sheet [{}]: {}",
                    sheet_name, e
                ))
            })?;

        let mut result = Vec::with_capacity(range.height());

        for row in range.rows() {
            let mut row_vec = Vec::with_capacity(row.len());

            for cell in row {
                row_vec.push(Self::cell_to_string(cell));
            }

            result.push(row_vec);
        }

        Ok(result)
    }

    #[inline]
    fn cell_to_string(cell: &Data) -> String {
        match cell {
            Data::Empty => String::new(),
            Data::String(s) => s.to_owned(),
            Data::Float(f) => float_fmt(*f),
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            Data::Error(_) => String::new(),
            Data::DateTime(f) => f.to_string(),
            Data::DateTimeIso(s) => s.clone(),   // ✅ 补上
            Data::DurationIso(s) => s.clone(),   // ✅ 补上
        }
    }


    // 从内嵌文件读取
    pub fn read_embedded_excel(&self, sheet_name: &str, file_name: &str) -> anyhow::Result<()> {
        // 1️⃣ 获取 EmbeddedFile
        let file_data = Templates::get(file_name)
            .ok_or_else(|| anyhow::anyhow!("File {} not found in templates", file_name))?;

        // 2️⃣ 从内存创建 calamine Xlsx
        let mut workbook: Xlsx<Cursor<&Cow<[u8]>>> = Xlsx::new(Cursor::new(&file_data.data))?;

        // 3️⃣ 获取 sheet
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|_| anyhow::anyhow!("Sheet {} not found", sheet_name))?; // ✅ 不要 ok_or_else

        // 4️⃣ 流式处理
        for row in range.rows() {
            let row_cow: Vec<Cow<str>> = row.iter().map(|cell| match cell {
                Data::Empty => Cow::Borrowed(""),
                Data::String(s) => Cow::Borrowed(s.as_str()), // ✅ 关键改动
                Data::Float(f) => {
                    if f.fract() == 0.0 { Cow::Owned((*f as i64).to_string()) }
                    else { Cow::Owned(f.to_string()) }
                },
                Data::Int(i) => Cow::Owned(i.to_string()),
                Data::Bool(b) => Cow::Owned(b.to_string()),
                Data::DateTime(f) => Cow::Owned(f.to_string()),
                Data::DateTimeIso(s) => Cow::Borrowed(s.as_str()),
                Data::DurationIso(s) => Cow::Borrowed(s.as_str()),
                Data::Error(_) => Cow::Borrowed(""),
                _ => Cow::Borrowed(""),
            }).collect();

            println!("{:?}", row_cow);
        }

        Ok(())
    }
}

async fn export_template(
    //State(state): State<Arc<AppState>>,
) -> Result<Vec<u8>, (axum::http::StatusCode, String)> {
    let template = Templates::get("report.html")
        .ok_or((axum::http::StatusCode::NOT_FOUND, "模板不存在".to_string()))?;

    /*println!(
        "使用配置: {}:{} | DB: {:?}",
        state.config.server.host,
        state.config.server.port,
        state.config.database.as_ref().map(|d| &d.url)
    );*/

    Ok(template.data.into_owned())
}

// 可选优化：避免 format! 带来的开销
#[inline]
fn float_fmt(f: f64) -> String {
    if f.fract() == 0.0 {
        // 没有小数 → 用整数方式（更快）
        (f as i64).to_string()
    } else {
        // 有小数
        f.to_string()
    }
}

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