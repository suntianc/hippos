"""LLM client for OpenAI and Anthropic."""

from abc import ABC, abstractmethod
from typing import Optional
from dataclasses import dataclass

import httpx
from openai import AsyncOpenAI
from anthropic import AsyncAnthropic


@dataclass
class Message:
    """Chat message."""

    role: str  # system, user, assistant, tool
    content: str
    tool_calls: Optional[list] = None
    tool_call_id: Optional[str] = None


@dataclass
class ToolCallResult:
    """Result from a tool call."""

    call_id: str
    tool_name: str
    content: str


class LLMClient(ABC):
    """Abstract LLM client."""

    @abstractmethod
    async def chat(
        self,
        messages: list[Message],
        tools: Optional[list[dict]] = None,
    ) -> tuple[str, Optional[list[ToolCallResult]]]:
        """Send chat request and get response."""
        ...


class OpenAIClient(LLMClient):
    """OpenAI API client."""

    def __init__(self, api_key: str, model: str = "gpt-4o", base_url: Optional[str] = None):
        self.client = AsyncOpenAI(api_key=api_key, base_url=base_url)
        self.model = model

    async def chat(
        self,
        messages: list[Message],
        tools: Optional[list[dict]] = None,
    ) -> tuple[str, Optional[list[ToolCallResult]]]:
        openai_messages = [{"role": m.role, "content": m.content} for m in messages]

        request = {
            "model": self.model,
            "messages": openai_messages,
            "max_tokens": 4096,
        }

        if tools:
            request["tools"] = tools
            request["tool_choice"] = "auto"

        response = await self.client.chat.completions.create(**request)

        content = response.choices[0].message.content or ""
        tool_calls = response.choices[0].message.tool_calls

        results = None
        if tool_calls:
            results = []
            for call in tool_calls:
                results.append(
                    ToolCallResult(
                        call_id=call.id,
                        tool_name=call.function.name,
                        content="",  # Will be filled after tool execution
                    )
                )

        return content, results


class AnthropicClient(LLMClient):
    """Anthropic API client."""

    def __init__(self, api_key: str, model: str = "claude-3-5-sonnet-20241022"):
        self.client = AsyncAnthropic(api_key=api_key)
        self.model = model

    async def chat(
        self,
        messages: list[Message],
        tools: Optional[list[dict]] = None,
    ) -> tuple[str, Optional[list[ToolCallResult]]]:
        # Convert messages to Anthropic format
        formatted_messages = []
        system_messages = []

        for m in messages:
            if m.role == "system":
                system_messages.append(m.content)
            else:
                formatted_messages.append({"role": m.role, "content": m.content})

        request = {
            "model": self.model,
            "messages": formatted_messages,
            "max_tokens": 4096,
        }

        if system_messages:
            request["system"] = system_messages[0]

        if tools:
            request["tools"] = tools

        response = await self.client.messages.create(**request)

        content = response.content[0].text if response.content else ""
        tool_calls = [b for b in response.content if b.type == "tool_use"]

        results = None
        if tool_calls:
            results = []
            for call in tool_calls:
                results.append(
                    ToolCallResult(
                        call_id=call.id,
                        tool_name=call.name,
                        content="",  # Will be filled after tool execution
                    )
                )

        return content, results


def create_llm_client(
    provider: str, api_key: str, model: str, base_url: Optional[str] = None
) -> LLMClient:
    """Factory function to create LLM client."""
    if provider == "openai":
        return OpenAIClient(api_key, model, base_url)
    elif provider == "anthropic":
        return AnthropicClient(api_key, model)
    else:
        raise ValueError(f"Unknown provider: {provider}")
