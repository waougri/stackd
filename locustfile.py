import random
import uuid

from locust import HttpUser, between, events, task

# Track route-level stats ourselves
route_stats = {
    "GET_ALL": 0,
    "GET_ONE": 0,
    "CREATE": 0,
    "UPDATE": 0,
    "DELETE": 0,
}


@events.request.add_listener
def _(request_type, name, response_time, response_length, exception, **kwargs):
    if exception:
        return

    if name in route_stats:
        route_stats[name] += 1


class RustServerTester(HttpUser):
    wait_time = between(0.01, 0.05)

    item_ids = []

    def on_start(self):
        """
        Seed database with known-good items.
        """
        for _ in range(100):
            item_id = str(uuid.uuid4())

            payload = {"id": item_id, "name": f"seed_{item_id[:8]}"}

            response = self.client.post("/items", json=payload, name="SEED")

            if response.status_code == 201:
                self.item_ids.append(item_id)

    @task(10)
    def get_all_items(self):
        self.client.get("/items", name="GET_ALL")

    @task(20)
    def get_single_item(self):
        if not self.item_ids:
            return

        item_id = random.choice(self.item_ids)

        with self.client.get(
            f"/items/{item_id}",
            name="GET_ONE",
            catch_response=True,
        ) as response:
            if response.status_code == 404:
                response.failure("Unexpected 404")

    @task(8)
    def create_item(self):
        item_id = str(uuid.uuid4())

        payload = {"id": item_id, "name": f"load_{item_id[:8]}"}

        with self.client.post(
            "/items",
            json=payload,
            name="CREATE",
            catch_response=True,
        ) as response:
            if response.status_code == 201:
                self.item_ids.append(item_id)
            else:
                response.failure(response.text)

    @task(4)
    def update_item(self):
        if not self.item_ids:
            return

        item_id = random.choice(self.item_ids)

        payload = {"name": f"updated_{random.randint(1, 1_000_000)}"}

        with self.client.patch(
            f"/items/{item_id}",
            json=payload,
            name="UPDATE",
            catch_response=True,
        ) as response:
            if response.status_code >= 400:
                response.failure(response.text)

    @task(2)
    def delete_item(self):
        if not self.item_ids:
            return

        item_id = random.choice(self.item_ids)

        with self.client.delete(
            f"/items/{item_id}",
            name="DELETE",
            catch_response=True,
        ) as response:
            if response.status_code == 200:
                try:
                    self.item_ids.remove(item_id)
                except ValueError:
                    pass
            else:
                response.failure(response.text)
