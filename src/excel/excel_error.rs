use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExcelError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Calamine error: {0}")]
    Calamine(#[from] calamine::Error),

    #[error("Xlsx write error: {0}")]
    Xlsx(#[from] rust_xlsxwriter::XlsxError),

    #[error("Sheet not found: {0}")]
    SheetNotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}