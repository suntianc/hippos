//! 问题模式数据模型
//!
//! 存储 Agent 学到的问题解决方案、技能、最佳实践

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 模式类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternType {
    /// 问题-解决方案模式
    #[serde(rename = "problem_solution")]
    ProblemSolution,

    /// 工作流程模式
    #[serde(rename = "workflow")]
    Workflow,

    /// 最佳实践
    #[serde(rename = "best_practice")]
    BestPractice,

    /// 常见错误模式
    #[serde(rename = "common_error")]
    CommonError,

    /// 技能模式
    #[serde(rename = "skill")]
    Skill,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::ProblemSolution => write!(f, "problem_solution"),
            PatternType::Workflow => write!(f, "workflow"),
            PatternType::BestPractice => write!(f, "best_practice"),
            PatternType::CommonError => write!(f, "common_error"),
            PatternType::Skill => write!(f, "skill"),
        }
    }
}

/// 问题模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// 模式唯一标识
    pub id: String,

    /// 租户隔离 ID
    pub tenant_id: String,

    /// 模式类型
    pub pattern_type: PatternType,

    /// === 模式定义 ===
    /// 模式名称
    pub name: String,

    /// 模式描述
    pub description: String,

    /// 触发条件（正则或关键词）
    pub trigger: String,

    /// 适用场景描述
    pub context: String,

    /// === 模式内容 ===
    /// 问题描述
    pub problem: String,

    /// 解决方案
    pub solution: String,

    /// 详细解释
    pub explanation: Option<String>,

    /// 示例列表
    pub examples: Vec<PatternExample>,

    /// === 效果追踪 ===
    /// 成功使用次数
    pub success_count: u32,

    /// 失败使用次数
    pub failure_count: u32,

    /// 平均结果评分 (-1.0 到 1.0)
    pub avg_outcome: f32,

    /// 最后使用时间
    pub last_used: Option<DateTime<Utc>>,

    /// === 元数据 ===
    /// 标签
    pub tags: Vec<String>,

    /// 创建者 ID
    pub created_by: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 使用次数
    pub usage_count: u32,

    /// 是否公开（所有用户可见）
    pub is_public: bool,

    /// 置信度
    pub confidence: f32,

    /// 版本号
    pub version: u32,

    /// 祖先模式 ID（用于模式继承）
    pub parent_pattern_id: Option<String>,
}

/// 模式示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExample {
    /// 示例 ID
    pub id: String,

    /// 输入/问题
    pub input: String,

    /// 输出/解决方案
    pub output: String,

    /// 结果评分 (-1.0 到 1.0)
    pub outcome: f32,

    /// 来源记忆 ID
    pub source_memory_id: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 模式使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUsage {
    /// 记录 ID
    pub id: String,

    /// 模式 ID
    pub pattern_id: String,

    /// 使用者 ID
    pub user_id: String,

    /// 使用时的输入
    pub input: String,

    /// 使用的输出
    pub output: String,

    /// 结果评分 (-1.0 到 1.0)
    pub outcome: f32,

    /// 用户反馈
    pub feedback: Option<String>,

    /// 使用时间
    pub used_at: DateTime<Utc>,

    /// 上下文
    pub context: Option<String>,
}

/// 模式查询条件
#[derive(Debug, Clone, Default)]
pub struct PatternQuery {
    /// 模式类型筛选
    pub types: Vec<PatternType>,

    /// 标签筛选
    pub tags: Vec<String>,

    /// 最小置信度
    pub min_confidence: f32,

    /// 最小成功率
    pub min_success_rate: Option<f32>,

    /// 关键词搜索
    pub keyword: Option<String>,

    /// 创建者筛选
    pub created_by: Option<String>,

    /// 是否只返回公开模式
    pub public_only: bool,

    /// 分页
    pub page: u32,
    pub page_size: u32,
}

impl Pattern {
    /// 创建新模式
    pub fn new(
        created_by: &str,
        pattern_type: PatternType,
        name: &str,
        problem: &str,
        solution: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("-{}", uuid::Uuid::new_v4().to_string()),
            tenant_id: "default".to_string(),
            pattern_type,
            name: name.to_string(),
            description: String::new(),
            trigger: String::new(),
            context: String::new(),
            problem: problem.to_string(),
            solution: solution.to_string(),
            explanation: None,
            examples: Vec::new(),
            success_count: 0,
            failure_count: 0,
            avg_outcome: 0.0,
            last_used: None,
            tags: Vec::new(),
            created_by: created_by.to_string(),
            created_at: now,
            updated_at: now,
            usage_count: 0,
            is_public: false,
            confidence: 0.5,
            version: 1,
            parent_pattern_id: None,
        }
    }

    /// 更新模式内容
    pub fn update_content(
        &mut self,
        name: Option<&str>,
        description: Option<&str>,
        trigger: Option<&str>,
        context: Option<&str>,
        problem: Option<&str>,
        solution: Option<&str>,
        explanation: Option<&str>,
    ) {
        if let Some(name) = name {
            self.name = name.to_string();
        }
        if let Some(description) = description {
            self.description = description.to_string();
        }
        if let Some(trigger) = trigger {
            self.trigger = trigger.to_string();
        }
        if let Some(context) = context {
            self.context = context.to_string();
        }
        if let Some(problem) = problem {
            self.problem = problem.to_string();
        }
        if let Some(solution) = solution {
            self.solution = solution.to_string();
        }
        if let Some(explanation) = explanation {
            self.explanation = Some(explanation.to_string());
        }
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 添加示例
    pub fn add_example(
        &mut self,
        input: &str,
        output: &str,
        outcome: f32,
        source_memory_id: Option<&str>,
    ) -> String {
        let example_id = uuid::Uuid::new_v4().to_string();
        self.examples.push(PatternExample {
            id: example_id.clone(),
            input: input.to_string(),
            output: output.to_string(),
            outcome,
            source_memory_id: source_memory_id.map(|s| s.to_string()),
            created_at: Utc::now(),
        });
        self.updated_at = Utc::now();
        example_id
    }

    /// 记录使用
    pub fn record_usage(
        &mut self,
        _user_id: &str,
        input: &str,
        output: &str,
        outcome: f32,
        _feedback: Option<&str>,
        context: Option<&str>,
    ) -> String {
        let usage_id = uuid::Uuid::new_v4().to_string();

        // 更新统计
        self.usage_count += 1;
        self.last_used = Some(Utc::now());

        if outcome >= 0.0 {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        // 更新平均结果
        let total = self.success_count + self.failure_count;
        self.avg_outcome = ((self.avg_outcome * (total - 1) as f32) + outcome) / total as f32;

        self.updated_at = Utc::now();

        usage_id
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: &str) {
        let tag = tag.to_lowercase();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// 计算成功率
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.5 // 默认 50%
        } else {
            self.success_count as f32 / total as f32
        }
    }

    /// 是否为高质量模式
    pub fn is_high_quality(&self) -> bool {
        self.confidence >= 0.7 && self.success_rate() >= 0.7
    }

    /// 模式匹配度（检查触发条件）
    pub fn matches_trigger(&self, input: &str) -> bool {
        // 简单的关键词匹配
        // 实际实现可以使用更复杂的匹配算法
        let trigger_lower = self.trigger.to_lowercase();
        let input_lower = input.to_lowercase();

        // 检查每个关键词
        trigger_lower
            .split(',')
            .map(|s| s.trim())
            .all(|keyword| input_lower.contains(keyword))
    }
}

/// 模式推荐结果
#[derive(Debug, Clone)]
pub struct PatternRecommendation {
    /// 模式
    pub pattern: Pattern,

    /// 推荐分数
    pub score: f32,

    /// 推荐原因
    pub reasons: Vec<String>,
}

/// 模式统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    /// 总模式数
    pub total_count: u64,

    /// 按类型统计
    pub problem_solution_count: u64,
    pub workflow_count: u64,
    pub best_practice_count: u64,
    pub common_error_count: u64,
    pub skill_count: u64,

    /// 效果统计
    pub avg_success_rate: f32,
    pub high_quality_count: u64,

    /// 使用统计
    pub total_usages: u64,
    pub most_used_pattern_id: String,
    pub most_used_pattern_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let pattern = Pattern::new(
            "user_123",
            PatternType::ProblemSolution,
            "Rust 异步编程错误",
            " tokio::spawn 导致任务丢失",
            "使用 tokio::spawn 时要确保正确处理 JoinError",
        );

        assert_eq!(pattern.pattern_type, PatternType::ProblemSolution);
        assert_eq!(pattern.created_by, "user_123");
        assert!(pattern.id.starts_with('-'));
    }

    #[test]
    fn test_pattern_operations() {
        let mut pattern = Pattern::new("user_123", PatternType::Skill, "测试模式", "输入", "输出");

        // 更新内容
        pattern.update_content(
            Some("更新的名称"),
            Some("更新的描述"),
            Some("rust"),
            Some("Rust 异步编程场景"),
            Some("更新的问题"),
            Some("更新的解决方案"),
            Some("详细解释"),
        );

        assert_eq!(pattern.name, "更新的名称");
        assert!(pattern.trigger.contains("rust"));

        // 添加示例
        let example_id = pattern.add_example("输入1", "输出1", 0.8, None);
        assert!(!example_id.is_empty());

        // 记录使用
        let usage_id = pattern.record_usage("user_456", "输入", "输出", 0.9, Some("很好"), None);
        assert!(!usage_id.is_empty());

        // 检查统计
        assert_eq!(pattern.usage_count, 1);
        assert_eq!(pattern.success_count, 1);
        assert_eq!(pattern.failure_count, 0);

        // 检查成功率
        assert_eq!(pattern.success_rate(), 1.0);

        // 检查触发匹配
        assert!(pattern.matches_trigger("学习 rust 异步编程"));
        assert!(!pattern.matches_trigger("学习 python"));
    }

    #[test]
    fn test_high_quality_pattern() {
        let mut pattern = Pattern::new(
            "user_123",
            PatternType::BestPractice,
            "高质量模式",
            "问题",
            "解决方案",
        );

        // 高置信度 + 高成功率
        pattern.confidence = 0.8;
        pattern.record_usage("u1", "i", "o", 0.8, None, None);
        pattern.record_usage("u2", "i", "o", 0.9, None, None);

        assert!(pattern.is_high_quality());

        // 低置信度
        pattern.confidence = 0.5;
        assert!(!pattern.is_high_quality());
    }
}
