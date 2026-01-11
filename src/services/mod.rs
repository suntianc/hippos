//! 服务模块

pub mod dehydration;
pub mod retrieval;

pub use dehydration::{DehydrationService, create_dehydration_service};
pub use retrieval::{RetrievalService, create_retrieval_service};
