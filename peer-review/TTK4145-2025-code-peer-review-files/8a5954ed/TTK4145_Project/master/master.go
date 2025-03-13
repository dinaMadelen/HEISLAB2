package master

import (
	"color"
	"config"
	"fmt"
	"log"
	"net"
	"network"
	"time"
)

var State = struct {
	Master, Backup string
}{
	"master", "backup",
}

type Master struct {
	Queue         *Master_queue
	Client_list   *Client_list
	Backup_addr   string
	Backup_exists bool
	Server_addr   string

	// Timers
	Distribution_timer *time.Timer
	Backup_check_timer *time.Timer
}

func Create_master(capacity int) *Master {
	return &Master{
		Queue:         Create_master_queue(capacity),
		Client_list:   Create_client_list(),
		Backup_addr:   "",
		Backup_exists: false,
		Server_addr:   "",
	}
}

type Backup struct {
	Queue	*Master_queue
	Conn    *net.TCPConn
	State   string
}

func Create_backup(capacity int) *Backup {
	return &Backup{
		Queue: 	Create_master_queue(capacity),
		Conn:	nil,
		State: 	State.Backup,
	}
}

//TODO: Latency of button reaction, (for loop in run single elevator) 
//			-> cant see anything in the loop that would block the switch case
//			-> maybe its the master that takes longer to send order? Can move timers to go routines to reduce utlization of main thread
//TODO: Fix logic of removing orders from queue according to spec (Only hallup order served on the way up)
//			-> maybe fixed, still strugelse when there is 3 consecutive hall_down/hall_up orders because of first order bias (stops at the second one)
//TODO: Find more bugs
//			-> sometimes the elevator freezes after disconnecting (or can take longer time)

func Run_master_backup() {
	backup := Create_backup(config.Num_elevators * config.Num_floors * 4)

	mst_newConn := make(chan *net.TCPConn)
	mst_connLoss := make(chan *net.TCPConn)
	mst_msgChan := make(chan network.Message)
	mst_taskTimeout := make(chan string)

	bck_connLoss := make(chan *net.TCPConn)
	bck_msgChan := make(chan network.Message)

	for {
		switch backup.State {
		case State.Master:
			fmt.Print(color.Blue + " --- MASTER --- \n" + color.Reset)
			master := Create_master(config.Num_elevators * config.Num_floors * 4)
			master.Queue = backup.Queue

			go network.Start_server(mst_connLoss, mst_newConn, mst_msgChan)
			go master.Client_list.Timer_handler(mst_taskTimeout)

			master.Distribution_timer = time.NewTimer(config.Distribution_delay)
			master.Backup_check_timer = time.NewTimer(config.Backup_check_delay)

			master.Server_addr = network.Find_server_address()
			// Handle distribution and backup check timers
			go master.Timer_manager()

			for {
				select {
				// New client
				case conn := <-mst_newConn:
					log.Printf(color.Green+"New connection: %s\n"+color.Reset, network.Get_addr_from_conn(conn))
					master.Client_list.Add(network.Get_addr_from_conn(conn), conn)

				// Loss of client
				case loss := <-mst_connLoss:
					log.Printf(color.Orange+"Lost connection: %s\n"+color.Reset, network.Get_addr_from_conn(loss))
					master.Handle_disconnect(loss)

				// New message from client
				case msg := <-mst_msgChan:
					log.Printf("New message %s from: %s\n", msg.Header, msg.Addr)
					master.Handle_new_messages(msg)

				// Client task timeout
				case addr := <-mst_taskTimeout:
					master.Handle_task_timeout(addr)
				}
			}

		case State.Backup:
			fmt.Print(color.Blue + " --- BACKUP --- \n" + color.Reset)
			backup.Connect_backup(config.Backup_Id, bck_msgChan, bck_connLoss)

			for backup.State == State.Backup {
				select {
				// Loss of connection to master
				case <-bck_connLoss:
					backup.Handle_connection_loss()

				// New message from master
				case msg := <-bck_msgChan:
					log.Print("New message from server \n ")
					backup.Handle_sync_message(msg)
				}
			}
		}
	}
}