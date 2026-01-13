"""Interactive chat interface for agent-tester."""

import asyncio
from typing import Optional

from rich.console import Console
from rich.panel import Panel

from .config import Settings
from .orchestrator import SessionOrchestrator
from .hippos import HipposClient

console = Console()


async def main(
    hippos_url: str = "http://localhost:8080",
    api_key: str = "dev-api-key",
    llm: str = "openai",
    model: Optional[str] = None,
    config: Optional[str] = None,
):
    """Run interactive chat."""
    # Load settings
    if config:
        settings = Settings.from_yaml(config)
    else:
        settings = Settings(
            hippos_url=hippos_url,
            hippos_api_key=api_key,
        )
        if model:
            llm_config = settings.get_llm(llm)
            llm_config.model = model

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
            "  /history - Show conversation history\n"
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

        cmd = user_input.lower().strip()

        if cmd in ["/quit", "/exit", "/q"]:
            console.print("[yellow]Goodbye![/]")
            break

        if cmd == "/session":
            session = await orchestrator.get_or_create_session(session.id)
            console.print(f"[bold]Session ID:[/] {session.id}")
            continue

        if cmd.startswith("/search "):
            query = user_input[8:].strip()
            results = await orchestrator.search_context(session.id, query, limit=5)
            if results:
                from rich.table import Table

                table = Table(title="Search Results")
                table.add_column("Score")
                table.add_column("Content")
                for r in results:
                    table.add_row(f"{r.score:.2f}", r.content[:100])
                console.print(table)
            else:
                console.print("[yellow]No results found[/]")
            continue

        if cmd == "/history":
            from rich.table import Table

            table = Table(title="Conversation History")
            table.add_column("Role")
            table.add_column("Content")
            for turn in orchestrator.conversation_history:
                content = turn.content[:80] + "..." if len(turn.content) > 80 else turn.content
                table.add_row(turn.role, content)
            console.print(table)
            continue

        # Send message
        with console.status("[bold]Thinking...[/]"):
            response = await orchestrator.chat(user_input, session_id=session.id)

        console.print(f"\n[bold assistant[/] > {response}")

    await orchestrator.close()
