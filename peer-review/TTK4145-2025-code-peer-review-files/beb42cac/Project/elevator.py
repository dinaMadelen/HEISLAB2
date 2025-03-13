import asyncio
from elevator_driver import (
    ElevatorDriver, MD_UP, MD_DOWN, MD_STOP,
    BT_HALL_UP, BT_HALL_DOWN, BT_CAB
)

DOOR_OPEN_TIME = 1.0

# Elevator states for clarity
IDLE = 0
MOVING = 1
DOOR_OPEN = 2

class DistributedElevatorController:
    """
    Controls a single elevator using an external driver.
    It processes events from the driver and handles both local and
    networked hall calls (if a network is connected).
    """
    def __init__(self, host, port, num_floors, network=None):
        # Connect to the remote elevator driver (hardware or simulator)
        self.driver = ElevatorDriver(host=host, port=port, num_floors=num_floors)
        self.driver.connect()
        self.driver.start_polling()

        self.state = IDLE
        self.current_floor = -1  # Unknown until calibrated by sensor
        self.num_floors = num_floors

        # Set of floors with pending requests
        self.orders = set()

        self.current_direction = MD_STOP

        # Reference to network communication (None if operating alone)
        self.network = network

    async def run(self):
        """
        Main loop: waits for events from the driver and processes them.
        Uses asyncio to avoid blocking on the driver's event queue.
        """
        print("Distributed Elevator Control started. Press Ctrl+C to quit.")
        loop = asyncio.get_running_loop()
        while True:
            event = await loop.run_in_executor(None, self.driver.event_queue.get)
            await self.handle_event(event)

    async def handle_event(self, event):
        # Dispatch events based on type
        if event.event_type == "button_press":
            await self.on_button_press(event.floor, event.button)
        elif event.event_type == "floor_sensor":
            await self.on_floor_sensor(event.floor)
        elif event.event_type == "stop_button":
            await self.on_stop_button(event.value)
        elif event.event_type == "obstruction":
            await self.on_obstruction(event.value)

    async def on_button_press(self, floor, button):
        print(f"Button pressed at floor {floor}, button type {button}")

        # For hall calls, send them over the network if available.
        if button in (BT_HALL_UP, BT_HALL_DOWN):
            if self.network is not None:
                direction_str = "up" if button == BT_HALL_UP else "down"
                await self.network.send_hall_call(floor, direction_str)
            else:
                # Handle locally in single-elevator mode.
                self.orders.add(floor)
                self.driver.set_button_lamp(button, floor, True)
        else:
            # Internal cab calls are handled locally.
            self.orders.add(floor)
            self.driver.set_button_lamp(button, floor, True)

        if self.state == IDLE:
            await self.start_moving_to_next_order()

        # If doors are already open and the call is at the same floor, refresh the door timer.
        if self.state == DOOR_OPEN and floor == self.current_floor:
            await self.stop_at_floor(self.current_floor)

    async def on_floor_sensor(self, floor):
        print(f"Arrived at floor {floor}")
        # First sensor reading calibrates the elevator position.
        if self.current_floor == -1:
            self.current_floor = floor
            await self.stop_at_floor(floor)

        self.current_floor = floor
        self.driver.set_floor_indicator(floor)
        if self.state == MOVING and floor in self.orders:
            await self.stop_at_floor(floor)

    async def on_stop_button(self, is_pressed):
        print(f"Stop button event: pressed={is_pressed}")
        if is_pressed:
            # Turn off all buttons and stop the elevator immediately.
            for f in range(self.num_floors):
                for b_type in (BT_HALL_UP, BT_HALL_DOWN, BT_CAB):
                    self.driver.set_button_lamp(b_type, f, False)
            self.driver.set_stop_lamp(True)
            self.driver.set_motor_direction(MD_STOP)
            self.state = IDLE
            self.orders.clear()
        else:
            self.driver.set_stop_lamp(False)

    async def on_obstruction(self, is_obstructed):
        print(f"Obstruction event: {is_obstructed}")
        if is_obstructed:
            self.driver.set_motor_direction(MD_STOP)
        else:
            if self.state == MOVING:
                self.driver.set_motor_direction(self.current_direction)

    async def start_moving_to_next_order(self):
        # If no orders, remain idle
        if not self.orders:
            self.state = IDLE
            self.driver.set_motor_direction(MD_STOP)
            return

        current = self.current_floor if self.current_floor != -1 else 0
        # Choose the nearest requested floor
        closest_floor = min(self.orders, key=lambda f: abs(f - current))

        if self.current_floor == -1:
            direction = MD_UP if closest_floor > 0 else MD_DOWN
        else:
            if closest_floor > self.current_floor:
                direction = MD_UP
            elif closest_floor < self.current_floor:
                direction = MD_DOWN
            else:
                await self.stop_at_floor(closest_floor)
                return

        self.current_direction = direction
        self.driver.set_motor_direction(direction)
        self.state = MOVING

    async def stop_at_floor(self, floor):
        # Stop the elevator, open doors, and clear the order.
        self.driver.set_motor_direction(MD_STOP)
        self.state = DOOR_OPEN

        if floor in self.orders:
            self.orders.remove(floor)

        for b_type in (BT_HALL_UP, BT_HALL_DOWN, BT_CAB):
            self.driver.set_button_lamp(b_type, floor, False)
        self.driver.set_door_open_lamp(True)

        print(f"Elevator stopped at floor {floor}. Doors opening.")
        await asyncio.sleep(DOOR_OPEN_TIME)

        if self.state == DOOR_OPEN:
            self.driver.set_door_open_lamp(False)
            print(f"Elevator doors closing at floor {floor}.")
            if self.orders:
                await self.start_moving_to_next_order()
            else:
                self.state = IDLE
                self.driver.set_motor_direction(MD_STOP)


async def main():
    import os 
    controller = DistributedElevatorController(
        host=os.environ.get('SERVER_IP'),  # The driver server's IP address
        port=15657,
        num_floors=4
    )
    await controller.run()

if __name__ == "__main__":
    asyncio.run(main())
