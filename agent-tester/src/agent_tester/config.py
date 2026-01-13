"""Configuration management for agent-tester."""

from typing import Optional
from pathlib import Path

from pydantic import Field, field_validator
from pydantic_settings import BaseSettings


class MCPServerConfig(BaseSettings):
    """MCP Server configuration."""

    name: str = "unnamed"
    command: str = ""
    args: list[str] = Field(default_factory=list)
    env: dict[str, str] = Field(default_factory=dict)
    url: Optional[str] = None

    @field_validator("command")
    @classmethod
    def validate_command(cls, v: str) -> str:
        if v and not Path(v).exists():
            raise ValueError(f"Command not found: {v}")
        return v


class LLMConfig(BaseSettings):
    """LLM provider configuration."""

    provider: str = "openai"  # openai, anthropic
    api_key: str = ""
    model: str = "gpt-4o"
    base_url: Optional[str] = None
    max_tokens: int = 4096
    temperature: float = 0.7

    @property
    def is_valid(self) -> bool:
        return bool(self.api_key)


class Settings(BaseSettings):
    """Application settings."""

    hippos_url: str = "http://localhost:8080"
    hippos_api_key: str = "dev-api-key"

    default_llm: str = "openai"
    llms: dict[str, LLMConfig] = Field(default_factory=dict)

    mcp_servers: list[MCPServerConfig] = Field(default_factory=list)

    system_prompt_template: str = """You are a helpful AI assistant.
Current session ID: {session_id}
Context from previous conversations:
{context}

Please respond to the user's message based on the context above.
If you need to search for information, use the available tools."""

    context_max_length: int = 2000
    max_history_turns: int = 10

    @field_validator("llms")
    @classmethod
    def validate_llms(cls, v: dict[str, LLMConfig]) -> dict[str, LLMConfig]:
        if not v:
            v = {
                "openai": LLMConfig(provider="openai"),
                "anthropic": LLMConfig(provider="anthropic"),
            }
        return v

    def get_llm(self, name: Optional[str] = None) -> LLMConfig:
        name = name or self.default_llm
        if name in self.llms:
            return self.llms[name]
        raise ValueError(f"LLM config not found: {name}")

    @classmethod
    def from_yaml(cls, path: str) -> "Settings":
        """Load settings from YAML file."""
        import yaml

        with open(path, "r") as f:
            data = yaml.safe_load(f)
        return cls(**data)
