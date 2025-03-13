package config

import (
	"time"
)

// Master
const Num_elevators = 3
const Task_period = (Num_floors * 2) * time.Second
const Distribution_delay = 15 * time.Second
const Backup_check_delay = 20 * time.Second
const Backup_filename = "run_master.go"
const Backup_Id = "101"

// Elevator
const Num_floors = 4
const Door_open_duration = 3 * time.Second
const Reconnect_delay = 5 * time.Second

// Networking
const UDP_port = 60000
const TCP_port = 30000
const Broadcast_delay = 1 * time.Second
const Keep_alive_period = 250 * time.Millisecond
