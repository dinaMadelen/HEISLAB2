package worldview

import (
	"TTK4145-Heislab/configuration"
	"TTK4145-Heislab/driver-go/elevio"
	"TTK4145-Heislab/single_elevator"
	"time"
)

type ElevStateMsg struct {
	Elev single_elevator.State
	Cab  []configuration.OrderMsg
}

type WorldView struct {
	ID string
	// Acklist []string - har droppet en overall acklist fordi vi heller vil ha en egen acklist per order som heisene legger seg til på
	ElevatorStatusList map[string]ElevStateMsg
	HallOrderStatus    [][configuration.NumButtons - 1]configuration.OrderMsg
}

func WorldViewManager(
	elevatorID string,
	WorldViewTXChannel chan<- WorldView,
	WorldViewRXChannel <-chan WorldView,
	buttonPressedChannel <-chan elevio.ButtonEvent,
	mergeChannel chan<- elevio.ButtonEvent,
	newOrderChannel chan<- single_elevator.Orders,
	completedOrderChannel <-chan elevio.ButtonEvent,
	numPeersChannel <-chan int,
	IDPeersChannel <-chan []string,
) {

	initLocalWorldView := InitializeWorldView(elevatorID)
	localWorldView := &initLocalWorldView

	SendLocalWorldViewTimer := time.NewTimer(time.Duration(configuration.SendWVTimer) * time.Millisecond)
	numPeers := 0

	orderDistributed := make([][configuration.NumButtons - 1]bool, configuration.NumFloors)

	IDsAliveElevators := []string{}

	for {
	OuterLoop:
		select {
		case IDList := <-IDPeersChannel:
			numPeers = len(IDList)
			IDsAliveElevators = IDList

		case <-SendLocalWorldViewTimer.C: //local world view updates
			localWorldView.ID = elevatorID
			WorldViewTXChannel <- *localWorldView
			SendLocalWorldViewTimer.Reset(time.Duration(configuration.SendWVTimer) * time.Millisecond)

		case buttonPressed := <-buttonPressedChannel:
			newLocalWorldView := updateWorldViewWithButton(localWorldView, buttonPressed, true)
			//feilhåndtering
			if !validWorldView(newLocalWorldView) { //ikke laget validWorldView enda
				continue
			}
			localWorldView = &newLocalWorldView
			WorldViewTXChannel <- *localWorldView
			ResetAckList(localWorldView) //tømmer ackliste og legger til egen ID - dette er ikke riktig lenger da vi nå skal implementere egen acklist per order

		case complete := <-completedOrderChannel:
			newLocalWorldView := updateWorldViewWithButton(localWorldView, complete, false)
			//feilhåndtering
			if !validWorldView(newLocalWorldView) { //ikke laget validWorldView enda
				continue
			}
			localWorldView = &newLocalWorldView
			WorldViewTXChannel <- *localWorldView
			ResetAckList(localWorldView) //tømmer ackliste og legger til egen ID - dette er ikke riktig lenger da vi nå skal implementere egen acklist per order

		//MESSAGE SYSTEM - connection with network
		case updatedWorldView := <-WorldViewRXChannel: //mottar en melding fra en annen heis
			//dette er ikke ferdig i det hele tatt.

			newLocalWorldView = MergeWorldViews(localWorldView, updatedWorldView, IDsAliveElevators) //mergeworldview er ikke laget
			if !validWorldView(newLocalWorldView) {
				continue
			}
			// send new worldview on network

			AssignHallOrders := AssignOrder(*localWorldView)
			OurHall := AssignHallOrders[localWorldView.ID] //value ut av map
			OurCab := GetOurCAB(*localWorldView, localWorldView.ID)
			OrderMatrix := MergeCABandHRAout(OurHall, OurCab)
			newOrderChannel <- OrderMatrix

		}
	}
}

//lys er kun satt for single elevator, og skal settes for hele heisen når hele order-systemet er implementert
