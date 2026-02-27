import typer
import httpx
import uuid
import os
from rich.console import Console
from rich.table import Table
from datetime import datetime
from typing import Any
from pydantic import BaseModel, Field


API_KEY = os.getenv("LOG_SERVER_API_KEY", "secret-key")

app = typer.Typer()
console = Console()

client = httpx.Client(
    base_url="http://localhost:8081",
    headers={"X-Api-Key": API_KEY},
)

schemas_app = typer.Typer()
app.add_typer(schemas_app, name="schemas")


logs_app = typer.Typer()
app.add_typer(logs_app, name="logs")


class Schema(BaseModel):
    id: uuid.UUID
    name: str
    version: str
    description: str | None = None
    schema_definition: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    @classmethod
    def from_api_response(cls, data: dict) -> "Schema":
        return cls.model_validate(data)

    def to_table_row(self, full_id: bool = False) -> tuple:
        schema_id = str(self.id) if full_id else str(self.id)[:8] + "..."
        description = (self.description or "")[:50]
        return (schema_id, self.name, self.version, description)


class Log(BaseModel):
    id: int
    schema_id: uuid.UUID
    log_data: dict[str, Any]
    created_at: datetime

    @classmethod
    def from_api_response(cls, data: dict) -> "Log":
        return cls.model_validate(data)

    def to_table_row(self, full_id: bool = False) -> tuple:
        schema_id = str(self.schema_id) if full_id else str(self.schema_id)[:8] + "..."
        # Get a preview of the data (first few key-value pairs)
        data_preview = (
            str(self.log_data)[:50] + "..."
            if len(str(self.log_data)) > 50
            else str(self.log_data)
        )
        return (
            str(self.id),
            schema_id,
            self.created_at.strftime("%Y-%m-%d %H:%M:%S"),
            data_preview,
        )

    def to_table_row_expanded(self, full_id: bool = False) -> tuple:
        import json
        from rich.syntax import Syntax

        schema_id = str(self.schema_id) if full_id else str(self.schema_id)[:8] + "..."

        json_str = json.dumps(self.log_data, indent=2)
        syntax = Syntax(json_str, "json", theme="monokai", line_numbers=False)

        return (
            str(self.id),
            schema_id,
            self.created_at.strftime("%Y-%m-%d %H:%M:%S"),
            syntax,
        )


@schemas_app.command("list")
def list_schemas(
    full: bool = typer.Option(False, "--full", "-f", help="Show full UUIDs"),
    json_output: bool = typer.Option(False, "--json", "-j", help="Output as JSON"),
):
    response = client.get("/schemas")
    response.raise_for_status()
    data = response.json()

    schemas_data = data.get("schemas", [])

    if not schemas_data:
        console.print("[yellow]No schemas found[/yellow]")
        return

    schemas = [Schema.model_validate(s) for s in schemas_data]

    if json_output:
        console.print_json(data=[s.model_dump(mode="json") for s in schemas])
        return

    table = Table(title="Schemas")
    table.add_column("ID", style="cyan", no_wrap=True)
    table.add_column("Name", style="green")
    table.add_column("Version", style="blue")
    table.add_column("Description", style="white")

    for schema in schemas:
        table.add_row(*schema.to_table_row(full_id=full))

    console.print(table)


@schemas_app.command("get")
def get_schema(
    schema_id: uuid.UUID = typer.Option(None, "--id", help="Schema UUID"),
    name: str = typer.Option(None, "--name", "-n", help="Schema name"),
    version: str = typer.Option(
        None, "--version", "-v", help="Schema version (defaults to latest)"
    ),
    json_output: bool = typer.Option(False, "--json", "-j", help="Output as JSON"),
):
    if schema_id and name:
        console.print("[red]Error: Cannot specify both --id and --name[/red]")
        raise typer.Exit(1)

    if schema_id:
        if version:
            console.print(
                "[yellow]Warning: --version is ignored when using --id[/yellow]"
            )
        response = client.get(f"/schemas/{schema_id}")
        response.raise_for_status()
        schema = Schema.model_validate(response.json())
    elif name:
        params = {"name": name}
        if version:
            params["version"] = version
        response = client.get("/schemas", params=params)
        response.raise_for_status()

        data = response.json()
        schemas_list = data.get("schemas", [])

        if not schemas_list:
            console.print(
                f"[yellow]No schema found with name '{name}'{' and version ' + version if version else ''}[/yellow]"
            )
            raise typer.Exit(1)

        schema = Schema.model_validate(schemas_list[0])

        if len(schemas_list) > 1:
            console.print(
                f"[yellow]Warning: Found {len(schemas_list)} schemas, displaying the first one[/yellow]\n"
            )
    else:
        console.print("[red]Error: Must specify either --id or --name[/red]")
        raise typer.Exit(1)

    if json_output:
        console.print_json(data=schema.model_dump(mode="json"))
        return

    schema_def = schema.schema_definition
    properties = schema_def.get("properties", {})
    required_fields = schema_def.get("required", [])

    def_table = Table(show_header=True, header_style="bold blue", box=None)
    def_table.add_column("Field", style="cyan")
    def_table.add_column("Type", style="green")
    def_table.add_column("Required", style="yellow", justify="center")
    def_table.add_column("Constraints", style="white")

    for field_name, field_props in properties.items():
        field_type = field_props.get("type", "N/A")
        is_required = "✓" if field_name in required_fields else ""

        constraints = []
        if "enum" in field_props:
            enum_values = ", ".join(str(v) for v in field_props["enum"])
            constraints.append(f"enum: [{enum_values}]")
        if "pattern" in field_props:
            constraints.append(f"pattern: {field_props['pattern']}")
        if "minLength" in field_props:
            constraints.append(f"minLength: {field_props['minLength']}")
        if "maxLength" in field_props:
            constraints.append(f"maxLength: {field_props['maxLength']}")
        if "minimum" in field_props:
            constraints.append(f"min: {field_props['minimum']}")
        if "maximum" in field_props:
            constraints.append(f"max: {field_props['maximum']}")
        if "format" in field_props:
            constraints.append(f"format: {field_props['format']}")

        constraints_str = "\n".join(constraints) if constraints else ""

        def_table.add_row(field_name, field_type, is_required, constraints_str)

    table = Table(
        title=f"Schema: {schema.name}", show_header=True, header_style="bold magenta"
    )
    table.add_column("Field", style="cyan", width=20)
    table.add_column("Value", style="white")

    table.add_row("ID", str(schema.id))
    table.add_row("Name", schema.name)
    table.add_row("Version", schema.version)
    table.add_row("Description", schema.description or "N/A")
    table.add_row("Created At", schema.created_at.strftime("%Y-%m-%d %H:%M:%S"))
    table.add_row("Updated At", schema.updated_at.strftime("%Y-%m-%d %H:%M:%S"))
    table.add_row("Schema Definition", def_table)

    console.print(table)


@logs_app.command("list")
def list_logs(
    schema_name: str = typer.Argument(None, help="Schema name to filter logs"),
    full: bool = typer.Option(False, "--full", "-f", help="Show full UUIDs"),
    json_output: bool = typer.Option(False, "--json", "-j", help="Output as JSON"),
    expand: bool = typer.Option(
        False, "--expand", "-e", help="Show full log data (pretty-printed)"
    ),
    limit: int = typer.Option(
        10, "--limit", "-l", help="Maximum number of logs to retrieve"
    ),
):
    if not schema_name:
        console.print("[red]Error: schema_name is required[/red]")
        raise typer.Exit(1)

    try:
        params = {"limit": limit}

        response = client.get(f"/logs/schema/{schema_name}", params=params)
        response.raise_for_status()
        data = response.json()

        logs_data = data.get("logs", [])

        if not logs_data:
            console.print(f"[yellow]No logs found for schema '{schema_name}'[/yellow]")
            return

        logs = [Log.model_validate(log) for log in logs_data]

        if json_output:
            console.print_json(data=[log.model_dump(mode="json") for log in logs])
            return

        table = Table(title=f"Logs for Schema: {schema_name}")
        table.add_column("Log ID", style="cyan", no_wrap=True)
        table.add_column("Schema ID", style="blue", no_wrap=True)
        table.add_column("Created At", style="green")
        table.add_column("Data Preview", style="white")

        for log in logs:
            if expand:
                table.add_row(*log.to_table_row_expanded(full_id=full))
            else:
                table.add_row(*log.to_table_row(full_id=full))

        console.print(table)

    except httpx.HTTPError as e:
        console.print(f"[red]Error fetching logs: {e}[/red]")
        raise typer.Exit(1)


if __name__ == "__main__":
    app()
