package master

import (
	"config"
	"datatype"
	"fmt"
	"log"
	"net"
	"network"
	"os/exec"
	"runtime"
	"time"
	"color"
	"os"
)

func Start_backup() {
	if runtime.GOOS == "windows" {
		exec.Command("cmd", "/C", "start", "powershell", "go", "run", config.Backup_filename).Run()
	} else if runtime.GOOS == "linux" {
		exec.Command("gnome-terminal", "--", "go", "run", config.Backup_filename).Run()
	} else {
		log.Fatalf(color.Red + "Not supported os. \n" + color.Reset)
	}
}

// Master loop functions:

// Send message to start backup at first client that doesnt have the same ip as server
func (m *Master) Request_backup() {
	m.Backup_check_timer.Reset(config.Backup_check_delay)
	if m.Client_list.Length() > 0 {
		conn := m.Client_list.Choose_backup_host(m.Server_addr)
		if conn != nil {
			network.Send_message(conn,
								 datatype.HeaderType.StartBackup,
								 datatype.DataPayload{})
		}
	} else {
		log.Print(color.Red + "Cant spawn backup, no clients. \n" + color.Reset)
	}
}

func (m *Master) Sync_to_backup() {
	if m.Backup_exists {
		network.Send_message(m.Client_list.Get(m.Backup_addr).Conn,
							 datatype.HeaderType.Sync,
							 datatype.DataPayload{Queue: m.Queue.Orders})
	}
}

// Check if backup is connected
func (m *Master) Update_backup_info() {
	for _, client := range m.Client_list.Clients {
		if client.Client_type == datatype.ClientType.Backup {
			m.Backup_exists = true
			m.Backup_addr = network.Get_addr_from_conn(client.Conn)
			return
		}
	}
	m.Backup_exists = false
	m.Backup_addr = ""
}

// Backup loop functions:

// Connect backup to server and send client information
func (b *Backup) Connect_backup(id string, msgChan chan<- network.Message, connLoss chan<- *net.TCPConn) {
	time.Sleep(2 * time.Second)
	serverAddr := network.Find_server_address()
	if serverAddr != "" {
		b.Conn = network.Connect_to_server(serverAddr)
		fmt.Printf(color.Green + "Connected to server: %s\n" + color.Reset, network.Get_addr_from_conn(b.Conn))
		network.Send_message(b.Conn,
							 datatype.HeaderType.ClientInfo,
							 datatype.DataPayload{ClientType: datatype.ClientType.Backup,
								                  ID:         id})
		go network.Listen_for_message(b.Conn, msgChan, connLoss)
	} else {
		b.State = State.Master
	}
}

func (b *Backup) Handle_connection_loss() {
	if network.Ping_google() {
		b.State = State.Master
	} else {
		os.Exit(0)
	}
}

func (b *Backup) Handle_sync_message(msg network.Message) {
	if msg.Header == datatype.HeaderType.Sync {
		// Update backup queue
		b.Queue.Orders = msg.Payload.Queue
		b.Queue.Print()
		// Send confirmation to server
		network.Send_message(b.Conn,
							 datatype.HeaderType.SyncConfirmation,
							 datatype.DataPayload{Queue: b.Queue.Orders})
	}
}
