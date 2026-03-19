use rust_embed::RustEmbed;

use std::io::{Cursor, Write};
use std::path::{PathBuf};
use anyhow::Result;
use tempfile::NamedTempFile;

#[derive(RustEmbed, Clone)]
#[folder = "assets/"]
pub struct Assets;


// ==================== 嵌入数据导出模板 ====================
#[derive(RustEmbed, Clone)]
#[folder = "templates/"]
pub struct Templates;

// 方法 1️⃣：写入临时文件 : 这是最常用的方法，兼容所有需要 Path 的 API。
/* 使用
fn main() -> Result<()> {
    let path: PathBuf = embedded_to_temp_path("report.html")?;

    // 可以像普通文件一样使用
    let content = std::fs::read_to_string(&path)?;
    println!("{}", content);

    Ok(())
}
 */
fn embedded_to_temp_path(file_name: &str) -> Result<PathBuf> {
    // 1️⃣ 从 Embedded 获取文件
    let file = Templates::get(file_name)
        .ok_or_else(|| anyhow::anyhow!("File {} not found", file_name))?;

    // 2️⃣ 创建临时文件
    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(&file.data)?; // 写入内存内容到临时文件

    // 3️⃣ 返回 PathBuf
    Ok(tmp.into_temp_path().to_path_buf()) // 自动删除时机受 TempPath 生命周期控制
}

// 方法 2️⃣：直接用内存实现 IO 接口（推荐纯 Rust）; 不占磁盘空间
fn read_embedded_file(file_name: &str) -> Result<()> {
    let file = Templates::get(file_name)
        .ok_or_else(|| anyhow::anyhow!("File {} not found", file_name))?;

    // 内存 Cursor 当作 Read 对象
    let mut reader = Cursor::new(&file.data);

    let mut content = String::new();
    use std::io::Read;
    reader.read_to_string(&mut content)?;

    println!("{}", content);
    Ok(())
}

/*fn open_embedded_excel(file_name: &str) -> Result<Xlsx<Cursor<&[u8]>>> {
    let file = Templates::get(file_name)
        .ok_or_else(|| anyhow::anyhow!("File {} not found", file_name))?;
    let workbook = Xlsx::new(Cursor::new(&file.data))?;
    Ok(workbook)
}*/