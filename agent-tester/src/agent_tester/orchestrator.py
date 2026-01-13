"""Session orchestrator for multi-round dialogue with dynamic system prompt injection."""

import asyncio
import json
from typing import Optional
from dataclasses import dataclass, field
from datetime import datetime

from .config import Settings
from .client import LLMClient, Message, ToolCallResult, create_llm_client
from .hippos import HipposClient, HipposSession, HipposTurn
from .mcp import MCPClientAggregator, Tool


@dataclass
class ConversationTurn:
    """A single turn in the conversation."""

    role: str  # user, assistant, system, tool
    content: str
    timestamp: datetime = field(default_factory=datetime.now)
    tool_calls: Optional[list] = None
    tool_results: Optional[list[ToolCallResult]] = None


@dataclass
class SearchResult:
    """Search result from Hippos."""

    id: str
    score: float
    content: str
    turn_number: int


class SessionOrchestrator:
    """Orchestrate multi-round dialogue with Hippos and MCP."""

    def __init__(
        self, settings: Settings, hippos: HipposClient, mcp: Optional[MCPClientAggregator] = None
    ):
        self.settings = settings
        self.hippos = hippos
        self.mcp = mcp

        # Initialize LLM client
        llm_config = settings.get_llm()
        self.llm = create_llm_client(
            provider=llm_config.provider,
            api_key=llm_config.api_key,
            model=llm_config.model,
            base_url=llm_config.base_url,
        )

        self.conversation_history: list[ConversationTurn] = []

    async def get_or_create_session(self, session_id: Optional[str] = None) -> HipposSession:
        """Get existing session or create new one."""
        if session_id:
            session = await self.hippos.get_session(session_id)
            if session:
                return session

        # Create new session
        session = await self.hippos.create_session(
            name=f"agent-session-{datetime.now().strftime('%Y%m%d%H%M%S')}",
            description="Created by agent-tester",
        )
        return session

    async def search_context(
        self, session_id: str, query: str, limit: int = 5
    ) -> list[SearchResult]:
        """Search for relevant context in Hippos."""
        results = await self.hippos.search(session_id, query, limit)
        return [
            SearchResult(
                id=r["id"],
                score=r.get("score", 0.0),
                content=r.get("content", ""),
                turn_number=r.get("metadata", {}).get("turn_number", 0),
            )
            for r in results
        ]

    def render_system_prompt(self, session_id: str, context: str = "") -> str:
        """Render system prompt with dynamic injection."""
        return self.settings.system_prompt_template.format(
            session_id=session_id,
            context=context or "No relevant context found.",
        )

    def format_context(self, results: list[SearchResult]) -> str:
        """Format search results as context."""
        if not results:
            return ""

        context_parts = []
        for r in results:
            truncated = r.content[:200] + "..." if len(r.content) > 200 else r.content
            context_parts.append(f"- [Turn {r.turn_number}, score={r.score:.2f}]: {truncated}")

        return "\n".join(context_parts)

    def format_history(self, turns: list[HipposTurn], max_turns: int = 10) -> list[Message]:
        """Format conversation history as messages."""
        messages = []

        # Take only the most recent turns
        recent_turns = turns[-max_turns:]

        for turn in recent_turns:
            if turn.role in ["user", "assistant"]:
                messages.append(
                    Message(
                        role=turn.role,
                        content=turn.content,
                    )
                )

        return messages

    async def execute_tool_call(self, call: ToolCallResult, arguments: dict) -> str:
        """Execute a tool call via MCP."""
        if not self.mcp:
            return f"Error: MCP not configured"

        try:
            result = await self.mcp.call_tool(call.tool_name, arguments)
            return result
        except Exception as e:
            return f"Error executing {call.tool_name}: {e}"

    async def chat(
        self,
        user_input: str,
        session_id: Optional[str] = None,
        system_prompt: Optional[str] = None,
        do_search: bool = True,
    ) -> str:
        """Send a message and get response with full orchestration."""
        # Get or create session
        session = await self.get_or_create_session(session_id)
        session_id = session.id

        # Search for relevant context
        context = ""
        if do_search:
            search_results = await self.search_context(session_id, user_input, limit=5)
            context = self.format_context(search_results)

        # Render system prompt
        base_system = system_prompt or self.render_system_prompt(session_id, context)

        # Get conversation history
        turns, _ = await self.hippos.list_turns(
            session_id, page_size=self.settings.max_history_turns
        )
        history_messages = self.format_history(turns)

        # Build messages: system first, then history, then user
        messages = [Message(role="system", content=base_system)]
        messages.extend(history_messages)
        messages.append(Message(role="user", content=user_input))

        # Get tools from MCP
        tools = self.mcp.get_tools() if self.mcp else None

        # Call LLM
        response, tool_calls = await self.llm.chat(messages, tools)

        # Execute tool calls if needed
        tool_results = None
        if tool_calls:
            tool_results = []
            for call in tool_calls:
                # Parse arguments
                try:
                    args = json.loads(call.content) if isinstance(call.content, str) else {}
                except json.JSONDecodeError:
                    args = {}

                # Execute tool
                result = await self.execute_tool_call(call, args)
                tool_results.append(
                    ToolCallResult(
                        call_id=call.call_id,
                        tool_name=call.tool_name,
                        content=result,
                    )
                )

                # Append tool result to messages
                messages.append(
                    Message(
                        role="tool",
                        content=result,
                        tool_call_id=call.call_id,
                    )
                )

            # Continue conversation with tool results
            response, _ = await self.llm.chat(messages, None)

        # Record turns to Hippos
        # User turn
        await self.hippos.add_turn(session_id, role="user", content=user_input)

        # Assistant turn (with tool results in metadata if any)
        assistant_turn = await self.hippos.add_turn(
            session_id,
            role="assistant",
            content=response,
        )

        # Update conversation history
        self.conversation_history.append(
            ConversationTurn(
                role="user",
                content=user_input,
            )
        )
        self.conversation_history.append(
            ConversationTurn(
                role="assistant",
                content=response,
                tool_calls=tool_calls,
                tool_results=tool_results,
            )
        )

        return response

    async def close(self):
        """Clean up resources."""
        if self.mcp:
            await self.mcp.shutdown()
        await self.hippos.close()
