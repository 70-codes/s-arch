//! # Migration Generation
//!
//! This module generates SQL migration files from the project's entity
//! definitions. Migrations are ordered by dependency (referenced tables
//! first) and support PostgreSQL, MySQL, and SQLite.
//!
//! ## Generated Files
//!
//! Each entity produces a migration file named:
//! ```text
//! migrations/{date}{index}_create_{table_name}.sql
//! ```
//!
//! ## Features
//!
//! - `CREATE TABLE` with correct column types per database
//! - Primary key definitions (UUID, SERIAL, CUID)
//! - Foreign key constraints with referential actions
//! - Unique constraints and indexes
//! - Default values (including `CURRENT_TIMESTAMP`, `gen_random_uuid()`)
//! - Timestamp columns (`created_at`, `updated_at`)
//! - Soft-delete support (`deleted_at`)
//! - `IF NOT EXISTS` for idempotent migrations

pub mod sql;

pub use sql::generate_migrations;
