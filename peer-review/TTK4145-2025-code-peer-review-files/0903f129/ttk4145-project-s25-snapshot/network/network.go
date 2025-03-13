package network

import (
	"Project/config"
	"Project/elevator"
	"Project/network/backup"
	"Project/network/conn"
	"Project/network/primary"
	"context"
	"fmt"
	"math/rand"
	"time"
)
// GetPrimaryId returns the ID of the primary elevator
func GetPrimaryId() string {
	var buf [1024]byte

	conn := conn.DialBroadcastUDP(config.PRIMARY_IP_PORT)
	defer conn.Close()
	conn.SetReadDeadline(time.Now().Add(config.PRIMARY_READ_DEADLINE))
	n, _, err := conn.ReadFrom(buf[0:])
	if err != nil {
		fmt.Println("Error reading from Primary:", err)
		return ""
	}
	return string(buf[:n])
}

type state int

const (
	primary_state = 1
	backup_state  = 2
	offline_state = 3
)

var my_state state

// PrimaryBackupNetwork is the main function for the network module
// It starts the primary and backup modules and distributes orders
// between them
func PrimaryBackupNetwork(MyID string, 
						localOrderRequest <-chan elevator.Order, 
						addToLocalQueue chan<- elevator.Order, 
						assignOrder chan<- elevator.OrderUpdate) {
	my_state = backup_state
	go DistributeOrders(localOrderRequest, addToLocalQueue, assignOrder)

	var primaryCancel context.CancelFunc = nil
	var backupCancel context.CancelFunc = nil

	primaryIDCh := make(chan string)
	lastPrimary := ""

	ticker := time.NewTicker(300 * time.Millisecond)
	defer ticker.Stop()
	for range ticker.C {
		primaryId := GetPrimaryId()

		switch my_state {

		case primary_state:
			if backupCancel != nil {
				backupCancel()
				backupCancel = nil
			}
			if primaryCancel == nil {
				ctx, cancel := context.WithCancel(context.Background())
				primaryCancel = cancel
				go primary.Primary(ctx, MyID, assignOrder)
			}

			if primaryId != MyID && primaryId != "" {
				fmt.Println("Another Primary detected with ID: ", primaryId)
				if primaryId > MyID {
					my_state = backup_state
				} else {
					fmt.Println("I am staying Primary")
				}
			}
		case backup_state:
			if primaryCancel != nil {
				primaryCancel()
				primaryCancel = nil
			}

			if primaryId == "" {
				delay := time.Duration(rand.Intn(300)) * time.Millisecond
				fmt.Printf("No Primary detected, waiting for %v until I take over.\n", delay)
				time.Sleep(delay)
				primaryId = GetPrimaryId()
				if primaryId == "" {
					fmt.Println("No Primary detected after delay, I am now Primary")
					my_state = primary_state
				}
			} else {
				if backupCancel == nil {
					ctx, cancel := context.WithCancel(context.Background())
					backupCancel = cancel
					go backup.Backup(ctx, MyID, primaryId, primaryIDCh)
					lastPrimary = primaryId
				} else {
					if lastPrimary != primaryId {
						primaryIDCh <- primaryId
						lastPrimary = primaryId
					}
				}
			}
		}
	}
}

// DistributeOrders distributes orders between the primary and backup elevators
func DistributeOrders(localOrderRequest <-chan elevator.Order, 
					addToLocalQueue chan<- elevator.Order, 
					assignOrder chan<- elevator.OrderUpdate) {
	for {
		order := <-localOrderRequest                                                   
		newOrder := elevator.OrderUpdate{Floor: order.Floor, 
										Button: order.Button, 
										Served: false} 				// New order to be assigned
		if order.Button == elevator.BT_Cab {
			fmt.Println("Cab Call: Adding to local queue")
			addToLocalQueue <- order 								// Add order to local queue
		} else {
			if my_state == primary_state {
				assignOrder <- newOrder 							// If I am primary, assign order
			} else {
				backup.SendOrderToPrimary(newOrder) 				// If I am backup, send order to primary
			}
		}
	}
}
