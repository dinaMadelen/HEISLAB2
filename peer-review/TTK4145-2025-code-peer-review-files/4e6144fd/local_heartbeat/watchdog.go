package local_heartbeat

// Implements a dual system communicating existence to each other.
// 3 missed heartbeats and assume program is hung -> PKill -> Takeover -> Spawn new watchdog.
