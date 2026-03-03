use aiql_core::crawlers::PostgresSchemaCrawler;
use aiql_core::translator::MockTranslator;
use aiql_core::healer::SimpleHealer;
use aiql_core::execution::PostgresExecutionEngine;
use aiql_core::cache::InMemorySemanticCache;
use aiql_core::privacy::SimplePrivacyGuard;
use aiql_core::client::SmartClient;
use aiql_core::{SchemaCrawler, Translator, QueryHealer, Schema, SafetyPolicy};
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
pub extern "C" fn aiql_translate(prompt: *const c_char, schema_json: *const c_char) -> *mut c_char {
    let prompt_str = unsafe { CStr::from_ptr(prompt).to_str().expect("Invalid UTF-8") };
    let schema_json_str = unsafe { CStr::from_ptr(schema_json).to_str().expect("Invalid UTF-8") };

    let schema: Schema = serde_json::from_str(schema_json_str).expect("Failed to deserialize schema");

    let rt = Runtime::new().expect("Failed to create runtime");
    let plan_json = rt.block_on(async {
        let translator = MockTranslator;
        let context = aiql_core::Context { now: chrono::Utc::now(), tenant_id: None };
        let result = translator.translate(prompt_str, &schema, aiql_core::DatabaseDialect::Postgres, &context, None, false).await.expect("Failed to translate");
        serde_json::to_string(&result).expect("Failed to serialize result")
    });

    CString::new(plan_json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn aiql_ask(
    prompt: *const c_char,
    db_url: *const c_char,
    schema_json: *const c_char
) -> *mut c_char {
    let prompt_str = unsafe { CStr::from_ptr(prompt).to_str().expect("Invalid UTF-8") };
    let db_url_str = unsafe { CStr::from_ptr(db_url).to_str().expect("Invalid UTF-8") };
    let schema_json_str = unsafe { CStr::from_ptr(schema_json).to_str().expect("Invalid UTF-8") };

    let schema: Schema = serde_json::from_str(schema_json_str).expect("Failed to deserialize schema");

    let rt = Runtime::new().expect("Failed to create runtime");
    let result_json = rt.block_on(async {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(db_url_str)
            .await
            .expect("Failed to connect");

        let client = SmartClient::new(
            MockTranslator,
            PostgresExecutionEngine::new(pool),
            SimpleHealer,
            MockTranslator, // implements EmbeddingEngine
            InMemorySemanticCache::new(0.9),
            SimplePrivacyGuard::new(),
            MockTranslator, // implements Advisor
        );

        let result = client.ask(
            prompt_str,
            &schema,
            aiql_core::DatabaseDialect::Postgres,
            None,
            None,
            SafetyPolicy::ReadOnly,
            false,
            None
        ).await.expect("SmartClient failed");

        serde_json::to_string(&result).expect("Failed to serialize result")
    });

    CString::new(result_json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn aiql_generate_mock_data(
    prompt: *const c_char,
    schema_json: *const c_char
) -> *mut c_char {
    let prompt_str = unsafe { CStr::from_ptr(prompt).to_str().expect("Invalid UTF-8") };
    let schema_json_str = unsafe { CStr::from_ptr(schema_json).to_str().expect("Invalid UTF-8") };

    let schema: Schema = serde_json::from_str(schema_json_str).expect("Failed to deserialize schema");

    let rt = Runtime::new().expect("Failed to create runtime");
    let queries_json = rt.block_on(async {
        let translator = MockTranslator; // Default to mock
        let queries = aiql_core::MockDataGenerator::generate_mock_data(&translator, prompt_str, &schema, aiql_core::DatabaseDialect::Postgres).await.expect("Mock generation failed");
        serde_json::to_string(&queries).expect("Failed to serialize queries")
    });

    CString::new(queries_json).unwrap().into_raw()
}
