TCP
---

The TCP module provides a TCP_Connection and a TCP_Connection_Manager struct
The intended use case is to use TCP Connection Manager and not use TCP Connection because the manager creates connections using Open_Server() and Connect_Client(), though the option exists to directly connect a connection you have made yourself.

Used by
---
P2P to broadcast messages and ensure idempotency and dependency-ordering by checking against addresses.

TCP Connection Manager
---

Provides a way of organizing multiple TCP Connections.

You may
- Add a TCP Connection
- Remove a TCP Connection
- Open a Server (and receive the Server IP:Port)
- Connect a client to address
- Broadcast
- Close all channels
- Send to an individual connection (if you know it's IP:Port)
- Ask if an address is connected

TCP Connection
---

Provides a Read and Write channel to a connection made either by a server or a client.
It is made using the New_TCP_Connection function.
The Close() function closes the reader and writer made by TCP Connection Manager.


Split Handler
---

The Split Handler handles the act of making a null terminated string and making sure messages are split according to null termination from the TCP message.
This is needed by the TCP module since TCP can send message split in half, either due to poor read moment or due to max message size reached.