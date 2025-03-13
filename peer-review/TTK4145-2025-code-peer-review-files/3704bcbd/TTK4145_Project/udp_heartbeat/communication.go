package udp_heartbeat

import (
	"fmt"
	"net"
	"strings"
	"sync"
	"time"
)

// Node represents a tracked computer
type Node struct {
	LastSeen time.Time
	Active   bool
}

var (
	nodes = make(map[string]*Node) // Tracks all known computers
	mu    sync.Mutex               // Ensures safe concurrent access to nodes
)

type NodeStatus struct {
	Alive_nodes map[string]bool
	Mu_an       sync.Mutex
}

// Broadcast_alive sends a heartbeat message on the network
func Broadcast_alive(subnet_ip string, broadcast_port string, message string, alive_frequency int) {
	// Resolve the UDP address
	udpAddr, err := net.ResolveUDPAddr("udp", subnet_ip+"255:"+broadcast_port)
	if err != nil {
		fmt.Println("Error resolving address:", err)
		return
	}

	// Create a UDP connection
	conn, err := net.DialUDP("udp", nil, udpAddr)
	if err != nil {
		fmt.Println("Error creating UDP connection:", err)
		return
	}
	defer conn.Close()

	// Enable broadcast
	err = conn.SetWriteBuffer(1024)
	if err != nil {
		fmt.Println("Error setting write buffer:", err)
		return
	}

	// Send the message periodically
	for {
		_, err := conn.Write([]byte(message))
		if err != nil {
			fmt.Println("Error sending message:", err)
		}
		time.Sleep(time.Duration(1/alive_frequency) * time.Second) // Adjust the interval as needed
	}
}

// Listen_broadcast listens for UDP messages and tracks nodes
func Listen_broadcast(subnet_ip string, broadcast_port string, own_id string) {
	// Resolve the UDP address
	udpAddr, err := net.ResolveUDPAddr("udp", subnet_ip+"255:"+broadcast_port)
	if err != nil {
		fmt.Println("Error resolving address:", err)
		return
	}

	// Create a UDP connection
	conn, err := net.ListenUDP("udp", udpAddr)
	if err != nil {
		fmt.Println("Error starting UDP server:", err)
		return
	}
	defer conn.Close()

	// Goroutine to handle incoming messages and update node status
	// go trackNodeStatus()

	// Listen for incoming broadcasts
	buffer := make([]byte, 1024)
	for {
		n, _, err := conn.ReadFromUDP(buffer)
		if err != nil {
			fmt.Println("Error receiving UDP packet:", err)
			continue
		}

		receivedID := strings.TrimSpace(string(buffer[:n]))

		// Update node's last seen time or add it to the list
		mu.Lock()
		if node, exists := nodes[receivedID]; exists {
			node.LastSeen = time.Now()
			node.Active = true
		} else {
			nodes[receivedID] = &Node{LastSeen: time.Now(), Active: true}
		}
		mu.Unlock()
	}
}

// Periodically checks and updates node statuses
func TrackNodeStatus(nodeStatus *NodeStatus, timeoutDuration time.Duration) {
	for {
		time.Sleep(500 * time.Millisecond) // Check every 0.5 seconds

		mu.Lock()
		for id, node := range nodes {
			if time.Since(node.LastSeen) > timeoutDuration {
				node.Active = false // Mark as inactive
			}
			nodeStatus.Mu_an.Lock()
			(nodeStatus.Alive_nodes)[id] = node.Active
			nodeStatus.Mu_an.Unlock()

		}
		mu.Unlock()
	}
}
