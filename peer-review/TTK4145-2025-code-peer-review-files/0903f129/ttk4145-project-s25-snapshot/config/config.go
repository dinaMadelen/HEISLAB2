package config

import (
	"time"
)

type ClearRequestVariant int

const (
	All ClearRequestVariant = iota
	InDir
)

const (
	NUM_FLOORS  				= 4
	NUM_BUTTONS 				= 3
	CLEAR_REQUEST_VARIANT 		= InDir
	PRIMARY_IP_PORT           	= 30001
	TCP_PORT                  	= 30002
	BROADCAST_PORT            	= 15657
	DOOR_OPEN_TIME           	= 3 * time.Second
	TRAVEL_TIME               	= 2 * time.Second
	PRIMARY_TRANSMIT_INTERVAL 	= 1 * time.Second
	PRIMARY_READ_DEADLINE     	= 2 * PRIMARY_TRANSMIT_INTERVAL
	TCP_CONNECTION_DEADLINE   	= 2 * time.Second
	BACKUP_HEARTBEAT_TIME     	= 20 * time.Second
)
