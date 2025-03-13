package node

import (
	"ElevatorProject/network/conn"
	"fmt"
	"net"
	"sort"
	"time"
)

type NodeUpdate struct {
	Master string
	Slaves []string
	New    string
	Lost   []string
}

const interval = 15 * time.Millisecond
const timeout = 500 * time.Millisecond

// Transmit function
func Transmitter(port int, id string, transmitEnable <-chan bool) {

	conn := conn.DialBroadcastUDP(port)
	addr, _ := net.ResolveUDPAddr("udp4", fmt.Sprintf("255.255.255.255:%d", port))

	enable := true
	for {
		select {
		case enable = <-transmitEnable:
		case <-time.After(interval):
		}
		if enable {
			conn.WriteTo([]byte(id), addr)
		}
	}
}

// Receive function
func Receiver(port int, nodeUpdateCh chan<- NodeUpdate) {

	var buf [1024]byte
	var node NodeUpdate
	lastSeen := make(map[string]time.Time)

	conn := conn.DialBroadcastUDP(port)

	for {
		updated := false

		conn.SetReadDeadline(time.Now().Add(interval))
		n, _, _ := conn.ReadFrom(buf[0:])

		id := string(buf[:n])

		// Adding new connection
		node.New = ""
		if id != "" {
			if _, idExists := lastSeen[id]; !idExists {
				node.New = id
				updated = true
			}

			lastSeen[id] = time.Now()
		}

		// Removing dead connection
		node.Lost = make([]string, 0)
		for k, v := range lastSeen {
			if time.Since(v) > timeout {
				updated = true
				node.Lost = append(node.Lost, k)
				delete(lastSeen, k)
			}
		}

		// Sending update
		if updated {
			node.Master, node.Slaves = electMaster(lastSeen)
			nodeUpdateCh <- node
		}
	}
}

func electMaster(lastSeen map[string]time.Time) (string, []string) {
	var ids []string
	for id := range lastSeen {
		ids = append(ids, id)
	}
	sort.Strings(ids)

	if len(ids) == 0 {
		return "", []string{}
	}

	master := ids[0]
	slaves := ids[1:]
	return master, slaves
}
