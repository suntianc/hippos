# Hippos Examples and Tutorials

**Practical examples for using the Hippos Memory Service**

## Table of Contents

1. [Quick Start](#quick-start)
2. [API Examples](#api-examples)
3. [WebSocket Examples](#websocket-examples)
4. [Use Cases](#use-cases)
5. [Integration Examples](#integration-examples)

---

## Quick Start

### Prerequisites

```bash
# Start SurrealDB
docker run -d --name surrealdb \
  -p 8000:8000 \
  surrealdb/surrealdb:2.0.0 \
  start --bind=0.0.0.0:8000 --user=root --pass=root

# Start Hippos
cargo run --release
```

### Your First Memory

```bash
# Create your first memory
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User asked about implementing async Rust patterns with tokio",
    "memory_type": "episodic",
    "source": "conversation",
    "importance": 0.8,
    "tags": ["rust", "async", "tokio"]
  }'

# Response:
# {
#   "id": "mem_abc123",
#   "message": "Memory created successfully"
# }
```

---

## API Examples

### Memory Management

#### Create a Memory

```bash
# Create an episodic memory (conversation)
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Discussed Rust error handling patterns. Key points: use Result for recoverable errors, panic for unrecoverable states, and custom error types for domain-specific errors.",
    "memory_type": "episodic",
    "source": "conversation",
    "importance": 0.75,
    "topics": ["rust", "error-handling"],
    "tags": ["programming", "rust", "best-practices"]
  }'
```

#### List All Memories

```bash
# List memories with pagination
curl "http://localhost:8080/api/v1/memories?page=1&page_size=10" \
  -H "Authorization: ApiKey dev-api-key"
```

#### Search Memories

```bash
# Semantic search
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Rust async programming patterns",
    "limit": 5,
    "strategy": "semantic"
  }'

# Hybrid search (semantic + keyword)
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "error handling in Rust",
    "limit": 10,
    "strategy": "hybrid"
  }'
```

#### Update a Memory

```bash
# Update memory importance
curl -X PUT http://localhost:8080/api/v1/memories/mem_abc123 \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "importance": 0.9,
    "tags": ["rust", "error-handling", "important"]
  }'
```

#### Delete a Memory

```bash
# Soft delete (archive)
curl -X DELETE http://localhost:8080/api/v1/memories/mem_abc123 \
  -H "Authorization: ApiKey dev-api-key"

# Response:
# {
#   "id": "mem_abc123",
#   "message": "Memory archived successfully"
# }
```

### Profile Management

#### Create a User Profile

```bash
# Create a profile
curl -X POST http://localhost:8080/api/v1/profiles \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "name": "John Doe",
    "role": "Senior Developer",
    "preferences": {
      "communication_style": "concise",
      "technical_level": "advanced",
      "preferred_language": "English"
    }
  }'
```

#### Add Facts to Profile

```bash
# Add a verified fact
curl -X POST http://localhost:8080/api/v1/profiles/user_123/facts \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "fact": "John has 10 years of programming experience",
    "category": "experience",
    "confidence": 0.9
  }'
```

#### Add Preferences

```bash
# Add preference
curl -X POST http://localhost:8080/api/v1/profiles/user_123/preferences \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "prefers_detailed_explanations": true,
    "likes_code_examples": true
  }'
```

### Pattern Management

#### Create a Pattern

```bash
# Create a problem-solution pattern
curl -X POST http://localhost:8080/api/v1/patterns \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Rust Error Handling Pattern",
    "description": "Best practices for error handling in Rust",
    "pattern_type": "best_practice",
    "trigger": "error handling, rust, result",
    "context": "When discussing error handling in Rust",
    "problem": "Developers often misuse panic for recoverable errors",
    "solution": "Use Result<T, E> for recoverable errors. Define custom error types that implement std::error::Error. Use thiserror for ergonomic error types.",
    "tags": ["rust", "error-handling", "best-practice"]
  }'
```

#### Match Patterns

```bash
# Find relevant patterns
curl -X POST http://localhost:8080/api/v1/patterns/match \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "context": "I need to handle errors in my Rust code",
    "limit": 3
  }'
```

#### Record Pattern Usage

```bash
# Record pattern outcome
curl -X POST http://localhost:8080/api/v1/patterns/pat_abc123/record-outcome \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "outcome": 0.9,
    "notes": "Pattern was helpful and improved error handling"
  }'
```

### Entity Management

#### Create an Entity

```bash
# Create a tool entity
curl -X POST http://localhost:8080/api/v1/entities \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tokio",
    "entity_type": "tool",
    "description": "Runtime for writing reliable asynchronous applications with Rust",
    "properties": {
      "version": "1.35",
      "license": "MIT"
    },
    "aliases": ["tokio-rs"]
  }'
```

#### Create a Relationship

```bash
# Create relationship between entities
curl -X POST http://localhost:8080/api/v1/relationships \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "source_entity_id": "ent_rust",
    "target_entity_id": "ent_tokio",
    "relationship_type": "uses",
    "strength": 0.95,
    "context": "Rust programs commonly use tokio for async runtime"
  }'
```

#### Query Knowledge Graph

```bash
# Query entity relationships
curl -X POST http://localhost:8080/api/v1/entities/ent_tokio/relationships \
  -H "Authorization: ApiKey dev-api-key"

# Query graph traversal
curl -X POST http://localhost:8080/api/v1/entities/graph \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "entity_id": "ent_rust",
    "depth": 2,
    "relationship_types": ["uses", "depends_on"]
  }'
```

---

## WebSocket Examples

### Connect and Subscribe

```javascript
// JavaScript WebSocket client example
const ws = new WebSocket('ws://localhost:8080/ws?token=dev-api-key');

ws.onopen = () => {
  console.log('Connected to Hippos');
  
  // Subscribe to memory updates
  ws.send(JSON.stringify({
    action: 'subscribe',
    topics: ['memory:*', 'profile:updated']
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
  
  if (message.type === 'memory:created') {
    console.log('New memory created:', message.data);
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected');
};
```

### Python WebSocket Client

```python
import asyncio
import json

try:
    import websockets
except ImportError:
    print("Install: pip install websockets")
    exit(1)

async def hippos_client():
    uri = "ws://localhost:8080/ws?token=dev-api-key"
    async with websockets.connect(uri) as ws:
        # Subscribe
        await ws.send(json.dumps({
            "action": "subscribe",
            "topics": ["memory:*"]
        }))
        
        # Listen for messages
        async for message in ws:
            data = json.loads(message)
            print(f"Received: {data}")

if __name__ == "__main__":
    asyncio.run(hippos_client())
```

---

## Use Cases

### Use Case 1: Conversation Memory System

```bash
# 1. Store conversation
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User asked about Rust async/await syntax. Explained that async functions return Future and use .await to execute. Demonstrated with a simple example.",
    "memory_type": "episodic",
    "source": "conversation",
    "importance": 0.7,
    "topics": ["rust", "async", "tutoring"]
  }'

# 2. Later, search for similar conversations
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Rust async programming teaching",
    "limit": 5
  }'
```

### Use Case 2: User Preference Learning

```bash
# 1. Create user profile
curl -X POST http://localhost:8080/api/v1/profiles \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_learner",
    "name": "Alex",
    "role": "Beginner Developer"
  }'

# 2. Add observations as facts
curl -X POST http://localhost:8080/api/v1/profiles/user_learner/facts \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "fact": "Alex prefers detailed explanations with examples",
    "category": "learning_style",
    "confidence": 0.8
  }'

# 3. Query profile for personalized responses
curl http://localhost:8080/api/v1/profiles/user_learner \
  -H "Authorization: ApiKey dev-api-key"
```

### Use Case 3: Best Practice Knowledge Base

```bash
# 1. Store a best practice pattern
curl -X POST http://localhost:8080/api/v1/patterns \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Git Commit Message Format",
    "description": "Conventional commit messages for better changelogs",
    "pattern_type": "best_practice",
    "trigger": "git, commit, message, conventional",
    "context": "When discussing version control and commit practices",
    "problem": "Inconsistent commit messages make changelog generation difficult",
    "solution": "Use conventional commits: <type>(<scope>): <description>\n\nTypes: feat, fix, docs, style, refactor, test, chore\n\nExample: feat(auth): add OAuth2 login support",
    "examples": [
      {"description": "New feature", "code": "feat(user): add profile picture upload"},
      {"description": "Bug fix", "code": "fix(api): resolve timeout issue"}
    ],
    "tags": ["git", "best-practice", "workflow"]
  }'

# 2. Find patterns for a context
curl -X POST http://localhost:8080/api/v1/patterns/match \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "context": "Our team needs better commit message conventions",
    "limit": 3
  }'
```

### Use Case 4: Technology Knowledge Graph

```bash
# 1. Create entities for technologies
curl -X POST http://localhost:8080/api/v1/entities \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Rust",
    "entity_type": "language",
    "description": "Systems programming language emphasizing safety and performance"
  }'

curl -X POST http://localhost:8080/api/v1/entities \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Actix-web",
    "entity_type": "framework",
    "description": "Powerful web framework for Rust"
  }'

# 2. Create relationships
curl -X POST http://localhost:8080/api/v1/relationships \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "source_entity_id": "ent_actix",
    "target_entity_id": "ent_rust",
    "relationship_type": "depends_on",
    "strength": 1.0,
    "context": "Actix-web is written in Rust"
  }'

# 3. Query the knowledge graph
curl -X POST http://localhost:8080/api/v1/entities/graph \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "entity_id": "ent_actix",
    "depth": 3,
    "relationship_types": ["depends_on", "uses"]
  }'
```

---

## Integration Examples

### Python Integration

```python
import requests

class HipposClient:
    def __init__(self, base_url="http://localhost:8080", api_key="dev-api-key"):
        self.base_url = base_url
        self.headers = {"Authorization": f"ApiKey {api_key}"}
    
    def create_memory(self, content, memory_type="episodic", importance=0.5):
        return requests.post(
            f"{self.base_url}/api/v1/memories",
            headers=self.headers,
            json={
                "content": content,
                "memory_type": memory_type,
                "importance": importance
            }
        ).json()
    
    def search_memories(self, query, limit=10):
        return requests.post(
            f"{self.base_url}/api/v1/memories/search",
            headers=self.headers,
            json={"query": query, "limit": limit}
        ).json()
    
    def create_profile(self, user_id, name=None):
        return requests.post(
            f"{self.base_url}/api/v1/profiles",
            headers=self.headers,
            json={"user_id": user_id, "name": name}
        ).json()
    
    def create_pattern(self, name, trigger, solution, pattern_type="problem_solution"):
        return requests.post(
            f"{self.base_url}/api/v1/patterns",
            headers=self.headers,
            json={
                "name": name,
                "trigger": trigger,
                "solution": solution,
                "pattern_type": pattern_type
            }
        ).json()

# Usage
client = HipposClient()
client.create_memory("Learned about Rust ownership today", importance=0.8)
results = client.search_memories("Rust ownership system")
```

### Node.js Integration

```javascript
class HipposClient {
  constructor(baseUrl = 'http://localhost:8080', apiKey = 'dev-api-key') {
    this.baseUrl = baseUrl;
    this.headers = { 'Authorization': `ApiKey ${apiKey}` };
  }

  async createMemory(content, options = {}) {
    const response = await fetch(`${this.baseUrl}/api/v1/memories`, {
      method: 'POST',
      headers: { ...this.headers, 'Content-Type': 'application/json' },
      body: JSON.stringify({ content, ...options })
    });
    return response.json();
  }

  async searchMemories(query, limit = 10) {
    const response = await fetch(`${this.baseUrl}/api/v1/memories/search`, {
      method: 'POST',
      headers: { ...this.headers, 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, limit })
    });
    return response.json();
  }

  async createPattern(name, trigger, solution, type = 'problem_solution') {
    const response = await fetch(`${this.baseUrl}/api/v1/patterns`, {
      method: 'POST',
      headers: { ...this.headers, 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, trigger, solution, pattern_type: type })
    });
    return response.json();
  }
}

// Usage
const client = new HipposClient();
await client.createMemory('Started learning Rust', { importance: 0.7 });
const results = await client.searchMemories('Rust learning');
```

### Rust Integration (Using Reqwest)

```rust
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct CreateMemoryRequest {
    content: String,
    memory_type: String,
    importance: f32,
}

#[derive(Deserialize)]
struct MemoryResponse {
    id: String,
    message: String,
}

struct HipposClient {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl HipposClient {
    fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn create_memory(&self, content: &str, memory_type: &str) -> Result<MemoryResponse, reqwest::Error> {
        let request = CreateMemoryRequest {
            content: content.to_string(),
            memory_type: memory_type.to_string(),
            importance: 0.5,
        };

        self.client
            .post(&format!("{}/api/v1/memories", self.base_url))
            .header("Authorization", format!("ApiKey {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json()
            .await
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = HipposClient::new("http://localhost:8080", "dev-api-key");
    let memory = client.create_memory("Test memory", "episodic").await?;
    println!("Created: {}", memory.id);
    Ok(())
}
```

---

## Best Practices

### 1. Memory Importance Scoring

```bash
# High importance for critical information
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User's password is stored securely using bcrypt",
    "memory_type": "semantic",
    "importance": 0.95,  # Critical info
    "tags": ["security", "important"]
  }'

# Lower importance for casual notes
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User mentioned liking coffee",
    "memory_type": "episodic",
    "importance": 0.3,  # Casual info
    "tags": ["personal"]
  }'
```

### 2. Effective Pattern Triggers

```bash
# Use comma-separated keywords for better matching
curl -X POST http://localhost:8080/api/v1/patterns \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Performance Pattern",
    "trigger": "performance, optimization, speed, slow, bottleneck, profiling",  # Multiple trigger terms
    "solution": "Profile first, then optimize. Use benchmarks to measure improvement.",
    "pattern_type": "best_practice"
  }'
```

### 3. Regular Maintenance

```bash
# Check memory statistics
curl http://localhost:8080/api/v1/memories/stats \
  -H "Authorization: ApiKey dev-api-key"

# Archive old low-importance memories periodically
# (Implement via MemoryIntegrator background task)
```

---

## Next Steps

- **[API Documentation](./API.md)** - Full API reference
- **[Architecture](./ARCHITECTURE.md)** - System architecture
- **[Deployment Guide](./DEPLOYMENT.md)** - Production deployment
- **[User Manual](./USER_MANUAL.md)** - Comprehensive user guide

---

**Happy Memory Building! 🦛**
