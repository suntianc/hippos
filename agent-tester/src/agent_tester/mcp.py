"""MCP client aggregator for multi-server support."""

import asyncio
import json
from typing import Optional, Callable, Any
from dataclasses import dataclass, field
from pathlib import Path

import anyio
import httpx
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


@dataclass
class MCPServer:
    """MCP Server configuration."""

    name: str
    command: str
    args: list[str] = field(default_factory=list)
    env: dict[str, str] = field(default_factory=dict)
    url: Optional[str] = None


@dataclass
class Tool:
    """MCP Tool."""

    name: str
    description: str
    input_schema: dict


class MCPClientAggregator:
    """Aggregate multiple MCP servers into a unified interface."""

    def __init__(self):
        self.sessions: dict[str, ClientSession] = {}
        self.tools: dict[str, Tool] = {}
        self._shutdown_event = asyncio.Event()

    async def connect_server(self, server: MCPServer) -> None:
        """Connect to an MCP server."""
        if server.url:
            # HTTP transport (not yet widely supported)
            raise NotImplementedError("HTTP transport not yet implemented")

        # Stdio transport
        server_params = StdioServerParameters(
            command=server.command,
            args=server.args,
            env=server.env if server.env else None,
        )

        async with stdio_client(server_params) as (read, write):
            async with ClientSession(read, write) as session:
                # Initialize the session
                await session.initialize()

                # List available tools
                result = await session.list_tools()

                # Register tools with prefix
                for tool in result.tools:
                    prefixed_name = f"{server.name}_{tool.name}"
                    self.tools[prefixed_name] = Tool(
                        name=prefixed_name,
                        description=tool.description,
                        input_schema=tool.inputSchema,
                    )
                    self.sessions[prefixed_name] = session

    async def call_tool(self, name: str, arguments: dict[str, Any]) -> str:
        """Call a tool by name."""
        if name not in self.sessions:
            raise ValueError(f"Unknown tool: {name}")

        session = self.sessions[name]
        result = await session.call_tool(name, arguments)

        # Return the text content
        if result.content and len(result.content) > 0:
            if hasattr(result.content[0], "text"):
                return result.content[0].text
            return str(result.content[0])
        return ""

    def get_tools(self) -> list[dict]:
        """Get all tools as OpenAI format."""
        return [
            {
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema,
                },
            }
            for tool in self.tools.values()
        ]

    async def shutdown(self) -> None:
        """Shutdown all connections."""
        self._shutdown_event.set()
        for session in self.sessions.values():
            # Session cleanup is handled by context manager exit
            pass
        self.sessions.clear()
        self.tools.clear()


async def connect_hippos_mcp(hippos_path: str = "./target/release/hippos") -> MCPClientAggregator:
    """Connect to Hippos MCP server."""
    aggregator = MCPClientAggregator()

    # Check if hippos binary exists
    hippos_bin = Path(hippos_path)
    if not hippos_bin.exists():
        raise FileNotFoundError(f"Hippos binary not found: {hippos_path}")

    server = MCPServer(
        name="hippos",
        command=str(hippos_bin),
        env={"HIPPOS_MCP_MODE": "1"},
    )

    await aggregator.connect_server(server)
    return aggregator
