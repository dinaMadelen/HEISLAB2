import os 
import argparse
import asyncio

from elevator import DistributedElevatorController
from network import PeerNetwork
from elevator_driver import MD_UP, MD_DOWN, MD_STOP

async def main():
    # Parse command line arguments
    parser = argparse.ArgumentParser(description="Distributed Elevator Control System")
    parser.add_argument("--id", type=int, required=True, help="Unique elevator ID")
    parser.add_argument("--driver-host", type=str, default=os.environ.get('SERVER_IP'),
                        help="Host where the elevator driver runs")
    parser.add_argument("--driver-port", type=int, default=15657, help="Port for the elevator driver")
    parser.add_argument("--listen-port", type=int, required=True, help="TCP port to listen on for networking")
    parser.add_argument("--floors", type=int, default=4, help="Number of floors")
    parser.add_argument("--peers", type=str, nargs="*", default=[],
                        help="Peer addresses (format: host:port)")
    args = parser.parse_args()

    # Build the list of peer addresses from arguments
    peer_addresses = []
    for peer in args.peers:
        if ":" in peer:
            host, port_str = peer.split(":")
            port = int(port_str)
        else:
            host = peer  # Assume just an IP was given
            port = 10000 + int(peer.split(".")[-1]) % 100  # Default port (can be changed)
        # Skip adding self to peers
        if host == args.driver_host and port == args.listen_port:
            continue
        peer_addresses.append((host, port))

    # Create the elevator controller, connecting to the driver
    elevator = DistributedElevatorController(
        host=args.driver_host,
        port=args.driver_port,
        num_floors=args.floors,
        network=None  # Network will be added next
    )
    elevator.id = args.id

    # Define a simple get_status function to report the elevator state
    def get_status():
        return {
            "id": elevator.id,
            "floor": elevator.current_floor,
            "direction": {MD_UP: "up", MD_DOWN: "down", MD_STOP: "stop"}.get(elevator.current_direction),
            "state": "idle" if elevator.state == 0 else "moving" if elevator.state == 1 else "door_open"
        }
    elevator.get_status = get_status

    # Create the network layer to handle peer-to-peer communication
    network = PeerNetwork(
        elevator_id=args.id,
        listen_host=args.driver_host,  # Using driver host here (could be modified to use a separate listen IP)
        listen_port=args.listen_port,
        peer_addresses=peer_addresses,
        elevator=elevator
    )
    elevator.network = network

    # Start the network tasks (server and connecting to peers)
    await network.start()

    # Periodically broadcast this elevator's status to peers
    async def status_updater():
        while True:
            await network.send_status_update()
            await asyncio.sleep(0.5)
    asyncio.create_task(status_updater())

    # Start the main elevator event loop
    await elevator.run()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Shutting down elevator system.")