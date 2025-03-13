Network
---
Currently not working. :smile:

The Network Module provides the shared state update consensus functionality using the Synchronization protocol to join the network or remove disconnected nodes from the network.
Uses the Synchronization protocol to ensure agreement on the shared state should they disagree for any reason.
Uses the Update protocol to ensure everyone updates a field to a new value at the same time.

Can only perform 1 protocol at a time, and will abort any incoming protocols while performing one.


Node
---
Provides the functionality described above.

Listener
---
A node that does not join the network, and only listens to the protocols holding it's own shared state and informing the terminal directly of the shared state.
Mainly used for testing and observation of the system. Not to be used as part of the system.