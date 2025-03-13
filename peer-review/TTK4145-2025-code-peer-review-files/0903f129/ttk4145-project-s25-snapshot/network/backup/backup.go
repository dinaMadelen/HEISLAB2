package backup

import (
	"Project/config"
	"Project/elevator"
	"Project/network/conn"
	"Project/network/localip"
	"context"
	"encoding/json"
	"fmt"
	"net"
)

// Backup is the main function for the backup module
// It listens for the primary ID and connects to the primary
// It also listens for heartbeats from the primary
func Backup(ctx context.Context, MyID string, primID string, PrimaryIdCh <-chan string) {
	primaryID := primID
	var primaryConn net.Conn
	var err error
	// Attempt initial connection
	primaryConn, err = ConnectBackupToPrimary(MyID, primaryID)
	if err != nil {
		fmt.Println("Error connecting to primary:", err)
	}

	//heartbeatTimer := time.NewTimer(config.BACKUP_HEARTBEAT_TIME)

	for {
		fmt.Printf("Primary ID: %v", primaryID)
		select {
		case primaryID = <-PrimaryIdCh:
			// Close the old connection before switching
			DestroyTCPConnection(&primaryConn)

			// Reconnect to the new primary
			primaryConn, err = ConnectBackupToPrimary(MyID, primaryID)
			if err != nil {
				fmt.Println("Error connecting to new primary:", err)
			}
			//heartbeatTimer.Reset(config.BACKUP_HEARTBEAT_TIME)

		// case <-heartbeatTimer.C:
		// 	fmt.Print("sending heartbeat now")
		// 	SendHeartBeat(&primaryConn, primaryID, MyID)
		// 	heartbeatTimer.Reset(config.BACKUP_HEARTBEAT_TIME)

		case <-ctx.Done():
			fmt.Println("I am done being backup")
			DestroyTCPConnection(&primaryConn)
			return
		}
	}
}

func SendHeartBeat(conn *net.Conn, primaryID string, MyID string) {
	if *conn != nil {
		_, err := fmt.Fprintf(*conn, "%s\n", MyID)
		if err != nil {
			fmt.Println("Error sending heartbeat:", err)
			DestroyTCPConnection(conn)
			*conn = nil                
		}
	} else {
		// Attempt to reconnect
		newConn, _ := ConnectBackupToPrimary(MyID, primaryID)
		*conn = newConn // Assign new connection
	}
}
// ConnectBackupToPrimary connects the backup to the primary
// It sends the backup ID to the primary
func ConnectBackupToPrimary(MyId string, primaryID string) (net.Conn, error) {

	primaryIP, _ := localip.IdToIp(primaryID)
	if primaryIP == "" {
		return nil, fmt.Errorf("there is no primary")
	}

	conn, err := net.DialTimeout("tcp", fmt.Sprintf("%s:%d", primaryIP, config.TCP_PORT), config.TCP_CONNECTION_DEADLINE)

	if err != nil {
		fmt.Println("Error connecting to primary:", err)
		return nil, err
	}

	fmt.Fprintf(conn, "%s\n", MyId)
	fmt.Print("Sent MyID to Primary")
	return conn, nil
}
// DestroyTCPConnection closes the TCP connection to the primary
func DestroyTCPConnection(primaryConn *net.Conn) {
	if (*primaryConn) != nil {
		(*primaryConn).Close()
	}
}


var tcpConn net.Conn 
var TcpConnReady = make(chan struct{}, 1)

// Receiver listens for UDP broadcasts on the specified port
// It also establishes a TCP connection to the primary
func Receiver(port int, ipChan chan string) {
	udpConn := conn.DialBroadcastUDP(port)
	if udpConn == nil {
		fmt.Println("Error establishing UDP connection")
		return
	}
	defer udpConn.Close()

	var buf [1024]byte
	for {
		n, addr, err := udpConn.ReadFrom(buf[:])
		if err != nil {
			fmt.Printf("Receiver: error reading from UDP: %v\n", err)
			continue
		}

		msg := string(buf[:n])
		fmt.Printf("Receiver: got message from %v: %s\n", addr, msg)
		ipChan <- msg

		// Hvis vi ikke har en tilkobling, opprett den
		if tcpConn == nil {
			primaryIP := msg
			primaryAddr := fmt.Sprintf("%s:%d", primaryIP, config.TCP_PORT)
			primaryTCPAddr, err := net.ResolveTCPAddr("tcp", primaryAddr)
			if err != nil {
				fmt.Println("Error resolving TCP address:", err)
				continue
			}
			tcpConn, err = net.DialTCP("tcp", nil, primaryTCPAddr)
			if err != nil {
				fmt.Println("Error dialing TCP connection:", err)
				continue
			}
			select {
			case TcpConnReady <- struct{}{}:
			default:
			}
		}
	}
}
// SendOrderToPrimary sends an order to the primary through the TCP connection
func SendOrderToPrimary(Order elevator.OrderUpdate) (bool, error) {
	fmt.Printf("Sending order: %+v\n", Order)
	if tcpConn == nil {
		fmt.Println("No connection to primary")
		return false, nil
	}
	// Send order to primary
	order, err := json.Marshal(Order)
	if err != nil {
		fmt.Println("Error marshalling order:", err)
		return false, err
	}
	_, err = fmt.Fprintln(tcpConn, string(order))
	if err != nil {
		fmt.Println("Error sending order to primary:", err)
		return false, err
	}
	return true, nil
}
