//! 用户画像数据模型
//!
//! 存储用户的基本信息、偏好、重要事实和工作模式

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 用户画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// 画像唯一标识
    pub id: String,

    /// 租户隔离 ID
    pub tenant_id: String,

    /// 用户 ID（与 Memory.user_id 对应）
    pub user_id: String,

    /// === 基本信息 ===
    /// 姓名
    pub name: Option<String>,

    /// 角色/职位
    pub role: Option<String>,

    /// 组织/公司
    pub organization: Option<String>,

    /// 位置
    pub location: Option<String>,

    /// === 偏好设置 ===
    /// 结构化偏好（键值对）
    pub preferences: HashMap<String, serde_json::Value>,

    /// 沟通风格偏好
    pub communication_style: Option<String>,

    /// 技术水平
    pub technical_level: Option<String>,

    /// 语言偏好
    pub language: Option<String>,

    /// === 重要事实 ===
    /// 用户告诉 Agent 的关键信息
    pub facts: Vec<ProfileFact>,

    /// 兴趣领域
    pub interests: Vec<String>,

    /// === 工作模式 ===
    /// 工作时间偏好
    pub working_hours: Option<WorkingHours>,

    /// 常见任务/项目
    pub common_tasks: Vec<String>,

    /// 常用工具
    pub tools_used: Vec<String>,

    /// === 元数据 ===
    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 画像置信度 (0.0-1.0)
    pub confidence: f32,

    /// 最后验证时间
    pub last_verified: Option<DateTime<Utc>>,

    /// 版本号（完整版本控制）
    pub version: u32,

    /// 变更历史
    pub change_history: Vec<ProfileChange>,
}

/// 用户告诉 Agent 的重要事实
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFact {
    /// 事实唯一标识
    pub id: String,

    /// 事实描述
    pub fact: String,

    /// 类别
    pub category: ProfileFactCategory,

    /// 来源记忆 ID
    pub source_memory_id: Option<String>,

    /// 置信度
    pub confidence: f32,

    /// 是否已验证
    pub verified: bool,

    /// 验证时间
    pub verified_at: Option<DateTime<Utc>>,

    /// 验证来源
    pub verified_by: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 事实类别
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfileFactCategory {
    /// 个人信息
    #[serde(rename = "personal")]
    Personal,

    /// 职业信息
    #[serde(rename = "professional")]
    Professional,

    /// 技术偏好
    #[serde(rename = "technical")]
    Technical,

    /// 项目信息
    #[serde(rename = "project")]
    Project,

    /// 沟通偏好
    #[serde(rename = "communication")]
    Communication,

    /// 生活偏好
    #[serde(rename = "lifestyle")]
    Lifestyle,

    /// 其他
    #[serde(rename = "other")]
    Other,
}

/// 工作时间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    /// 星期几开始（0=周一，7=周日）
    pub start_day: u32,

    /// 开始时间（小时，0-23）
    pub start_hour: u32,

    /// 星期几结束
    pub end_day: u32,

    /// 结束时间（小时）
    pub end_hour: u32,

    /// 时区
    pub timezone: String,

    /// 是否灵活工作时间
    pub flexible: bool,
}

/// 画像变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileChange {
    /// 变更版本号
    pub version: u32,

    /// 变更类型
    pub change_type: ProfileChangeType,

    /// 变更字段
    pub field: String,

    /// 旧值
    pub old_value: Option<serde_json::Value>,

    /// 新值
    pub new_value: Option<serde_json::Value>,

    /// 变更原因
    pub reason: Option<String>,

    /// 变更时间
    pub changed_at: DateTime<Utc>,
}

/// 变更类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfileChangeType {
    /// 创建
    #[serde(rename = "created")]
    Created,

    /// 更新
    #[serde(rename = "updated")]
    Updated,

    /// 验证
    #[serde(rename = "verified")]
    Verified,

    /// 合并
    #[serde(rename = "merged")]
    Merged,

    /// 重置
    #[serde(rename = "reset")]
    Reset,
}

/// 画像查询条件
#[derive(Debug, Clone, Default)]
pub struct ProfileQuery {
    /// 用户 ID
    pub user_id: Option<String>,

    /// 最低置信度
    pub min_confidence: Option<f32>,

    /// 事实类别筛选
    pub fact_categories: Vec<ProfileFactCategory>,

    /// 工具筛选
    pub tools: Vec<String>,

    /// 分页
    pub page: u32,
    pub page_size: u32,
}

impl Profile {
    /// 创建新画像
    pub fn new(user_id: &str) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tenant_id: "default".to_string(),
            user_id: user_id.to_string(),
            name: None,
            role: None,
            organization: None,
            location: None,
            preferences: HashMap::new(),
            communication_style: None,
            technical_level: None,
            language: Some("zh-CN".to_string()),
            facts: Vec::new(),
            interests: Vec::new(),
            working_hours: None,
            common_tasks: Vec::new(),
            tools_used: Vec::new(),
            created_at: now,
            updated_at: now,
            confidence: 0.5,
            last_verified: None,
            version: 1,
            change_history: Vec::new(),
        }
    }

    /// 更新基本信息
    pub fn update_basic_info(
        &mut self,
        name: Option<&str>,
        role: Option<&str>,
        organization: Option<&str>,
        reason: Option<&str>,
    ) {
        if let Some(name) = name {
            self.add_change(
                "name".to_string(),
                self.name.clone().map(serde_json::Value::String),
                Some(serde_json::Value::String(name.to_string())),
                reason,
            );
            self.name = Some(name.to_string());
        }
        if let Some(role) = role {
            self.add_change(
                "role".to_string(),
                self.role.clone().map(serde_json::Value::String),
                Some(serde_json::Value::String(role.to_string())),
                reason,
            );
            self.role = Some(role.to_string());
        }
        if let Some(organization) = organization {
            self.add_change(
                "organization".to_string(),
                self.organization.clone().map(serde_json::Value::String),
                Some(serde_json::Value::String(organization.to_string())),
                reason,
            );
            self.organization = Some(organization.to_string());
        }
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 添加偏好
    pub fn add_preference(&mut self, key: &str, value: serde_json::Value, reason: Option<&str>) {
        let old_value = self.preferences.get(key).cloned();
        self.preferences.insert(key.to_string(), value);
        self.add_change(
            format!("preferences.{}", key),
            old_value,
            self.preferences.get(key).cloned(),
            reason,
        );
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 添加重要事实
    pub fn add_fact(
        &mut self,
        fact: &str,
        category: ProfileFactCategory,
        source_memory_id: Option<&str>,
        confidence: f32,
    ) -> String {
        let fact_id = uuid::Uuid::new_v4().to_string();
        let new_fact = ProfileFact {
            id: fact_id.clone(),
            fact: fact.to_string(),
            category,
            source_memory_id: source_memory_id.map(|s| s.to_string()),
            confidence,
            verified: false,
            verified_at: None,
            verified_by: None,
            created_at: Utc::now(),
        };
        self.facts.push(new_fact);
        self.updated_at = Utc::now();
        fact_id
    }

    /// 验证事实
    pub fn verify_fact(&mut self, fact_id: &str, verified_by: Option<&str>) -> bool {
        if let Some(fact) = self.facts.iter_mut().find(|f| f.id == fact_id) {
            fact.verified = true;
            fact.verified_at = Some(Utc::now());
            fact.verified_by = verified_by.map(|s| s.to_string());
            self.last_verified = Some(Utc::now());
            self.updated_at = Utc::now();
            self.version += 1;
            return true;
        }
        false
    }

    /// 添加工具
    pub fn add_tool(&mut self, tool: &str) {
        let tool = tool.to_lowercase();
        if !self.tools_used.contains(&tool) {
            self.tools_used.push(tool);
            self.updated_at = Utc::now();
        }
    }

    /// 添加兴趣
    pub fn add_interest(&mut self, interest: &str) {
        let interest = interest.to_lowercase();
        if !self.interests.contains(&interest) {
            self.interests.push(interest);
            self.updated_at = Utc::now();
        }
    }

    /// 添加常用任务
    pub fn add_common_task(&mut self, task: &str) {
        if !self.common_tasks.contains(&task.to_string()) {
            self.common_tasks.push(task.to_string());
            self.updated_at = Utc::now();
        }
    }

    /// 添加变更记录
    fn add_change(
        &mut self,
        field: String,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        reason: Option<&str>,
    ) {
        let change = ProfileChange {
            version: self.version + 1,
            change_type: ProfileChangeType::Updated,
            field,
            old_value,
            new_value,
            reason: reason.map(|s| s.to_string()),
            changed_at: Utc::now(),
        };
        self.change_history.push(change);
    }

    /// 获取验证的事实数量
    pub fn verified_facts_count(&self) -> usize {
        self.facts.iter().filter(|f| f.verified).count()
    }

    /// 获取特定类别的事实
    pub fn get_facts_by_category(&self, category: &ProfileFactCategory) -> Vec<&ProfileFact> {
        self.facts
            .iter()
            .filter(|f| &f.category == category)
            .collect()
    }
}

/// 画像对比结果（用于合并）
#[derive(Debug, Clone)]
pub struct ProfileComparison {
    /// 新增的事实
    pub added_facts: Vec<ProfileFact>,

    /// 冲突的事实（相同的 key，不同的值）
    pub conflicting_facts: Vec<(ProfileFact, ProfileFact)>,

    /// 一致的值
    pub consistent_values: Vec<(String, serde_json::Value)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("user_123");
        assert_eq!(profile.user_id, "user_123");
        assert!(!profile.id.is_empty());
        assert_eq!(profile.version, 1);
        assert!(profile.facts.is_empty());
    }

    #[test]
    fn test_profile_operations() {
        let mut profile = Profile::new("user_123");

        // 更新基本信息
        profile.update_basic_info(
            Some("张三"),
            Some("软件工程师"),
            Some("科技公司"),
            Some("用户自我介绍"),
        );

        assert_eq!(profile.name, Some("张三".to_string()));
        assert_eq!(profile.role, Some("软件工程师".to_string()));

        // 添加工具
        profile.add_tool("VSCode");
        profile.add_tool("VSCode"); // 重复添加
        assert_eq!(profile.tools_used.len(), 1);

        // 添加事实
        let fact_id = profile.add_fact(
            "用户对 Rust 感兴趣",
            ProfileFactCategory::Technical,
            Some("mem_123"),
            0.8,
        );

        // 验证事实
        assert!(profile.verify_fact(&fact_id, Some("user")));
        assert_eq!(profile.verified_facts_count(), 1);

        // 获取特定类别的事实
        let tech_facts = profile.get_facts_by_category(&ProfileFactCategory::Technical);
        assert_eq!(tech_facts.len(), 1);
    }
}
