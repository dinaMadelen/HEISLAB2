P2P
---

The Peer 2 Peer module provides an automatically detecting and connecting network that uses UDP to detect peers and TCP to speak to peers.
Additionally the P2P module provides idempotency and dependency ordering. (DAG ordering)


Lamport Clock
---
A struct that implements a Lamport Clock. Uses and upper and lower edge to wrap around should max integer value be reached.

Dependency
---
A struct that holds a sender and a lamport clock. Which essentially provides an unique id to a message.
(See definition of a Lamport Clock.)

Dependency-Handler
---
Checks if we have seen a dependency or not before. Used to check idempotency.

Dependency-Resolver
---
Stores sent messages, and resends messages upon request from others. Used to ensure dependency ordering and almost-guaranteed sending.

P2P_Message
---
A struct created using P2P_Network.Create_Message(message, type).
Contains the Sender, Message, LamportClock, Type and eventual dependency added by message_to_send.Depend_On(message_received)

P2P_Network
---
Provides the network functionality when created using New_P2P_Network().
Will automatically connect to other nodes on the same network. And uses Dependency-Handler and Dependency-Resolver to handle idempotency and dependency ordering.
