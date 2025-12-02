use crate::complex_example::run_complex_example;
use crate::simple_example::run_simple_example;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use vector_xlite::{VectorXLite, customizer::SqliteConnectionCustomizer};

mod complex_example;
mod simple_example;

fn main() {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder()
        .max_size(15)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)
        .unwrap();

    let vlite = VectorXLite::new(pool.clone()).unwrap();

    run_simple_example(&vlite);

    println!("--- * ---");

    run_complex_example(&vlite, pool);
}
