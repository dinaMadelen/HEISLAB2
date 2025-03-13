Local Heartbeat / Process Pair
---
Not implemented yet.

Implements the Local Heartbeat module, which will either be run in main or in backup mode.

Main mode:
---
When in main mode, will send a heartbeat to the backup and also read heartbeats from the backup.
Should the backup not respond, it will kill the backup and spawn a new one.

Backup mode
---
When in backup mode, will send a heartbeat to the main and also read heartbeats from the main.
Should the main not respond, it will kill the main, take over as a new main and spawn a new backup.