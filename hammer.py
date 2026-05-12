import asyncio
import sys
import time
import uuid
from collections import defaultdict

import aiohttp

# --- CONFIGURATION ---
TARGET_URL = "http://192.168.1.140:3000"
CONCURRENT_WORKERS = 200  # Start here, push to 500+ to break it
REQUESTS_PER_WORKER = 50  # Cycles each worker will run
TIMEOUT_SECONDS = 5

# Terminal Colors
C_GREEN = "\033[92m"
C_RED = "\033[91m"
C_YELLOW = "\033[93m"
C_CYAN = "\033[96m"
C_RESET = "\033[0m"

# Global Stats Tracker
stats = {
    "requests": 0,
    "success": 0,
    "http_4xx": 0,
    "http_5xx": 0,
    "dropped_connections": 0,
    "route_stats": defaultdict(lambda: {"success": 0, "fail": 0}),
}


async def make_request(session, method, path, json=None):
    """Executes a request and records the result safely."""
    start = time.perf_counter()
    stats["requests"] += 1
    route_key = f"{method} {path.split('/')[1] if len(path) > 1 else '/'}"

    try:
        async with session.request(
            method, f"{TARGET_URL}{path}", json=json, timeout=TIMEOUT_SECONDS
        ) as response:
            await response.read()  # Force read to complete connection

            if response.status < 400:
                stats["success"] += 1
                stats["route_stats"][route_key]["success"] += 1
                return True, response.status
            elif response.status < 500:
                stats["http_4xx"] += 1
                stats["route_stats"][route_key]["fail"] += 1
                return False, response.status
            else:
                stats["http_5xx"] += 1
                stats["route_stats"][route_key]["fail"] += 1
                return False, response.status

    except (aiohttp.ClientError, asyncio.TimeoutError) as e:
        # This is where we catch the server dropping connections under pressure
        stats["dropped_connections"] += 1
        stats["route_stats"][route_key]["fail"] += 1
        return False, "CONN_DROP"


async def worker(session, worker_id):
    """Simulates a user going through a full API lifecycle."""
    for i in range(REQUESTS_PER_WORKER):
        # 1. Hit the base route
        await make_request(session, "GET", "/")

        # 2. Add an item
        item_id = str(uuid.uuid4())
        item_name = f"LoadTest_W{worker_id}_R{i}_{str(uuid.uuid4())[:8]}"
        success, _ = await make_request(
            session, "POST", "/items", json={"id": item_id, "name": item_name}
        )

        if success:
            # 3. Get the specific item
            await make_request(session, "GET", f"/items/{item_id}")

            # 4. Update the item
            await make_request(
                session,
                "PATCH",
                f"/items/{item_id}",
                json={"name": f"{item_name}_updated"},
            )

            # 5. Delete the item
            await make_request(session, "DELETE", f"/items/{item_id}")

        # 6. Occasionally query all items (Expensive for DB)
        if i % 10 == 0:
            await make_request(session, "GET", "/items")


async def chaos_monkey(session):
    """A rogue function that randomly deletes everything, testing concurrency locks."""
    while True:
        await asyncio.sleep(3)  # Wait a bit, then wipe the board
        await make_request(session, "DELETE", "/items")


async def print_dashboard():
    """Prints a beautifully formatted, updating dashboard."""
    start_time = time.time()
    while True:
        await asyncio.sleep(1)
        elapsed = time.time() - start_time
        rps = stats["requests"] / elapsed if elapsed > 0 else 0

        # Clear screen and move cursor to top
        sys.stdout.write("\033[2J\033[H")

        print(f"{C_CYAN}=== RUST AXUM PRESSURE TESTER ==={C_RESET}")
        print(f"Time Elapsed: {elapsed:.1f}s | RPS: {rps:.1f}\n")

        print(f"{C_GREEN}Success:{C_RESET} {stats['success']}")
        print(f"{C_YELLOW}HTTP 4xx (Not Found/Bad Req):{C_RESET} {stats['http_4xx']}")
        print(f"{C_RED}HTTP 5xx (Server Errors/Locked):{C_RESET} {stats['http_5xx']}")
        print(f"{C_RED}Dropped Connections:{C_RESET} {stats['dropped_connections']}\n")

        print(f"{C_CYAN}--- Route Breakdown ---{C_RESET}")
        for route, data in stats["route_stats"].items():
            total = data["success"] + data["fail"]
            if total > 0:
                sr = (data["success"] / total) * 100
                print(f"{route:<15} | Hits: {total:<6} | Success: {sr:.1f}%")


async def main():
    # TCPConnector limit=0 means "open infinite connections, let the OS handle it"
    connector = aiohttp.TCPConnector(limit=0)
    async with aiohttp.ClientSession(connector=connector) as session:
        # Start the dashboard task
        dashboard_task = asyncio.create_task(print_dashboard())

        # Start the chaos monkey to test the DELETE all route
        chaos_task = asyncio.create_task(chaos_monkey(session))

        # Spin up the workers
        workers = [worker(session, i) for i in range(CONCURRENT_WORKERS)]

        # Wait for all workers to finish
        await asyncio.gather(*workers)

        # Clean up background tasks
        dashboard_task.cancel()
        chaos_task.cancel()

        # Final Print
        print(f"\n{C_GREEN}TEST COMPLETE.{C_RESET} Server survived... or did it?")


if __name__ == "__main__":
    # Windows-specific fix for asyncio
    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    asyncio.run(main())
