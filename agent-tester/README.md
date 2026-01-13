# Agent Tester

A smart agent application to test hippos functionality with multi-round dialogue and dynamic system prompt injection.

## Features

- **Multi-round Dialogue**: Store and retrieve conversation history in Hippos
- **Dynamic System Prompt**: Inject session-id and context on each request
- **MCP Integration**: Connect to multiple MCP servers (Hippos + external)
- **Multiple LLM Providers**: Support for OpenAI and Anthropic

## Quick Start

### Prerequisites

- Python 3.10+
- Hippos running (`./target/release/hippos`)
- OpenAI or Anthropic API key

### Installation

```bash
cd agent-tester
pip install -e .
```

### Configuration

Create a `config.yaml` file:

```yaml
hippos_url: "http://localhost:8080"
hippos_api_key: "dev-api-key"

default_llm: openai
llms:
  openai:
    provider: openai
    api_key: "sk-..."
    model: "gpt-4o"
  anthropic:
    provider: anthropic
    api_key: "sk-ant-..."
    model: "claude-3-5-sonnet-20241022"

system_prompt_template: |
  You are a helpful AI assistant.
  Current session ID: {session_id}
  Context from previous conversations:
  {context}

  Please respond to the user's message based on the context above.
```

### Usage

#### Interactive Chat

```bash
agent-chat --config config.yaml
```

Commands:
- `/session` - Show current session ID
- `/search <query>` - Search context
- `/history` - Show conversation history
- `/quit` - Exit

#### CLI Commands

```bash
# Check Hippos health
agent-tester health

# List sessions
agent-tester sessions

# List turns for a session
agent-tester turns <session_id>

# Search within a session
agent-tester search <session_id> <query>
```

### Programmatic Usage

```python
import asyncio
from agent_tester import SessionOrchestrator, Settings

async def main():
    settings = Settings(
        hippos_url="http://localhost:8080",
        hippos_api_key="dev-api-key",
    )
    
    orchestrator = SessionOrchestrator(settings, hippos)
    
    # Start chat
    response = await orchestrator.chat(
        "Hello! I'm testing hippos.",
        session_id=None,  # Auto-create session
    )
    
    print(response)
    
    await orchestrator.close()

asyncio.run(main())
```

## Architecture

```
┌───────────────────────────────┐
│      Agent Tester             │
├───────────────────────────────┤
│  SessionOrchestrator          │
│  - system prompt render       │
│  - retrieve context (Hippos)  │
│  - tool routing (MCP)         │
└───────┬───────────┬───────────┘
        │           │
        ▼           ▼
┌─────────────┐   ┌───────────────────────────┐
│ Hippos REST │   │ MCP Client Aggregator     │
│ (sessions)  │   │ - Hippos MCP Server       │
│ (turns)     │   │ - Other MCP Servers       │
└──────┬──────┘   └───────────────────────────┘
       │
       ▼
┌───────────────────────────────┐
│  Hippos Storage + Retrieval   │
│  - SurrealDB + Index           │
└───────────────────────────────┘
```

## License

MIT
