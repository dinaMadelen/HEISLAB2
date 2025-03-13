package comms

import (
	"log"
	"net"
	"time"

	"group48.ttk4145.ntnu/elevators/models"
)

const SendInterval = time.Millisecond * 100
const BroadcastAddr = "255.255.255.255"

// # RunComms runs the communication module
//
// It listens for updates on the local elevator state and validated requests channels.
// It send UDP messages with the local elevator state and validated requests to the broadcast address in a regular interval.
// It listens for incoming UDP messages and sends the elevator state and requests to the outgoing channels.
// It sends a health monitor ping on the health monitor ping channel when it receives an update from the local elevator state or validated requests channels.
func RunComms(
	localPeer models.Id,
	local net.IPAddr,
	port uint16,
	localElevatorUpdates <-chan models.ElevatorState,
	internalValidatedRequests <-chan models.Request,
	outgoingEStatesUpdates chan<- models.ElevatorState,
	outgoingUnvalidatedRequests chan<- models.RequestMessage,
	healthMonitorPing chan<- models.Id) {
	var validatedRequestsBuffer = make(map[models.Origin]models.Request)
	var internalEState models.ElevatorState
	var sendTicker = time.NewTicker(SendInterval)

	var receiveUdp = make(chan udpMessage)
	ra := net.UDPAddr{IP: local.IP, Port: int(port)}
	go RunUdpReader(receiveUdp, ra)

	var sendUdp = make(chan udpMessage)
	sa := net.UDPAddr{IP: net.ParseIP(BroadcastAddr), Port: int(port)}
	go RunUdpWriter(sendUdp, sa)

	for {
		select {
		case eState := <-localElevatorUpdates:
			log.Printf("[comms] Received local elevator state update: %v", eState)
			internalEState = eState
		case request := <-internalValidatedRequests:
			log.Printf("[comms] Received validated request: %v", request)
			validatedRequestsBuffer[request.Origin] = request
		case <-sendTicker.C:
			u := udpMessage{Source: localPeer, EState: internalEState, Requests: convert(validatedRequestsBuffer)}
			sendUdp <- u
		case msg := <-receiveUdp:
			if msg.Source == localPeer {
				continue
			}

			healthMonitorPing <- msg.Source
			outgoingEStatesUpdates <- msg.EState
			for _, r := range msg.Requests {
				outgoingUnvalidatedRequests <- models.RequestMessage{Source: msg.Source, Request: r}
			}
		}
	}
}

// convert converts a map of requests to a slice of requests
func convert(m map[models.Origin]models.Request) []models.Request {
	var requests = make([]models.Request, 0)
	for _, r := range m {
		requests = append(requests, r)
	}
	return requests
}
