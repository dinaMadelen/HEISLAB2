package elevator_network

import (
	"encoding/json"
	"fmt"
	"net"
	"strconv"
	"strings"

	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/driver-go/elevio"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/elevator"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/network"
	"github.com/anonym/TTK4145-Project/elev_algo_go/elevator"
	"github.com/anonym/TTK4145-Project/elev_algo_go/network"
)

func WorldviewToPacket(worldview elevator.ElevWorldView, SequenceNumber uint32, Timestamp uint64) (network.Packet, error) {
	elevStateJSON, err := json.Marshal(worldview.SenderElevState)
	if err != nil {
		return network.Packet{}, fmt.Errorf("Error marshaling elev state: %v", err)
	}

	hallRequestStrings := make([]string, 8) //This is the tricky part, formatting the requests string correctly
	for i := 0; i < 4; i++ {
		for j := 0; j < 2; j++ {
			hallRequestStrings[i*2+j] = strconv.FormatBool(worldview.HallRequests[i][j])
		}
	}
	hallRequestsStr := strings.Join(hallRequestStrings, ",")

	// Combine all parts into a single string
	dataStr := fmt.Sprintf("%s;%s;%s", string(elevStateJSON), hallRequestsStr, worldview.ID)

	// Create the packet
	packet := network.Packet{
		SequenceNumber: SequenceNumber,
		Timestamp:      Timestamp,
		Data:           []byte(dataStr),
	}

	return packet, nil
}

func SendWorldview(worldviewPacket network.Packet) error {
	serializedWorldview := network.SerializePacket(worldviewPacket)
	worldviewAdrress := net.UDPAddr{IP: net.ParseIP(elevator.BroadcastingIP), Port: elevator.MsgTypeWorldview}

	err := network.SendUDP(&worldviewAdrress, serializedWorldview)
	if err != nil {
		return fmt.Errorf("Sending worldview failed: %v\n", err)
	}
	return err
}

func ButtonpressToPacket(SequenceNumber_ uint32, Timestamp_ uint64, buttonPress elevio.ButtonEvent) (network.Packet, error) {
	floorStr := strconv.Itoa(buttonPress.Floor)
	buttonTypeStr := strconv.Itoa(int(buttonPress.Button))

	dataStr := strings.Join([]string{floorStr, buttonTypeStr}, ";")

	packet := network.Packet{
		SequenceNumber: SequenceNumber_,
		Timestamp:      Timestamp_,
		Data:           []byte(dataStr),
	}

	return packet, nil
}

func SendButtonRequest(SequenceNumber_ uint32, Timestamp_ uint64, buttonRequest elevio.ButtonEvent) error {
	buttonRequestAddress := net.UDPAddr{IP: net.ParseIP(elevator.BroadcastingIP), Port: int(elevator.MsgTypeButtonReq)}
	buttonPacket, err := ButtonpressToPacket(SequenceNumber_, Timestamp_, buttonRequest)
	if err != nil {
		return err
	}
	serializedPacket := network.SerializePacket(buttonPacket)
	err = network.SendUDP(&buttonRequestAddress, serializedPacket)
	return err
}

//TODO: Make elevator listener functionality
