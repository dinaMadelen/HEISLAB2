package distribution

import (
	"fmt"
	"net"
	"sort"
	"time"
)

type PeerUpdate struct {
	Peers []string
	New   string
	Lost  []string
}

const interval = 15 * time.Millisecond
const timeout = 500 * time.Millisecond

func peerTransmitter(port int, id string, transmitEnable <-chan bool) {
	conn := DialBroadcastUDP(port)
	addr, _ := net.ResolveUDPAddr("udp4", fmt.Sprintf("255.255.255.255:%d", port))
	enable := true

	for {
		select {
		case enable = <-transmitEnable:
		case <-time.After(interval):
		}
		if enable {
			_, err := conn.WriteTo([]byte(id), addr)
			if err != nil {
				fmt.Println("Error sending packet:", err)
			}
		}
	}
}

func peerReceiver(port int, peerUpdateCh chan<- PeerUpdate) {
	var buf [1024]byte
	var p PeerUpdate
	lastSeen := make(map[string]time.Time)

	conn := DialBroadcastUDP(port)

	for {
		updated := false

		err := conn.SetReadDeadline(time.Now().Add(interval))
		if err != nil {
			fmt.Println("Error setting read deadline:", err)
			continue
		}

		n, _, err := conn.ReadFrom(buf[0:])
		if err != nil {
			if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
				continue
			}
			fmt.Println("Error reading from UDP:", err)
			continue
		}

		id := string(buf[:n])

		p.New = ""
		if id != "" {
			if _, idExists := lastSeen[id]; !idExists {
				p.New = id
				updated = true
			}

			lastSeen[id] = time.Now()
		}

		p.Lost = make([]string, 0)
		for k, v := range lastSeen {
			if time.Now().Sub(v) > timeout {
				updated = true
				p.Lost = append(p.Lost, k)
				delete(lastSeen, k)
			}
		}

		if updated {
			p.Peers = make([]string, 0, len(lastSeen))

			for k, _ := range lastSeen {
				p.Peers = append(p.Peers, k)
			}

			sort.Strings(p.Peers)
			sort.Strings(p.Lost)
			peerUpdateCh <- p
		}
	}
}
