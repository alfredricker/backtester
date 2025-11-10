use polars::prelude::*;
use chrono::NaiveDate;
use std::path::PathBuf;
/// Reads a parquet file for a given date.
/// 
/// # Arguments
/// * `date` - A NaiveDate specifying the date to load
/// 
/// # Returns
/// * `Result<DataFrame>` - The loaded DataFrame or an error
/// Box<dyn std::error::Error is a boxed pointer to any type that implements the Error trait
/// The ? operator automatically converts std::io::Error and Polars eerrors -> Box<dyn Error>
pub fn read_parquet_by_date(date: NaiveDate) -> std::result::Result<DataFrame, Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(super::DATA_LOAD_DIR)
        .join(format!("{}{}.parquet", super::FILE_PREFIX, date.format("%Y-%m-%d")));
    
    let file = std::fs::File::open(&file_path)?;
    let df = polars::prelude::ParquetReader::new(file).finish()?;
    Ok(df)
}