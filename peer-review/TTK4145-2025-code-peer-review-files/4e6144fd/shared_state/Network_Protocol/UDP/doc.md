UDP
---

The UDP module provides an UDP_Channel struct made by New_UDP_Channel and provides a read and write go channel that can be used by the UDP_Channel owner.
When making the channel via New_UDP_Channel a server and a client are automatically created listening/sending to Constants.UDP_BROADCAST_IP_PORT.

Close() closes both the server and client
Broadcast() is an alias for just using the write channel.

Used by
---
P2P to detect and inform other peers, and connect to them via TCP.
