package primary

import (
	"Project/config"
	"Project/elevator"
	"Project/network/conn"
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"
	"strings"
	"time"
)
// Primary is the main function for the primary module
func Primary(ctx context.Context, MyID string, assignOrder chan<- elevator.OrderUpdate) {
	go TransmitPrimaryID(MyID, ctx)
	go ConnectPrimaryToBackups(ctx, assignOrder)

	for range ctx.Done() {
		return
	}
}
// TransmitPrimaryID sends the primary ID to the network
func TransmitPrimaryID(ID string, ctx context.Context) {
	port := config.PRIMARY_IP_PORT
	conn := conn.DialBroadcastUDP(port)
	defer conn.Close() 
	addr, _ := net.ResolveUDPAddr("udp4", fmt.Sprintf("255.255.255.255:%d", port))

	ticker := time.NewTicker(config.PRIMARY_TRANSMIT_INTERVAL)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			conn.WriteTo([]byte(ID), addr)
		case <-ctx.Done():
			return
		}
	}
}
// ListenForBackups listens for incoming connections from backup elevators
func ListenForBackups(ctx context.Context, newConnCh chan<- net.Conn) {
	fmt.Println("Listening for backups")
	port := config.TCP_PORT
	listener, err := net.Listen("tcp", fmt.Sprintf(":%d", port))
	if err != nil {
		fmt.Println("Error starting TCP listener:", err)
		return
	}
	defer listener.Close()

	for {
		select {
		case <-ctx.Done():
			return
		default:
			conn, err := listener.Accept()
			if err != nil {
				fmt.Println("Error accepting connection:", err)
			} else {
				newConnCh <- conn
			}
		}
	}
}
// ConnectPrimaryToBackups connects the primary elevator to backup elevators
func ConnectPrimaryToBackups(ctx context.Context, assignOrder chan<- elevator.OrderUpdate) {
	var backupConnections = make(map[string]net.Conn)

	newConnCh := make(chan net.Conn) 
	go ListenForBackups(ctx, newConnCh)

	deleteConnCh := make(chan string)

	for {
		select {
		case conn := <-newConnCh:
			reader := bufio.NewReader(conn)
			done := make(chan bool)

			go func() {
				message, err := reader.ReadString('\n')
				if err != nil {
					fmt.Println("Error reading from connection:", err)
					conn.Close()
					done <- false
					return
				}

				peerID := strings.TrimSpace(message)
				backupConnections[peerID] = conn
				go ReceiveFromBackup(ctx, conn, peerID, deleteConnCh, assignOrder)
				done <- true
			}()

			select {
			case success := <-done:
				if success {
					fmt.Println("Connections: ", backupConnections)
				}
			case <-time.After(config.TCP_CONNECTION_DEADLINE):
				fmt.Println("Timeout reading from connection, closing it")
				conn.Close()
			}

		case peerID := <-deleteConnCh:
			if conn, exists := backupConnections[peerID]; exists {
				conn.Close()
				delete(backupConnections, peerID)
			}

		case <-ctx.Done():
			return
		}
	}
}
// ReceiveFromBackup receives messages from backup elevators
func ReceiveFromBackup(ctx context.Context, conn net.Conn, peerID string, deleteConnCh chan<- string, assignOrder chan<- elevator.OrderUpdate) {
	scanner := bufio.NewScanner(conn)
	for {
		select {
		case <-ctx.Done():
			fmt.Println("Context cancelled, closing connection to", peerID)
			conn.Close()
			deleteConnCh <- peerID
			return
		default:
			if scanner.Scan() {
				message := scanner.Text()
				var order elevator.OrderUpdate
				err := json.Unmarshal([]byte(message), &order)
				if err != nil {
					fmt.Println("Error unmarshalling order:", err)
					continue
				}
				assignOrder <- order
			} else {
				if err := scanner.Err(); err != nil {
					fmt.Println("Scanner error:", err)
				}
				fmt.Println("Closing connection to", peerID)
				conn.Close()
				deleteConnCh <- peerID
				return
			}
		}
	}
}
