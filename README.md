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

## 🗺 Roadmap (200+ Commits Milestone)

We are committed to building the most robust AI Query Layer. Our journey to 200+ commits includes:

- [x] **Phase 1: Foundation (Commits 1-50)**
    - [x] Monorepo scaffolding with Grit & Rust.
    - [x] Postgres Schema Crawler (PK/FK/Index detection).
    - [x] C-FFI, Go, and Python bindings.
    - [x] CLI basic interface.
    - [x] Unit testing suite for core crawlers.
- [x] **Phase 2: Intelligence Layer (Commits 51-100)**
    - [x] `aiql-translator`: OpenAI & Ollama (Local-first) integration.
    - [x] `aiql-healer`: Automated error correction loop.
    - [x] Dry-run validation using `EXPLAIN` and `explain()`.
    - [x] Context pruning for massive schemas.
    - [x] MongoDB (MQL) Crawler and Execution Engine.
- [x] **Phase 3: Advanced Flows & Production (Commits 101-150)**
    - [x] Natural Language Migrations (`ai.migrate`).
    - [x] Vector-Integrated Queries (pgvector).
    - [x] Multi-turn "Chat-with-DB" session management.
    - [x] Semantic Caching with cosine similarity.
    - [x] Privacy-First Masking (PII scrubbing).
    - [x] Performance Budgeting (Cost/Time limits).
    - [x] Ambiguity Guard (Clarification requests).
    - [x] Automatic Join-Discovery (Name-based).
    - [x] Synthetic Data Mocking (`ai.mock`).
    - [x] Temporal Queries (Current time awareness).
- [ ] **Phase 4: Web Dashboard & Final Polish (Commits 151-200+)**
    - [ ] Grit Web Dashboard (Interactive Schema Map).
    - [ ] Performance budgeting and rate limiting.
    - [ ] Privacy masking for PII.
    - [ ] Comprehensive documentation and tutorials.

## 🛠 Features in Depth

### I. Self-Healing Loops
When a generated query fails (e.g., due to a typo in a column name or a hallucinated join), AIQL doesn't just crash. It catches the error, sends it back to the LLM with the error message and the relevant schema portion, and generates a corrected query. This happens in under 100ms.

### II. Dry-Run Validation
Every query is run through `EXPLAIN` before execution. If the query is dangerously slow (full table scan on a large table) or invalid, AIQL blocks it and suggests an optimized version.

### III. Cross-DB Translation
AIQL abstracts the dialect. `ai.ask("Get newest users")` returns MQL for MongoDB and SQL for Postgres, making it the perfect driver for multi-database architectures.
