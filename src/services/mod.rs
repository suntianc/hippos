//! 服务模块

pub mod dehydration;
pub mod memory_builder;
pub mod memory_integrator;
pub mod memory_recall;
pub mod pattern_manager;
pub mod performance;
pub mod retrieval;
pub mod session;
pub mod turn;

pub use dehydration::{DehydrationService, create_dehydration_service};
pub use memory_builder::{MemoryBuilder, create_memory_builder};
pub use memory_recall::{MemoryRecall, MemoryRecallService, create_memory_recall_service, SearchOptions, SearchResultItem, TimeRange, RrfWeights};
pub use pattern_manager::{
    PatternManager, PatternRecommendation, PatternUpdates, PatternDiscoveryResult,
    DiscoveryMethod, PatternSuggestion, OutcomeRecord, PatternCreateRequest,
    PatternGenerator, create_pattern_manager, create_pattern_manager_basic,
};
pub use retrieval::{RetrievalService, create_retrieval_service};
pub use session::{Pagination, SessionQuery, SessionService, create_session_service};
pub use turn::{BatchCreateResult, TurnGroup, TurnQuery, TurnService, create_turn_service};
