//! 存储层模块
//!
//! 提供数据持久化服务，支持 ArangoDB 和 Surrealdb。

#[cfg(feature = "arangodb")]
pub mod arangodb;

#[cfg(feature = "arangodb")]
pub mod arangodb_repository;

#[cfg(feature = "surrealdb")]
pub mod surrealdb;

#[cfg(feature = "surrealdb")]
pub mod repository;

#[cfg(not(feature = "surrealdb"))]
pub mod repository;

pub mod factory;
