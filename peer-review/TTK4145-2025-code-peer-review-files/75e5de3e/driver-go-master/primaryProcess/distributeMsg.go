package primaryprocess

import (
	. "Driver-go/elevator"
	. "Driver-go/utilities"
	"fmt"
	"time"
)

// Parser JSON-stringen til en ElevatorMessage-struktur og håndterer den basert på Tag
func PrimaryProcessDistributeMsg(jsonStr string) {

	message, err := UtilitiesRecieveJsonString(jsonStr)
	if err != nil {
		fmt.Println("Feil ved parsing av melding")
		return
	}

	switch Tag(message.Tag) {

	case Acknowledgement:
		fmt.Println("Mottok Acknowledgement")
		//TODO
		//PrimaryProcessAssignHallCall(dir, floor)
		//Sender kopi av hallcalls til backup

	case ButtonPress:
		//Lagrer hallcall
		PrimaryProcessSaveHallCall(message.Button, message.Floor)
		//PrimaryProcessSendCopyToBackup()

	case HeartbeatSlave:
		fmt.Println("Mottok Heartbeat fra slave")
		//Lagrer verdensbildet
		vb := Worldview{
			HeisID:    message.ElevatorID,
			Timestamp: time.Now(),
			requests:  message.State.Requests,
			e:         message.State,
		}
		PrimaryProcessUpdateAliveNodes(vb)

	case HeartbeatBackup:
		fmt.Println("Mottok Heartbeat fra backup")
		//Lagrer verdensbildet
		vb := Worldview{
			HeisID:    message.ElevatorID,
			Timestamp: time.Now(),
			requests:  message.State.Requests,
			e:         message.State,
		}
		PrimaryProcessUpdateAliveNodes(vb)

	case HeartbeatMaster:
		fmt.Println("Mottok Heartbeat fra master")

	default:
		fmt.Println("Ukjent Tag")
	}
}
