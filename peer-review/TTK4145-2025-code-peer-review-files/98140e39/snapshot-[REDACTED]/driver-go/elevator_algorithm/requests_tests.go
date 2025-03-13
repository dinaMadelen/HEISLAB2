package elevatoralgorithm

import "log"

func testChooseDirectionUp() {
	var elevator Elevator
	elevator.behaviour = moving
	elevator.direction = up

	elevator.requests = [NumFloors][NumButtons]bool{{false, false, false}, {false, false, false}, {false, false, false}, {false, false, false}}

	var myBehaviourPair behaviourPair = behaviourPair{stop, idle}

	if myBehaviourPair != elevator.chooseDirection() {
		log.Fatal("testChooseDirection failed!")
	}

}

func AllRequestsTests() {
	testChooseDirectionUp()
}
