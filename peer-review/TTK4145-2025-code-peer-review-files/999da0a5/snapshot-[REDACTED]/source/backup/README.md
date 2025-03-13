# Backup module
All elevators have an active backup module. The function `backup.Run()` gets constant worldview updates from the primary.
Worldviews are broadcasted to all backups running on all active elevators. A list of active peers is part of this worldview.

In the event of a primary disconnection, a `timeout` will trigger. The active peer list functions like a queue determining which elevator will take over as the new primary. After the timeout trigger happens, each running backup will check the first ID in the queue. If it matches its own, it takes over as primary. If not, it prematurely removes the first peer in the queue, exits the loop and waits for updates.

If the removed peer *is* alive and has taken over as primary, the latest worldview will be updated before the next timeout, and the modified worldview is void.

If not, the process will repeat and iterate until a primary is detected or itself becomes primary. This algorithm ensures slaves will compete for primary status.
