package request

import (
	"fmt"
	"time"
	"project/sanntid/elevator/elevio"
	"project/sanntid/network/peers"
)

type Request struct {
	Button			elevio.ButtonEvent
	TimeAlive		time.Time
	Status			int
	Barrier			[]string
}

type CabRequests	map[string]map[string]Request
type HallRequests	map[string]Request

type HallRequestsTransmit struct {
	Id				string
	Requests		HallRequests
}

type CabRequestsTransmit struct {
	Id				string
	Requests		CabRequests
}

func InitCabRequests(id string, numFloors int) CabRequests {
	cabRequests := make(CabRequests)

	var button_type elevio.ButtonType = elevio.BT_Cab

	for i := 0; i < numFloors; i++ {
		event := elevio.ButtonEvent{
			Floor:		i,
			Button:		button_type,
		}

		eventString := elevio.BeToString(event)

		cabRequests[id][eventString] = Request{
			Button:			event,
			TimeAlive:		time.Now(),
			Status:			0,
			Barrier:		make([]string, 0),
		}
	}

	fmt.Println("Cab requests:")
	for key, value := range cabRequests[id] {
		fmt.Println(key, ":")
		fmt.Println(value.Button)
	}

	return cabRequests
}

func InitHallRequests(numFloors int) HallRequests {
	hallRequests := make(HallRequests)

	button_type := []elevio.ButtonType{elevio.BT_HallUp, elevio.BT_HallDown}

	for i := 0; i < numFloors; i++ {
		for _, b_type := range button_type {
			event := elevio.ButtonEvent {
				Floor:			i,
				Button:			b_type,
			}

			eventString := elevio.BeToString(event)

			hallRequests[eventString] = Request{
				Button:		event,
				TimeAlive:	time.Now(),
				Status:		0,
				Barrier:	make([]string, 0),
			}
		}
	}

	fmt.Println("Hall requests:")
	for key, value := range hallRequests {
		fmt.Println(key, ":")
		fmt.Println(value.Button)
	}

	return hallRequests
}

func RequestsHandler(id string, hallReqTx chan HallRequestTransmit,
					hallReqRx chan HallRequestTransmit,
					reqAssigner chan Request,
					reqUpdate chan Request,
					peerRx chan peers.PeerUpdate) {

	hallRequests := HallRequestsTransmit{
		Id:			id,
		Requests:	InitHallRequests(4)
	}

	cabRequests := CabRequestsTransmit{
		Id:			id,
		Requests:	InitCabRequests(id, 4)
	}

	// Channels
    drv_buttons := make(chan elevio.ButtonEvent)

	// Start button poll
	go elevio.PollButtons(drv_buttons)

	var peers_alive []string

	updateTicker := time.NewTicker(50 * time.Millisecond)
	defer updateTicker.Stop()

	displayTicker := time.NewTicker(4 * time.Second)
	defer displayTicker.Stop()

	for {
		select {
			case <-updateTicker.C:
				Tx <- hallRequests

			case <-displayTicker.C:
				fmt.Println("-----------------------")
				for key, value := range(hallRequests.Requests) {
					if value.Status != 0 {
						fmt.Println(key, ":")
						fmt.Println(value.Status)
					}
				}
				fmt.Println("-----------------------")

			case buttonEvent := <-drv_buttons:
				buttonEventString := elevio.BeToString(buttonEvent)
				fmt.Println(buttonEventString)

				if localRequests.Requests[buttonEventString].Status == 0 {
					req, _ := localRequests.Requests[buttonEventString]
					req.Button = buttonEvent
					req.TimeAlive = time.Now()
					req.Status = 1
					localRequests.Requests[buttonEventString] = req

					// Test - Send and handle internaly
					reqAssigner <- req
					Tx <- localRequests
				}

			case updatedReq := <-reqUpdate:
				buttonEventString := elevio.BeToString(updatedReq.Button)
				req, _ := localRequests.Requests[buttonEventString]

				// Første if kan fjernes dersom vi kun mottar fullførte og ikke feilet
				if updatedReq.Status == 3 {
					fmt.Println(peers_alive)
					if len(peers_alive) == 1 {
						// Only one alive on the network
						req.Status = 0
					} else {
						req.Status = updatedReq.Status
					}
				}

				localRequests.Requests[buttonEventString] = req
				fmt.Println("Finished request:", buttonEventString)

			case otherRequests := <-Rx:
				// Disregard our own message
				if otherRequests.Id == localRequests.Id {
					break
				}

				for key, value := range localRequests.Requests {
					otherValue := otherRequests.Requests[key]

					if value.Status == 0 && otherValue.Status == 0 {
						continue
					}
					localRequests.Requests[key] = CyclicCounter(id, value, otherValue, peers_alive)
					//fmt.Println(GlobalReq[key])
				}

			case peer_update := <-peerRx:
				peers_alive = peer_update.Peers
				fmt.Println(peers_alive)
		}
	}
}

func CyclicCounter(id string, req Request, otherReq Request, peers_alive []string) Request {
	n := 3

	if req.Status == 0 && otherReq.Status == n {
		return req
	}

	// Discard any number less than ours, except if we are at n and someone else is at 0
	if req.Status < otherReq.Status {
		return otherReq
	} else if req.Status == n && otherReq.Status == 0 {
		return otherReq
	}

	if req.Status == n {
		// Combine the barrier arrays
		req.Barrier = MergeUnique(req.Barrier, otherReq.Barrier)

		if !Contains(req.Barrier, id) {
			req.Barrier = append(req.Barrier, id)
		}

		if len(req.Barrier) >= len(peers_alive) {
			fmt.Println("Able to reset counter for task:", req.Button)
			req.Status = 0
			req.Barrier = make([]string, 0)
		}

		return req
	}

	return req
}

func CostToString(id string, cost int) string {
	return fmt.Sprintf("%s:%d", id, cost)
}

func MergeUnique(arr1 []string, arr2 []string) []string {
	unique := make(map[string]bool)
	merged := make([]string, 0)

	for _, str := range arr1 {
		if !unique[str] {
			unique[str] = true
			merged = append(merged, str)
		}
	}

	for _, str := range arr2 {
		if !unique[str] {
			unique[str] = true
			merged = append(merged, str)
		}
	}

	return merged
}

func Contains(slice []string, str string) bool {
    for _, v := range slice {
        if v == str {
            return true
        }
    }
    return false
}
