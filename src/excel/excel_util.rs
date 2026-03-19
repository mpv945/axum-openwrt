use crate::excel::excel_error::ExcelError;
use crate::excel::excel_reader::ExcelReader;
use crate::excel::excel_writer::ExcelWriter;

pub struct ExcelUtil;

impl ExcelUtil {
    pub fn read(path: &str, sheet: &str) -> Result<Vec<Vec<String>>, ExcelError> {
        ExcelReader::new(path).read_sheet(sheet)
    }

    pub fn read_embedded(path: &str, sheet: &str) -> anyhow::Result<()> {
        ExcelReader::new(path).read_embedded_excel(sheet,path)
    }

    pub fn write(
        path: &str,
        sheet: &str,
        data: &[Vec<String>],
    ) -> Result<(), ExcelError> {
        ExcelWriter::new()
            .write_sheet(sheet, data)?
            .save(path)
    }

    /*pub fn edit(file_name: &str, sheet_name: &str, data: Vec<Vec<String>>) -> Result<()> {
        let file_data = Templates::get(file_name)
            .ok_or_else(|| anyhow!("File {} not found in templates", file_name))?;

        let mut workbook: Xlsx<Cursor<&[u8]>> = Xlsx::new(Cursor::new(&file_data.data))?;

        let sheet = workbook
            .worksheet_mut(sheet_name)
            .map_err(|e| anyhow!("Error accessing sheet '{}': {}", sheet_name, e))?;

        for (row_index, row) in data.iter().enumerate() {
            for (col_index, value) in row.iter().enumerate() {
                sheet.write_string(row_index as u32, col_index as u16, value)
                    .map_err(|e| anyhow!("Error writing to sheet '{}': {}", sheet_name, e))?;
            }
        }

        // 保存修改后的工作簿
        workbook.save(file_name).map_err(|e| anyhow!("Error saving file '{}': {}", file_name, e))?;

        Ok(())
    }*/
}