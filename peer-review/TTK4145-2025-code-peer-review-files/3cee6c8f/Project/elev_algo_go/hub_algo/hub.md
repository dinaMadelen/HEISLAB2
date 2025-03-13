# Distributed “Hub + Backup” Elevator System

This document outlines how to build a distributed elevator system where **each** elevator hosts **two** processes:


## 1. Overview
This code implements a distributed elevator coordinator with three **roles**:

- **Active** (Main Hub): One node that processes incoming orders and assigns them to elevators.
- **Backup**: A designated standby that monitors the Active hub and takes over if it fails.
- **Idle**: Any other node not currently Active or Backup.

Each node runs:
- A **finite-state machine** (FSM) described in the `hub_fsm` package.
- **Heartbeat**-based detection to decide who is Active, who is Backup, etc.
- **Order distribution** logic that uses an external `HRA` (Hall Request Assigner) to decide which elevator handles each hall request.

**Key**: The system only guarantees correctness if only **one** node fails at a time. Double-faults or partitions can produce multiple Active hubs.

---

## 2. Package Structure

1. **`hub_fsm`** (this file)
   - Holds the main FSM methods: `Init`, `OnReceivedBtnPress`, `OnHeartbeat`, `OnUpdateWorldView`, `DistributeOrder`, etc.
   - Has global variables tracking current role/state (`hubElement.State`), IP addresses, last-seen heartbeat timestamps, etc.
2. **`hub`** package
   - Defines structs like `Hub`, `WorldView`, `ButtonEvent`, `Heartbeat`, and enumerations for roles.
   - Contains basic initialization (`hub.Initialize()`).
3. **`hra`** package
   - The “Hall Request Assigner,” which is called to determine which elevator should handle a given hall request.
4. **`hub_network`** package
   - Responsible for low-level network I/O, e.g., sending heartbeats, receiving them, distributing orders to a specific elevator.

---

## 3. Core Global Variables

- **`hubElement hub.Hub`**: Holds the local node’s data, including `State` (`Idle`, `Backup`, `Active`) and an array of `WorldView` entries for up to 3 elevators.
- **`backupWorldView hub.WorldView`**: A snapshot used to compare against or update the backup’s data.
- **`localIP, activeIP, backupIP, idleIP`**: Strings to store IP addresses that identify who’s who.
- **`lastSeen`**: A map `[string]time.Time` storing the last time we received a heartbeat from each known node.
- **`TIMEOUT`**: A constant, `2s`, controlling how long we wait before assuming a node is dead.

---

## 4. Initialization

### `func Init(IP string)`

1. Calls `hub.Initialize()` to set up internal Hub data.
2. Sets `localIP = IP`.
3. Calls `ElectActive()` to discover if there’s an existing Active hub or if we should become Active.
4. Prints "Hub FSM initialized!".

### `func ElectActive()`

1. Retrieves the local IP (again) and marks this node as **Idle**.
2. Starts a short 2-second discovery window by sending out idle heartbeats (`InitByHeartbeats`) and collecting inbound heartbeats.
3. If it discovers an existing Active, we remain **Idle**.
4. Otherwise, if only Idle nodes are found, the **lowest IP** becomes Active.
5. If we become **Active**, we call `pickNewBackup()`.

---

## 5. Handling Orders and World Views

### `func OnReceivedBtnPress(BtnPress hub.ButtonEvent)`

- Triggered whenever this node (if Active) receives a hall request.
- If we’re **Backup** (or Idle), we do nothing special.
- If we’re **Active**:
  1. Call `DistributeOrder(BtnPress)` to assign the request.
  2. Call `broadcastOrder(...)` to forward it to the chosen elevator until the elevator’s `WorldView` reflects that order.

### `func OnUpdateWorldView(worldview hub.WorldView)`

- Updates our internal `hubElement.WVs[]` to incorporate new hall requests or elevator state from the sending node.
- Merges the changed `HallRequests` and `SenderElevState` fields.

---

## 6. Distributing a Single Order

### `func DistributeOrder(ev hub.ButtonEvent) (string, hub.ButtonEvent)`

1. Converts the single `ButtonEvent` to a minimal hall-request matrix using `SingleButtonEventToMatrix`.
2. Builds an `hra.HRAInput` from that matrix + elevator states (`CollectHRAElevStates`).
3. Calls `hra.Hra(input)`, which returns a map from `elevatorID -> [][2]bool`.
4. Looks up who got assigned for `(ev.Floor, ev.Button)` with `findAssignedElevator(...)`.
5. Returns the chosen elevator’s IP plus the same `ButtonEvent`.

### `func broadcastOrder(elevatorIP string, event hub.ButtonEvent)`

- Repeatedly sends the order to the designated elevator (via `hub_network.SendOrder`) until we see that elevator’s `HallRequests[event.Floor][event.Button]` is `true` in our local copy of `hubElement.WVs`.
- Exits once the elevator’s state is updated.

---

## 7. Heartbeat Handling and Failover

### `func OnHeartbeat(hb hub.Heartbeat)`

- Updates `lastSeen[hb.IP] = time.Now()`.
- Depending on the **local** role (`Active`, `Backup`, or `Idle`), checks if we lost contact with our counterpart:
  - **Active**:
    - If the known `backupIP` hasn’t been seen for 2s, call `pickNewBackup()`.
  - **Backup**:
    - If the known `activeIP` hasn’t been seen for 2s, call `becomeActive()` to take over.
  - **Idle**:
    - By default, do nothing.
- Additional logic sets `activeIP` if we see a valid heartbeat instructing us to become Idle or if the sender is recognized as Active.

### `func pickNewBackup()`

- Chooses an Idle node to become the new Backup (here stored in `idleIP`).
- Sends a heartbeat instructing that node to adopt **Backup**.

### `func becomeActive()`

- Sets `hubElement.State = hub.Active`, `activeIP = localIP`, and calls `pickNewBackup()`.

---

## 8. Utility Methods

### `func SingleButtonEventToMatrix(ev hub.ButtonEvent, numFloors int) [][2]bool`
- Creates a 2D array of `[floor][2]bool`, setting exactly `(ev.Floor, Up/Down)` to `true`. Ignores `BT_Cab`.

### `func CollectHRAElevStates(wvs [3]hub.WorldView) map[string]hra.HRAElevState`
- Builds a map from `IP -> HRAElevState` so the HRA can figure out each elevator’s floor, direction, etc.

### `func handleInitHeartbeat(...)` (during `ElectActive()`)
- Called in the 2s “init window” to record discovered IPs. If we see an **Active** node, we set `discoveredActive = true`.

### `func GetState() int`
- Exposes `hubElement.State` for external modules.

---

## 9. Run-Time Flow Summary

1. **Init**
   - `Init(...)` sets up local IP, calls `ElectActive()`, which either finds an existing Active or becomes Active if we have the lowest IP.
2. **Heartbeat**
   - `OnHeartbeat(...)` keeps track of the last-seen times for each hub and triggers failover if the counterpart times out.
3. **Backup**
   - If we’re **Backup**, we watch `activeIP`; if it vanishes for 2s, we do `becomeActive()`.
4. **Active**
   - If we’re **Active** and see no backup or the backup is timed out, we do `pickNewBackup()`.
5. **Order Handling**
   - If we’re **Active**, `OnReceivedBtnPress(...)` → `DistributeOrder(...)` → `broadcastOrder(...)`.  If we’re not Active, we ignore the button event.
6. **World View Updates**
   - Elevator nodes or backups can call `OnUpdateWorldView(...)` to keep the main hub in sync. The main hub merges these changes.
