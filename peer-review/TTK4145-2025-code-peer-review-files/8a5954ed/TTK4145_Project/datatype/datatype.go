package datatype

import (
	"elevio"
)

// Client types
var ClientType = struct {
	Elevator, Backup, Unknown string
}{
	"elevator", "backup", "unknown",
}

// Headers - describe the purpose of the message
var HeaderType = struct {
	OrderReceived, OrderFulfilled, UpdateFloor, UpdateObstruction, Sync, SyncConfirmation, ClientInfo, StartBackup string
}{
	"order_received", "order_fulfilled", "update_floor", "update_obstruction", "sync", "sync_confirmation", "client_info", "start_backup",
}

// Message payload - data corresponding to the header
type DataPayload struct {
	ClientType   string                 `json:"client_type,omitempty"`
	ID           string                 `json:"id,omitempty"`
	CurrentFloor int                    `json:"current_floor,omitempty"`
	Obstruction  bool					`json:"obstruction,omitempty"`
	OrderFloor   int                    `json:"order_floor,omitempty"`
	OrderButton  elevio.ButtonType      `json:"order_button,omitempty"`
	Queue        []Master_queue_element `json:"queue,omitempty"`
}

// Master queue element
type Master_queue_element struct {
	Floor        int
	Button       elevio.ButtonType
	ReceivedFrom string 			// ID of the elevator that received the order
	SentTo       string 			// ID of the elevator that the order is sent to
}

// Elevator queue element
type Queue_element struct {
	Floor  		int
	Button 		elevio.ButtonType
}
