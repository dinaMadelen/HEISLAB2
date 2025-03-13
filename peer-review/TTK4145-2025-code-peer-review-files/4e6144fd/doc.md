Main
---

Depending on arguments added will do one of the following:

- Enter test mode (currently testing TCP)
- Enter listener mode (not implemented)
- Enter backup mode (not implemented)
- Continue to normal mode (not implemented)

In test mode:
---
Will start some test code.

In listener mode:
---
Will open a listener node and print to terminal the shared state. + Any messages on the network.

In backup mode:
---
Will just run the local heartbeat code in backup.

Normal mode:
---
Firstly spawns a backup from the local heartbeat module.
Will start the Shared State, then the Network, then the Elevator.