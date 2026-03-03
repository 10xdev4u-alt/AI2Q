use aiql_core::crawlers::PostgresSchemaCrawler;
use aiql_core::SchemaCrawler;
use libc::c_char;
use sqlx::postgres::PgPoolOptions;
use std::ffi::{CStr, CString};
use tokio::runtime::Runtime;

#[no_mangle]
pub extern "C" fn aiql_crawl_postgres(db_url: *const c_char) -> *mut c_char {
    let db_url_str = unsafe {
        CStr::from_ptr(db_url).to_str().expect("Invalid UTF-8")
    };

    let rt = Runtime::new().expect("Failed to create runtime");
    let schema_json = rt.block_on(async {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(db_url_str)
            .await
            .expect("Failed to connect to database");

        let crawler = PostgresSchemaCrawler::new(pool);
        let schema = crawler.crawl().await.expect("Failed to crawl schema");
        serde_json::to_string(&schema).expect("Failed to serialize schema")
    });

    CString::new(schema_json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn aiql_free_string(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
