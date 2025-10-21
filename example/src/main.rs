use std::sync::Arc;

use rusqlite::Connection;
use vector_xlite::VectorXLite;

use crate::simple_example::run_simple_example;
use crate::complex_example::run_complex_example;

mod simple_example;
mod complex_example;

fn main() {
    let sqlite_connection = Arc::new(Connection::open_in_memory().unwrap());

    
    let vlite = VectorXLite::new(Arc::clone(&sqlite_connection)).unwrap();

    run_simple_example(&vlite);

    run_complex_example(&vlite, sqlite_connection);
}
