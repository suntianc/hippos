"""CLI interface for agent-tester."""

import asyncio
from typing import Optional

import click
from rich.console import Console
from rich.table import Table
from rich.panel import Panel

from .config import Settings
from .hippos import HipposClient
from .orchestrator import SessionOrchestrator
from .mcp import MCPClientAggregator

console = Console()


@click.group()
def main():
    """Agent Tester - Test hippos with intelligent multi-round dialogue."""
    pass


@main.command()
@click.option("--config", "-c", type=str, help="Path to config file")
@click.option("--hippos-url", default="http://localhost:8080", help="Hippos URL")
@click.option("--api-key", default="dev-api-key", help="Hippos API key")
@click.option("--llm", default="openai", help="LLM provider (openai/anthropic)")
@click.option("--model", help="LLM model")
async def chat(
    config: Optional[str],
    hippos_url: str,
    api_key: str,
    llm: str,
    model: Optional[str],
):
    """Start interactive chat with the agent."""
    from .client import create_llm_client

    # Load settings
    if config:
        settings = Settings.from_yaml(config)
    else:
        settings = Settings(
            hippos_url=hippos_url,
            hippos_api_key=api_key,
        )
        if model:
            settings.llms[llm] = settings.get_llm(llm)
            settings.llms[llm].model = model

    # Initialize clients
    hippos = HipposClient(settings.hippos_url, settings.hippos_api_key)

    # Check health
    if not await hippos.health_check():
        console.print(f"[red]Error: Hippos is not available at {hippos_url}[/]")
        return

    orchestrator = SessionOrchestrator(settings, hippos)

    console.print(
        Panel(
            "[bold green]Agent Tester[/]\n\n"
            "Commands:\n"
            "  /session - Show current session\n"
            "  /search <query> - Search context\n"
            "  /quit - Exit\n",
            title="Welcome",
        )
    )

    session = await orchestrator.get_or_create_session()
    console.print(f"[bold]Current Session:[/] {session.id}")

    while True:
        try:
            user_input = console.input("\n[bold you[/] > ")
        except EOFError:
            break

        if not user_input.strip():
            continue

        if user_input.lower() in ["/quit", "/exit", "/q"]:
            console.print("[yellow]Goodbye![/]")
            break

        if user_input.lower() == "/session":
            session = await orchestrator.get_or_create_session(session.id)
            console.print(f"[bold]Session ID:[/] {session.id}")
            continue

        if user_input.lower().startswith("/search "):
            query = user_input[8:].strip()
            results = await orchestrator.search_context(session.id, query, limit=5)
            if results:
                table = Table(title="Search Results")
                table.add_column("Score")
                table.add_column("Content")
                for r in results:
                    table.add_row(f"{r.score:.2f}", r.content[:100])
                console.print(table)
            else:
                console.print("[yellow]No results found[/]")
            continue

        # Send message
        with console.status("[bold]Thinking...[/]"):
            response = await orchestrator.chat(user_input, session_id=session.id)

        console.print(f"\n[bold assistant[/] > {response}")

    await orchestrator.close()


@main.command()
@click.option("--config", "-c", type=str, help="Path to config file")
@click.option("--hippos-url", default="http://localhost:8080", help="Hippos URL")
@click.option("--api-key", default="dev-api-key", help="Hippos API key")
async def sessions(config: Optional[str], hippos_url: str, api_key: str):
    """List Hippos sessions."""
    hippos = HipposClient(hippos_url, api_key)

    if not await hippos.health_check():
        console.print(f"[red]Error: Hippos is not available at {hippos_url}[/]")
        return

    sessions, total = await hippos.list_sessions()

    table = Table(title=f"Sessions (total: {total})")
    table.add_column("ID")
    table.add_column("Name")
    table.add_column("Status")
    table.add_column("Created")

    for s in sessions:
        table.add_row(s.id, s.name, s.status, s.created_at.strftime("%Y-%m-%d %H:%M"))

    console.print(table)
    await hippos.close()


@main.command()
@click.argument("session_id")
@click.option("--hippos-url", default="http://localhost:8080", help="Hippos URL")
@click.option("--api-key", default="dev-api-key", help="Hippos API key")
async def turns(session_id: str, hippos_url: str, api_key: str):
    """List turns for a session."""
    hippos = HipposClient(hippos_url, api_key)

    if not await hippos.health_check():
        console.print(f"[red]Error: Hippos is not available at {hippos_url}[/]")
        return

    turns, total = await hippos.list_turns(session_id)

    table = Table(title=f"Turns for {session_id} (total: {total})")
    table.add_column("#")
    table.add_column("Role")
    table.add_column("Content")
    table.add_column("Created")

    for t in turns:
        content = t.content[:50] + "..." if len(t.content) > 50 else t.content
        table.add_row(str(t.turn_number), t.role, content, t.created_at.strftime("%Y-%m-%d %H:%M"))

    console.print(table)
    await hippos.close()


@main.command()
@click.argument("session_id")
@click.argument("query")
@click.option("--hippos-url", default="http://localhost:8080", help="Hippos URL")
@click.option("--api-key", default="dev-api-key", help="Hippos API key")
@click.option("--limit", default=5, help="Max results")
async def search(session_id: str, query: str, hippos_url: str, api_key: str, limit: int):
    """Search within a session."""
    hippos = HipposClient(hippos_url, api_key)

    if not await hippos.health_check():
        console.print(f"[red]Error: Hippos is not available at {hippos_url}[/]")
        return

    results = await hippos.search(session_id, query, limit)

    table = Table(title=f"Search Results for '{query}'")
    table.add_column("Score")
    table.add_column("Turn")
    table.add_column("Content")

    for r in results:
        score = r.get("score", 0.0)
        turn = r.get("metadata", {}).get("turn_number", "?")
        content = r.get("content", "")[:100]
        table.add_row(f"{score:.2f}", str(turn), content)

    console.print(table)
    await hippos.close()


@main.command()
@click.option("--hippos-url", default="http://localhost:8080", help="Hippos URL")
async def health(hippos_url: str):
    """Check Hippos health."""
    hippos = HipposClient(hippos_url, "dev-api-key")

    healthy = await hippos.health_check()
    if healthy:
        console.print("[bold green]✓ Hippos is healthy[/]")
    else:
        console.print("[bold red]✗ Hippos is not available[/]")

    await hippos.close()


if __name__ == "__main__":
    main()
