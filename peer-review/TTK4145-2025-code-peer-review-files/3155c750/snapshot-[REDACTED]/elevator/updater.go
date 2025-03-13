package elevator

import(
	"root/assigner"
	"root/SharedData"
	"root/transmitter"
	"fmt"
)

func UpdatesharedHallRequests(update [3]int){
	sharedHallRequests := sharedData.GetsharedHallRequests()
	if update[2] == 1 && update[1] != 2{//igneores updates to cab requests(update[1] != 2)
		sharedHallRequests[update[0]][update[1]] = true
			
		}else if update[1] != 2{
		sharedHallRequests[update[0]][update[1]] = false
		}
	sharedData.ChangeSharedHallRequests(sharedHallRequests)
	ChangeLocalHallRequests()
	}
func Transmitt_update_and_update_localHallRequests(update_val [3]int, elevatorData sharedData.Elevator_data){ //sends the hall requests update to the other elevator and updates the local hall requests
	UpdatesharedHallRequests(update_val)
	transmitter.Send_update(update_val)
	transmitter.Send_Elevator_data(elevatorData)
}

func ChangeLocalHallRequests(){
	fmt.Println(GetElevatordata())
	fmt.Println(sharedData.GetRemoteElevatorData())

	elevator.requests = makeRequests(assigner.Assigner(GetElevatordata(), sharedData.GetRemoteElevatorData(),sharedData.GetsharedHallRequests()),GetCabRequests(elevator.requests))

}

