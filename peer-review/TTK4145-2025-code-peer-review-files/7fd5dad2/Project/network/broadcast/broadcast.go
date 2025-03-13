package broadcast

import (
	. "Project/dataenums"
	"Project/network/conn"
	"encoding/json"
	"fmt"
	"net"
	"sort"
	"time"
)

func Sender(port int, msgCh <-chan Message) {
	conn := conn.DialBroadcastUDP(port)
	addr, _ := net.ResolveUDPAddr("udp4", fmt.Sprintf("255.255.255.255:%d", port))

	for msg := range msgCh {
		jsonBytes, _ := json.Marshal(msg)
		if len(jsonBytes) > BufferSize {
			panic("Packet too large.")
		}
		conn.WriteTo(jsonBytes, addr)
	}
}

func Receiver(port int, myID string, messageCh chan<- Message, registryCh chan<- NetworkNodeRegistry) {
	lastSeen := make(map[string]time.Time)
	reportedNew := make(map[string]bool)
	var buf [BufferSize]byte

	conn := conn.DialBroadcastUDP(port)

	for {
		conn.SetReadDeadline(time.Now().Add(HeartbeatInterval))
		n, _, err := conn.ReadFrom(buf[:])
		if err != nil {
			if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
			} else {
				fmt.Println("Receiver error:", err)
			}
		} else {
			var msg Message
			if err := json.Unmarshal(buf[:n], &msg); err != nil {
				fmt.Println("Failed to unmarshal Message:", err)
			} else {
				lastSeen[msg.SenderId] = time.Now()
				if _, exists := reportedNew[msg.SenderId]; !exists {
					reportedNew[msg.SenderId] = false
				}
				if msg.SenderId != myID {
					messageCh <- msg
				}
			}
		}

		// Heartbeat check
		now := time.Now()
		var lostNodes, activeNodes, newNodes []string
		for id, t := range lastSeen {
			if now.Sub(t) > HeartbeatTimeout {
				lostNodes = append(lostNodes, id)
				delete(lastSeen, id)
				delete(reportedNew, id)
			} else {
				activeNodes = append(activeNodes, id)
			}
		}
		for _, id := range activeNodes {
			if !reportedNew[id] {
				newNodes = append(newNodes, id)
				reportedNew[id] = true
			}
		}
		if len(lostNodes) > 0 || len(newNodes) > 0 {
			sort.Strings(activeNodes)
			registryCh <- NetworkNodeRegistry{
				Nodes: activeNodes,
				New:   newNodes,
				Lost:  lostNodes,
			}
		}
	}
}
