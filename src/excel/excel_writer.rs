use rust_xlsxwriter::{Workbook, Worksheet};
use crate::excel::excel_error::ExcelError;

pub struct ExcelWriter {
    workbook: Workbook,
}

impl ExcelWriter {
    pub fn new() -> Self {
        Self {
            workbook: Workbook::new(),
        }
    }

    pub fn write_sheet(
        mut self,
        sheet_name: &str,
        data: &[Vec<String>],
    ) -> Result<Self, ExcelError> {
        let worksheet = self.workbook.add_worksheet();

        worksheet.set_name(sheet_name)?;

        for (row_idx, row) in data.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                worksheet.write_string(row_idx as u32, col_idx as u16, value)?;
            }
        }

        Ok(self)
    }

    pub fn save(mut self, path: &str) -> Result<(), ExcelError> {
        self.workbook.save(path)?; // workbook 需要 &mut self
        Ok(())
    }
}