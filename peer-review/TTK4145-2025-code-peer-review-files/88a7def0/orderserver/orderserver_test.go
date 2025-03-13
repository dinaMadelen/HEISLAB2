package orderserver

import (
	"fmt"
	"testing"

	"group48.ttk4145.ntnu/elevators/models"
)

// TestCalculateOrders tests the CalculateOrders function
func TestCalculateOrders(t *testing.T) {
	// Create a channel for the global state
	fmt.Println("TestCalculateOrders started")
	validatedRequests := make(chan models.Request, 1)
	alive := make(chan []models.Id, 1)
	orders := make(chan models.Orders, 1)
	state := make(chan models.ElevatorState, 1)

	go RunOrderServer(validatedRequests, state, alive, orders, 1)

	// Send test data to the channels
	alive <- []models.Id{1, 2}
	state <- models.ElevatorState{
		Id:        1,
		Floor:     0,
		Direction: models.Stop,
		Behavior:  models.Idle,
	}
	state <- models.ElevatorState{
		Id:        2,
		Floor:     0,
		Direction: models.Stop,
		Behavior:  models.Idle,
	}

	validatedRequests <- models.Request{
		Origin: models.Origin{
			Source:     models.Hall{},
			Floor:      1,
			ButtonType: models.HallUp,
		},
		Status: models.Confirmed,
	}

	// Check the output from the orders channel
	o := <-orders
	fmt.Println("Orders received:", o)

	fmt.Println("TestCalculateOrders done")
}

func TestOptimalHallRequests(t *testing.T) {
	numFloors := 4

	tests := []struct {
		name     string
		states   []elevatorstate
		hallreqs [][2]bool
		expected map[models.Id]models.Orders
	}{
		{
			name: "Elevator 1 is idle one floor away, other elevators have several cab orders",
			states: []elevatorstate{
				{models.ElevatorState{Id: 1, Floor: 0, Direction: models.Stop, Behavior: models.Idle}, make([]bool, numFloors)},
				{models.ElevatorState{Id: 2, Floor: 3, Direction: models.Down, Behavior: models.DoorOpen}, []bool{true, false, false, false}},
				{models.ElevatorState{Id: 3, Floor: 2, Direction: models.Up, Behavior: models.Moving}, []bool{true, false, false, true}},
			},
			hallreqs: [][2]bool{
				{false, false},
				{true, false},
				{false, false},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				1: {{false, false}, {true, false}, {false, false}, {false, false}},
				2: {{false, false}, {false, false}, {false, false}, {false, false}},
				3: {{false, false}, {false, false}, {false, false}, {false, false}},
			},
		},
		{
			name: "Two elevators moving from each end toward the middle floors",
			states: []elevatorstate{
				{models.ElevatorState{Id: 1, Floor: 0, Direction: models.Stop, Behavior: models.Idle}, make([]bool, numFloors)},
				{models.ElevatorState{Id: 2, Floor: 3, Direction: models.Stop, Behavior: models.Idle}, make([]bool, numFloors)},
			},
			hallreqs: [][2]bool{
				{false, false},
				{false, true},
				{true, false},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				1: {{false, false}, {false, true}, {false, false}, {false, false}},
				2: {{false, false}, {false, false}, {true, false}, {false, false}},
			},
		},
		{
			name: "Change E1 idle->moving, stop->up. E1 is closer, but otherwise same scenario",
			states: []elevatorstate{
				{models.ElevatorState{Id: 1, Floor: 0, Direction: models.Up, Behavior: models.Moving}, make([]bool, numFloors)},
				{models.ElevatorState{Id: 2, Floor: 3, Direction: models.Stop, Behavior: models.Idle}, make([]bool, numFloors)},
			},
			hallreqs: [][2]bool{
				{false, false},
				{false, true},
				{true, false},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				1: {{false, false}, {false, true}, {false, false}, {false, false}},
				2: {{false, false}, {false, false}, {true, false}, {false, false}},
			},
		},
		{
			name: "Add cab order to E1, so that it has to continue upward anyway",
			states: []elevatorstate{
				{models.ElevatorState{Id: 1, Floor: 0, Direction: models.Stop, Behavior: models.Idle}, []bool{false, false, true, false}},
				{models.ElevatorState{Id: 2, Floor: 3, Direction: models.Stop, Behavior: models.Idle}, make([]bool, numFloors)},
			},
			hallreqs: [][2]bool{
				{false, false},
				{false, true},
				{true, false},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				1: {{false, false}, {false, false}, {true, false}, {false, false}},
				2: {{false, false}, {false, true}, {false, false}, {false, false}},
			},
		},
		{
			name: "Two elevators are the same number of floors away from an order, but one is moving toward it",
			states: []elevatorstate{
				{models.ElevatorState{Id: 27, Floor: 1, Direction: models.Down, Behavior: models.Moving}, make([]bool, numFloors)},
				{models.ElevatorState{Id: 20, Floor: 1, Direction: models.Down, Behavior: models.DoorOpen}, make([]bool, numFloors)},
			},
			hallreqs: [][2]bool{
				{true, false},
				{false, false},
				{false, false},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				27: {{true, false}, {false, false}, {false, false}, {false, false}},
				20: {{false, false}, {false, false}, {false, false}, {false, false}},
			},
		},
		{
			name: "Two hall requests at the same floor, but the closest elevator also has a cab call further in the same direction",
			states: []elevatorstate{
				{models.ElevatorState{Id: 1, Floor: 3, Direction: models.Down, Behavior: models.Moving}, []bool{true, false, false, false}},
				{models.ElevatorState{Id: 2, Floor: 3, Direction: models.Down, Behavior: models.Idle}, make([]bool, numFloors)},
			},
			hallreqs: [][2]bool{
				{false, false},
				{true, true},
				{false, false},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				1: {{false, false}, {false, true}, {false, false}, {false, false}},
				2: {{false, false}, {true, false}, {false, false}, {false, false}},
			},
		},
		{
			name: "Single elevator starting at 0, up + down orders at both floor 1 and 2, for inDirn mode specifically",
			states: []elevatorstate{
				{models.ElevatorState{Id: 1, Floor: 0, Direction: models.Stop, Behavior: models.Idle}, make([]bool, numFloors)},
			},
			hallreqs: [][2]bool{
				{false, false},
				{true, true},
				{true, true},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				1: {{false, false}, {true, true}, {true, true}, {false, false}},
			},
		},
		{
			name: "Single elevator starting at 1, idle with up orders at 1 and 2",
			states: []elevatorstate{
				{models.ElevatorState{Id: 1, Floor: 1, Direction: models.Stop, Behavior: models.Idle}, make([]bool, numFloors)},
			},
			hallreqs: [][2]bool{
				{false, false},
				{true, false},
				{true, false},
				{false, false},
			},
			expected: map[models.Id]models.Orders{
				1: {{false, false}, {true, false}, {true, false}, {false, false}},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := make(map[models.Id]elevatorstate)
			for _, state := range tt.states {
				m[state.Id] = state
			}

			elevators := elevators{
				states:       m,
				hallRequests: tt.hallreqs,
			}
			actual := optimalHallRequests(elevators)
			for id, orders := range tt.expected {
				if !equalOrders(actual[id], orders) {
					t.Errorf("expected %v, got %v", orders, actual[id])
				}
			}
		})
	}
}

func equalOrders(a, b models.Orders) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		for j := range a[i] {
			if a[i][j] != b[i][j] {
				return false
			}
		}
	}
	return true
}
