//! # ERP CLI - Enterprise Resource Planning Command Line Interface
//!
//! A high-performance, modular ERP system built with Rust following a 4-layer architecture:
//!
//! 1. **CLI Interface Layer** - Command parsing, validation, and user interaction
//! 2. **Business Logic Layer** - Core business modules (inventory, sales, customers, reports, config)
//! 3. **Core Services Layer** - Authentication, database, configuration, and logging services
//! 4. **Data Layer** - PostgreSQL (production), SQLite (development), Redis (caching)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     ERP CLI System                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  CLI Interface Layer                                       │
//! │  ┌─────────────────┬─────────────────┬─────────────────┐   │
//! │  │   Commands      │    Parser       │    Validator    │   │
//! │  └─────────────────┴─────────────────┴─────────────────┘   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Business Logic Layer                                      │
//! │  ┌──────────┬──────────┬──────────┬──────────┬──────────┐ │
//! │  │Inventory │  Sales   │Customers │ Reports  │  Config  │ │
//! │  │ Module   │  Module  │  Module  │  Module  │  Module  │ │
//! │  └──────────┴──────────┴──────────┴──────────┴──────────┘ │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Core Services Layer                                       │
//! │  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
//! │  │ Auth Service│Database Svc │Config Svc  │  Log Service│ │
//! │  └─────────────┴─────────────┴─────────────┴─────────────┘ │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Data Layer                                                │
//! │  ┌─────────────────┬─────────────────┬─────────────────┐   │
//! │  │   PostgreSQL    │     SQLite      │     Redis       │   │
//! │  │   (Production)  │   (Development) │   (Caching)     │   │
//! │  └─────────────────┴─────────────────┴─────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod app_module;
pub mod cli;
pub mod core;
pub mod modules;
pub mod utils;

// Re-export commonly used types
pub use crate::utils::error::{ErpError, ErpResult};
