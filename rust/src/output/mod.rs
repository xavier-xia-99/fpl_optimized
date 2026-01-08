//! Output formatting for optimization results

mod csv_output;
mod display;
mod json_output;

pub use csv_output::write_csv;
pub use display::display_result;
pub use json_output::write_json;
