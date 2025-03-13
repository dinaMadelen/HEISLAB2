package requests

import (
	"log"

	m "group48.ttk4145.ntnu/elevators/models"
)

func RunRequestServer(
	incomingRequests <-chan m.RequestMessage,
	peerStatus <-chan []m.Id,
	subscribers []chan<- m.Request) {

	var requestManager = newRequestManager()

	for {
		select {
		case msg := <-incomingRequests:
			log.Printf("[requests] Received request: %v", msg.Request)

			r := requestManager.process(msg)
			log.Printf("[requests] Processed request: %v", r)
			for _, s := range subscribers {
				s <- r
			}
		case alivePeers := <-peerStatus:
			log.Printf("[requests] Received alive peers: %v", alivePeers)

			requestManager.alivePeers = alivePeers
		}
	}
}

type requestManager struct {
	store      map[m.Origin]m.Request
	ledgers    map[m.Origin][]m.Id
	alivePeers []m.Id
}

func newRequestManager() *requestManager {
	return &requestManager{
		store:      make(map[m.Origin]m.Request),
		ledgers:    make(map[m.Origin][]m.Id),
		alivePeers: make([]m.Id, 0),
	}
}

func (rm *requestManager) process(msg m.RequestMessage) m.Request {
	if _, ok := rm.store[msg.Request.Origin]; !ok {
		rm.store[msg.Request.Origin] = msg.Request
	}

	switch msg.Request.Status {
	case m.Absent:
		return rm.processAbsent(msg)
	case m.Unconfirmed:
		return rm.processUnconfirmed(msg)
	case m.Confirmed:
		return rm.processConfirmed(msg)
	default:
		return msg.Request
	}
}

func (rm *requestManager) processAbsent(msg m.RequestMessage) m.Request {
	if msg.Request.Status != m.Absent {
		return msg.Request
	}

	storedRequest := rm.store[msg.Request.Origin]
	if storedRequest.Status == m.Confirmed || storedRequest.Status == m.Unknown {
		storedRequest.Status = m.Absent
	}

	rm.store[msg.Request.Origin] = storedRequest
	return storedRequest
}

func (rm *requestManager) processUnconfirmed(msg m.RequestMessage) m.Request {
	if msg.Request.Status != m.Unconfirmed {
		return msg.Request
	}

	ledgers := rm.ledgers[msg.Request.Origin]
	storedRequest := rm.store[msg.Request.Origin]

	switch storedRequest.Status {
	case m.Unknown:
		fallthrough
	case m.Absent:
		fallthrough
	case m.Unconfirmed:
		ledgers = append(ledgers, msg.Source)

		isConfirmed := isSetEqual(ledgers, rm.alivePeers)
		if isConfirmed {
			storedRequest.Status = m.Confirmed
			ledgers = make([]m.Id, 0)
		} else {
			storedRequest.Status = m.Unconfirmed
		}
	}

	rm.ledgers[msg.Request.Origin] = ledgers
	rm.store[msg.Request.Origin] = storedRequest
	return storedRequest
}

func (rm *requestManager) processConfirmed(msg m.RequestMessage) m.Request {
	if msg.Request.Status != m.Confirmed {
		return msg.Request
	}

	storedRequest := rm.store[msg.Request.Origin]
	storedRequest.Status = m.Confirmed

	rm.store[msg.Request.Origin] = storedRequest
	return storedRequest
}
