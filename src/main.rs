mod parsing;
mod position;
mod config;
mod indicators;
mod types;
mod equity;
mod backtest;
mod strategy;
mod strategies;

use chrono::NaiveDate;
fn main() {
    let test_date = NaiveDate::from_ymd_opt(2021,3,1).unwrap();
    let df_result = parsing::parquet::read_parquet_by_date(test_date);
    match df_result {
        Ok(df) => println!("{}", df.head(Some(5))),
        Err(e) => eprintln!("Error: {}", e),
    }
}
