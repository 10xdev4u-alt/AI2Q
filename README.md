# AIQL (Artificial Intelligence Query Layer)

**"Don't Query. Just Ask."**

AIQL is a universal, AI-powered query layer designed to bring the simplicity of natural language to any database, in any language. Inspired by the simplicity of Excel's `AI()` function, AIQL provides a high-performance, safe, and "self-healing" bridge between human intent and structured data.

## 🚀 Vision

To democratize database access by providing a universal "Intelligence Driver" that works across Go, Python, C++, and more, supporting SQL and NoSQL databases like PostgreSQL, MySQL, MongoDB, and Supabase.

## ✨ Key Features (25+ Planned)

### I. Smart Execution & Reliability
- **Dry-Run Validation:** Automatically runs `EXPLAIN` to ensure query safety.
- **Self-Healing Loops:** Invisibly retries queries on schema errors.
- **Ambiguity Guard:** Asks for clarification when prompts are vague.
- **Schema-Aware Pruning:** Minimizes token usage for large schemas.
- **Type-Safe Generation:** Generates native language structs/classes for query results.

### II. Developer Experience
- **Cross-DB Translation:** One prompt, any syntax (SQL or MQL).
- **Natural Language Migrations:** `ai.migrate("add a field for 'social_security' but mask it by default")`.
- **Semantic Caching:** Identifies and caches semantically identical queries.
- **Privacy-First Masking:** Automatically detects and handles PII.
- **Explainable SQL:** Includes comments explaining the logic behind generated queries.

### III. Performance & Security
- **Vector-Integrated Queries:** Joins relational data with vector embeddings.
- **Streaming NL-Queries:** Cursor-based streaming for massive datasets.
- **Local-First Inference:** Supports local LLMs (via Llama.cpp) for privacy.
- **Read-Only Enforcement:** Safe mode to prevent destructive commands.
- **Performance Budgeting:** Limits execution time and cost per query.

### IV. Advanced "Next-Gen" Flows
- **Chat-with-DB:** Multi-turn session state for follow-up questions.
- **Synthetic Data Mocking:** Generate realistic test data with simple prompts.
- **Automatic Index Suggestions:** Recommends performance improvements.
- **Temporal Queries:** Natural language "time-travel" (e.g., "churn rate last Christmas").
- **Join-Discovery:** Automatically finds relationships without explicit Foreign Keys.

## 🏗 Architecture

- **Core (`aiql-core`):** High-performance Rust engine.
- **FFI (`aiql-ffi`):** C-compatible bridge for language bindings.
- **Bindings:** Idiomatic wrappers for Go, Python, and C++.
- **CLI:** Terminal app for interactive database conversations.
- **Web Dashboard:** Grit-inspired UI for schema management and monitoring.

## 🛡 License

Licensed under both [MIT](LICENSE-MIT) and [Apache License, Version 2.0](LICENSE-APACHE).

## 🤝 Contributing

We follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) and welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
