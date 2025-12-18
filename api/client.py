import httpx
from rich.console import Console


console = Console()


class APIClient:
    def __init__(self, base_url: str = "http://localhost:8081"):
        self.client = httpx.Client(base_url=base_url, timeout=30.0)

    def get(self, path: str, **kwargs):
        try:
            response = self.client.get(path, **kwargs)
            response.raise_for_status()
            return response
        except httpx.HTTPStatusError as e:
            console.print(f"[red]X HTTP {e.response.status_code}: {e.response.text}[/red]")
            raise
        except httpx.RequestError as e:
            console.print(f"[red]X Request failed: {e}[/red]")
            raise

    def post(self, path: str, **kwargs):
        try:
            response = self.client.post(path, **kwargs)
            response.raise_for_status()
            return response
        except httpx.HTTPStatusError as e:
            console.print(f"[red]✗ HTTP {e.response.status_code}: {e.response.text}[/red]")
            raise
        except httpx.RequestError as e:
            console.print(f"[red]✗ Request failed: {e}[/red]")
            raise
    
    def close(self):
        self.client.close()


api_client = APIClient()
