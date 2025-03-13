package hub_network

import (
	"encoding/json"
	"fmt"
	"net"
	"strconv"
	"strings"
	"time"

	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/driver-go/elevio"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hub"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/network"
)

func SendHeartbeat(Timestamp_ int64, heartbeat_ hub.Heartbeat) error {
	address := net.UDPAddr{IP: net.ParseIP(hub.BroadcastingIP), Port: int(hub.MsgTypeHeartbeat)}

	stringIP := heartbeat_.IP
	HubSequenceNumber := "0" // Placeholder value since SequenceNumber is not defined
	stringState := string(heartbeat_.State)
	stringInstruction := string(heartbeat_.Instruction)
	dataString := stringIP + ";" + stringState + ";" + stringInstruction
	Data_ := []byte(dataString)

	heartbeatPacket := network.Packet{SequenceNumber: HubSequenceNumber, Timestamp: Timestamp_, Data: Data_}
	//A heartbeat is defined as a message being "hubState;order", where order is either an order or "-1"
	messageToSend := network.SerializePacket(heartbeatPacket)
	err := network.SendUDP(address, messageToSend)
	if err != nil {
		return err
	}

	return nil
}

func ReceivePackets(WorldviewChannel chan network.Packet, ButtonChannel chan network.Packet, HeartbeatChannel chan network.Packet) error {
	worldviewAddress := net.UDPAddr{IP: net.ParseIP(hub.BroadcastingIP), Port: int(hub.MsgTypeWorldview)}
	buttonReqAddress := net.UDPAddr{IP: net.ParseIP(hub.BroadcastingIP), Port: int(hub.MsgTypeButtonReq)}
	heartbeatAddress := net.UDPAddr{IP: net.ParseIP(hub.BroadcastingIP), Port: int(hub.MsgTypeHeartbeat)}

	go network.ListenUDP(&worldviewAddress, WorldviewChannel)
	go network.ListenUDP(&buttonReqAddress, ButtonChannel)
	go network.ListenUDP(&heartbeatAddress, HeartbeatChannel)

	return nil
}

func PacketToWorldview(worldviewPacket network.Packet) (uint32, uint64, hub.WorldView, error) {
	SequenceNumber_ := worldviewPacket.SequenceNumber
	Timestamp_ := worldviewPacket.Timestamp
	dataStr := string(worldviewPacket.Data) //Retrieves the string from the packet
	parts := strings.Split(dataStr, ";")    //Separates string into parts

	if len(parts) < 4 {
		return 0, 0, hub.WorldView{}, fmt.Errorf("Invalid packet data format\n")
	}

	var elevState hra.HRAElevState
	err := json.Unmarshal([]byte(parts[0]), &elevState) //Translates a []byta back into usable JSON format
	if err != nil {
		return 0, 0, hub.WorldView{}, fmt.Errorf("Error unmarshaling elev state: %v\n", err)
	}

	hallRequests := [4][2]bool{}
	hallRequestParts := strings.Split(parts[1], ",")
	if len(hallRequestParts) != 8 {
		return 0, 0, hub.WorldView{}, fmt.Errorf("Invalid hall requests format\n")
	}

	for i := 0; i < 4; i++ {
		for j := 0; j < 2; j++ {
			val, err := strconv.ParseBool(hallRequestParts[i*2+j])
			if err != nil {
				return 0, 0, hub.WorldView{}, fmt.Errorf("Error parsing hall request: %v\n", err)
			}
			hallRequests[i][j] = val //Translates string of bools into the [4][2]bool type
		}
	}

	// Parse ID
	id := parts[2]

	receivedWorldview := hub.WorldView{ //Reconstructs the received worldview
		SenderElevState: elevState,
		HallRequests:    hallRequests,
		ID:              id,
	}

	return SequenceNumber_, Timestamp_, receivedWorldview, nil
}

func PacketToButtonpress(buttonPacket network.Packet) (uint32, uint64, elevio.ButtonType, error) {
	SequenceNumber_ := buttonPacket.SequenceNumber
	Timestamp_ := buttonPacket.Timestamp
	dataStr := string(buttonPacket.Data)
	parts := strings.Split(dataStr, ";")

	if len(parts) != 2 {
		return 0, 0, elevio.ButtonEvent{}, fmt.Errorf("Invalid packet(ButtonEvent) data format\n")
	}

	Floor_, err := strconv.Atoi(parts[0])
	if err != nil {
		return 0, 0, elevio.ButtonEvent{}, fmt.Errorf("Could not convert string(Floor) to int: %v\n", err)
	}

	ButtonType_, err := strconv.Atoi(parts[1])
	if err != nil {
		return 0, 0, elevio.ButtonEvent{}, fmt.Errorf("Could not convert string(ButtonType) to int: %v\n", err)
	}

	switch elevio.ButtonType(ButtonType_) {
	case elevio.BT_HallDown, elevio.BT_HallDown, elevio.BT_Cab:
		ButtonType_ = elevio.ButtonType(ButtonType_)
	default:
		return 0, 0, elevio.ButtonEvent{}, fmt.Errorf("Could not convert Buttontype int to type ButtonType\n")
	}

	receivedButtonPress := elevio.ButtonEvent{Floor: Floor_, Button: ButtonType_}

	return SequenceNumber_, Timestamp_, receivedButtonPress, nil
}

func PacketToHeartbeat(heartbeatPacket network.Packet) (uint32, uint64, hub.Heartbeat, error) {
	SequenceNumber_ := heartbeatPacket.SequenceNumber
	Timestamp_ := heartbeatPacket.Timestamp
	dataStr := string(heartbeatPacket.Data)
	parts := strings.Split(dataStr, ";")

	if len(parts) != 3 {
		return 0, 0, hub.Heartbeat{}, fmt.Errorf("Invalid packet(Heartbeat) data format\n")
	}

	IP_ := string(parts[0])

	State_, err := strconv.Atoi(parts[1])
	if err != nil {
		return 0, 0, hub.Heartbeat{}, fmt.Errorf("Could not convert Heartbeat state to int: %v\n", err)
	}

	Instruction_, err := strconv.Atoi(parts[2])
	if err != nil {
		return 0, 0, hub.Heartbeat{}, fmt.Errorf("Could not convert Heartbeat instruction to int: %v\n", err)
	}

	newHeartbeat := hub.Heartbeat{IP: IP_, State: State_, Instruction: Instruction_}

	return SequenceNumber_, Timestamp_, newHeartbeat, nil
}

func ButtonOrderToPacket(SequenceNumber_ uint32, Timestamp_ uint64, buttonPress elevio.ButtonEvent) (network.Packet, error) {
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

func SendOrder(elevatorIP string, order elevio.ButtonEvent) error {
	orderAddress := net.UDPAddr{IP: net.ParseIP(hub.BroadcastingIP), Port: int(hub.MsgTypeButtonOrder)}
	elevatorIPUint, err := strconv.ParseUint(elevatorIP, 10, 32)
	if err != nil {
		return err
	}
	orderPacket, err := ButtonOrderToPacket(uint32(elevatorIPUint), 0, order) //TODO
	if err != nil {
		return err
	}
	serializedOrder := network.SerializePacket(orderPacket)

	err = network.SendUDP(&orderAddress, serializedOrder)
	return err

func InitByHeartbeat(HeartbeatChannel chan network.Packet, HubSequenceNumber_ uint32, Timestamp_ int64, heartbeatToSend hub.Heartbeat) error {
	address := net.UDPAddr{IP: net.ParseIP(hub.BroadcastingIP), Port: int(hub.MsgTypeHeartbeat)}
	go func() {
		time.Sleep(time.Millisecond * 200)
		SendHeartbeat(HubSequenceNumber_, Timestamp_, heartbeatToSend) //TODO: This will not send up to date timestamps
	}()

	network.ListenUDP(&address, HeartbeatChannel) //TDOD: Figure out a way to stop the listener after set duration
	return nil
}

func GetLocalIP() (net.IP, error) {
    conn, err := net.Dial("udp", "8.8.8.8:80")
    if err != nil {
        log.Fatal(err)
    }
    defer conn.Close()

    localAddr := conn.LocalAddr().(*net.UDPAddr)

    return localAddr.IP, err
}
