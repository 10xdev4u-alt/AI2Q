use aiql_core::crawlers::PostgresSchemaCrawler;
use aiql_core::SchemaCrawler;
use pyo3::prelude::*;
use sqlx::postgres::PgPoolOptions;

#[pyfunction]
fn crawl_postgres(db_url: String) -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let schema_json = rt.block_on(async {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&db_url)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let crawler = PostgresSchemaCrawler::new(pool);
        let schema = crawler.crawl().await.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        serde_json::to_string(&schema).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    })?;

    Ok(schema_json)
}

#[pymodule]
fn aiql(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crawl_postgres, m)?)?;
    Ok(())
}
