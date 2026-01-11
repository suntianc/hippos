//! Hippos - 高性能上下文管理服务
//!
//! 为 AI Agent 提供持久化的对话记忆能力，解决大语言模型在长对话场景中
//! 面临的上下文窗口限制问题。

pub mod api;
pub mod config;
pub mod error;
pub mod index;
pub mod mcp;
pub mod models;
pub mod observability;
pub mod security;
pub mod services;
pub mod storage;
