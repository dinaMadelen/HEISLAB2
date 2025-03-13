package elevator

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
)

// Global instance of SharedState
var GlobalState *SharedState

// Lokal tilstand for denne heisen
var localElevator Elevator

// Kanaler for å sende og motta oppdrag
var AssignRequestChannel = make(chan struct{})            // Signaler for ny oppdragsfordeling
var AssignResultChannel = make(chan map[string][][2]bool) // Resultat fra assigner

// ===================== SHARED STATE HELPERS ===================== //
// RequestAssigner listens for new requests and assigns them to elevators
// The calls to the AssignRequestChannel will come from FSM whenever a new request appears or an elevator changes state.K

// TODO: RequestAssigner må kjøre i en egen goroutine
func RequestAssigner() {
	for {
		<-AssignRequestChannel
		assignments := getHallRequestAssignments()
		AssignResultChannel <- assignments
	}
}

// Initialize shared state with default values
func InitSharedState() {
	GlobalState = &SharedState{
		HRA: HRAInput{
			HallRequests: make([][2]bool, N_FLOORS),
			States:       make(map[string]Elevator),
		},
	}
	// Sett opp lokal heistilstand
	localElevator = ElevatorUninitialized()
}

// Update the state of a single elevator
func UpdateLocalElevator(e Elevator) {
	localElevator = e
}

// Oppdaterer SharedState med denne heisens lokale tilstand
func UpdateSharedState() {
	GlobalState.mu.Lock()
	defer GlobalState.mu.Unlock()
	GlobalState.HRA.States[getElevatorID()] = localElevator
}

// Retrieve a copy of all elevator states
func GetLocalElevator() Elevator {
	// Function can become useful when/if local elevator is replaced
	return localElevator
}

// Henter en kopi av hele SharedState
func GetSharedState() HRAInput {
	GlobalState.mu.Lock()
	defer GlobalState.mu.Unlock()
	return GlobalState.HRA
}

// getElevatorID returns a unique identifier for this elevator instance.
func getElevatorID() string {
	hostname, err := os.Hostname()
	if err != nil {
		fmt.Println("Error getting hostname:", err)
		return "unknown_elevator"
	}
	return hostname
}

// ===================== ELEVATOR HELPERS ===================== //
// Initialize a new elevator with default values
func ElevatorUninitialized() Elevator {
	return Elevator{
		Floor:       -1,
		Dirn:        D_Stop,
		Behaviour:   EB_Idle,
		CabRequests: make([]bool, N_FLOORS),
	}
}

// Print elevator state to the console
func ElevatorPrint(es Elevator) {
	fmt.Println("  +--------------------+")
	fmt.Printf("  | Floor  = %-2d        |\n", es.Floor)
	fmt.Printf("  | Dirn   = %-12.12s |\n", es.Dirn)
	fmt.Printf("  | Behav  = %-12.12s |\n", es.Behaviour)
	fmt.Println("  +--------------------+")
	fmt.Println("  |  | Up  | Down | Cab |")

	for f := N_FLOORS - 1; f >= 0; f-- {
		fmt.Printf("  | %d", f)
		for btn := 0; btn < N_BUTTONS; btn++ {
			if (f == N_FLOORS-1 && btn == B_HallUp) || (f == 0 && btn == B_HallDown) {
				fmt.Print("|     ")
			} else {
				if es.CabRequests[f] {
					fmt.Print("|  #  ")
				} else {
					fmt.Print("|  -  ")
				}
			}
		}
		fmt.Println("|")
	}
	fmt.Println("  +--------------------+")
}

// ===================== REQUEST ASSIGNER ===================== //
// Calls `hall_request_assigner` to get updated assignments for all elevators
func getHallRequestAssignments() map[string][][2]bool {
	// Fetch latest state
	sharedState := GetSharedState()

	// Convert to JSON
	jsonBytes, err := json.Marshal(sharedState)
	if err != nil {
		fmt.Println("json.Marshal error:", err)
		return nil
	}

	// Call `hall_request_assigner`
	ret, err := exec.Command("../hall_request_assigner/hall_request_assigner", "-i", string(jsonBytes)).CombinedOutput()
	if err != nil {
		fmt.Println("exec.Command error:", err)
		fmt.Println(string(ret))
		return nil
	}

	// Debugging: Print raw response
	fmt.Println("Raw response from hall_request_assigner:", string(ret))

	// Parse JSON response
	output := make(map[string][][2]bool)
	err = json.Unmarshal(ret, &output)
	if err != nil {
		fmt.Println("json.Unmarshal error:", err)
		return nil
	}

	return output
}
