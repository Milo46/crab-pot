import typer
import httpx
import uuid
from pathlib import Path
from rich.console import Console
from rich.table import Table


app = typer.Typer()
console = Console()

client = httpx.Client(base_url="http://localhost:8081")

schemas_app = typer.Typer()
app.add_typer(schemas_app, name="schemas")


@schemas_app.command("list")
def list_schemas(
    full: bool = typer.Option(False, "--full", "-f", help="Show full UUIDs"),
    json_output: bool = typer.Option(False, "--json", "-j", help="Output as JSON")
):
    response = client.get("/schemas")
    response.raise_for_status()

    data = response.json()
    schemas = data.get("schemas", [])

    if not schemas:
        console.print("[yellow]No schemas found[/yellow]")
        return

    if json_output:
        console.print_json(data=data)
        return

    table = Table(title="Schemas")
    table.add_column("ID", style="cyan", no_wrap=True)
    table.add_column("Name", style="green")
    table.add_column("Version", style="blue")
    table.add_column("Description", style="white")

    for schema in schemas:
        schema_id = schema["id"] if full else schema["id"][:8] + "..."
        table.add_row(
            schema_id,
            schema["name"],
            schema["version"],
            schema.get("description", "")[:50],
        )

    console.print(table)


@schemas_app.command("get")
def get_schema(schema_id: uuid.UUID):
    response = client.get(f"/schemas/{schema_id}")
    response.raise_for_status()
    console.print_json(data=response.json())


if __name__ == "__main__":
    app()
