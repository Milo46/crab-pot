import asyncio
import json
import sys
from typing import Callable
import websockets


class EventHandlerRegistry:
    def __init__(self):
        self._handlers: dict[str, list[Callable]] = {}

    def register(self, event_type: str):
        def decorator(func: Callable):
            if event_type not in self._handlers:
                self._handlers[event_type] = []
            self._handlers[event_type].append(func)
            return func
        return decorator

    def get_handlers(self, event_type: str) -> list[Callable]:
        return self._handlers.get(event_type, [])

    def dispatch(self, event: dict):
        event_type = event.get("event_type", "")
        handlers = self.get_handlers(event_type)

        if not handlers:
            print(f"No handlers registered for event type: {event_type}")
            return

        for handler in handlers:
            try:
                handler(event)
            except Exception as e:
                print(f"Error in handler {handler.__name__}: {e}", file=sys.stderr)


class LogListener:
    def __init__(self):
        self.events = EventHandlerRegistry()
        self._register_handlers()

    def _register_handlers(self):
        @self.events.register("created")
        def on_log_create(event: dict[str, str]) -> None:
            print(f"Log created: {event}")

        @self.events.register("deleted")
        def on_log_delete(event: dict[str, str]) -> None:
            print(f"Log deleted: {event}")

    async def listen(self, url: str):
        print(f"Connecting to {url}")

        try:
            async with websockets.connect(url) as websocket:
                print("Connected! Listening for logs events...")

                async for message in websocket:
                    try:
                        event = json.loads(message)
                        self.events.dispatch(event)

                    except json.JSONDecodeError as e:
                        print(f"Failed to parse message: {e}", file=sys.stderr)
                    except Exception as e:
                        print(f"Error processing message: {e}", file=sys.stderr)

        except websockets.exceptions.WebSocketException as e:
            print(f"WebSocket error: {e}", file=sys.stderr)
            return 1
        except KeyboardInterrupt:
            print("\nDisconnected.")
            return 0


async def main():
    listener = LogListener()
    return await listener.listen("ws://localhost:8081/ws/logs")


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
