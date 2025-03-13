package hub_fsm

import (
	"fmt"
	"time"
	"sort"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hra"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hub"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hub_network"
)

var (
	hubElement      hub.Hub
	backupWorldView hub.WorldView
	localIP		    string
	activeIP		string
	backupIP		string
	idleIP          string
	lastSeen		map[string]time.Time
)

const (
	TIMEOUT = 2 * time.Second
)

func Init(IP string) {	
	hub.Initialize()
	localIP = IP
	ElectActive()
	fmt.Println("Hub FSM initialized!")
}

func OnReceivedBtnPress(BtnPress hub.ButtonEvent) {
	fmt.Println("Hub FSM: On order received")
	var chosenelev string
	var event hub.ButtonEvent
	switch hubElement.State {
	case hub.Backup:
		// Update world view or do nothing
		// UpdateWorldView(BtnPress)
	case hub.Active:
		chosenelev, event = DistributeOrder(BtnPress)
		broadcastOrder(chosenelev, event)
	}
}

func OnUpdateWorldView(worldview hub.WorldView) {
	// Find the correct WorldView to update based on ID
	var targetWV *hub.WorldView
	for i := range hubElement.WVs {
		if hubElement.WVs[i].IP == worldview.IP {
			targetWV = &hubElement.WVs[i]
			break
		}
	}

	if targetWV == nil {
		fmt.Println("WorldView with ID", worldview.IP, "not found")
		return
	}

	// Update hall requests
	for i := 0; i < len(worldview.HallRequests); i++ {
		for j := 0; j < len(worldview.HallRequests[i]); j++ {
			if worldview.HallRequests[i][j] != targetWV.HallRequests[i][j] {
				targetWV.HallRequests[i][j] = worldview.HallRequests[i][j]
			}
		}
	}

	// Update sender's elevator state
	senderState := worldview.SenderElevState
	worldViewState := targetWV.SenderElevState

	if senderState.Behavior != worldViewState.Behavior {
		worldViewState.Behavior = senderState.Behavior
	}

	if senderState.Floor != worldViewState.Floor {
		worldViewState.Floor = senderState.Floor
	}
	if senderState.Direction != worldViewState.Direction {
		worldViewState.Direction = senderState.Direction
	}
	for i := 0; i < len(senderState.CabRequests); i++ {
		if senderState.CabRequests[i] != worldViewState.CabRequests[i] {
			worldViewState.CabRequests[i] = senderState.CabRequests[i]
		}
	}
}


// DistributeSingleOrder handles exactly one ButtonEvent (hall request),
func DistributeOrder(ev hub.ButtonEvent) (string, hub.ButtonEvent) {
    fmt.Println("Hub FSM: Distributing single order")

    hallMatrix := SingleButtonEventToMatrix(ev, 4) 


    input := hra.HRAInput{
        HallRequests: hallMatrix,                 
        States:       CollectHRAElevStates(hubElement.WVs),
    }

    assignment := hra.Hra(input)

    chosenElev := findAssignedElevator(ev, assignment)
    if chosenElev == "" {
        fmt.Println("Warning: No elevator was assigned by HRA. Maybe no active elevators?")
    } else {
        fmt.Printf("Chosen Elevator: %s for (floor=%d, button=%d)\n", chosenElev, ev.Floor, ev.Button)
    }

    return chosenElev, ev
}

func findAssignedElevator(ev hub.ButtonEvent, assignment map[string][][2]bool) string {
    floor := ev.Floor
    dirIndex := 0
    if ev.Button == hub.BT_HallDown {
        dirIndex = 1
    }

    for elevatorID, out := range assignment {
        if floor >= 0 && floor < len(out) {
            row := out[floor]
            if len(row) >= 2 && row[dirIndex] {
                // This elevator was assigned
				for _, e := range hubElement.WVs {
					if e.IP == elevatorID {
						return e.IP
					}
				}
            }
        }
    }
    return ""
}

func SingleButtonEventToMatrix(ev hub.ButtonEvent, numFloors int) [][2]bool {
    matrix := make([][2]bool, numFloors)

    if ev.Floor < 0 || ev.Floor >= numFloors {
        return matrix
    }

    switch ev.Button {
    case hub.BT_HallUp:
        matrix[ev.Floor][0] = true
    case hub.BT_HallDown:
        matrix[ev.Floor][1] = true
    // ignore BT_Cab
    }

    return matrix
}

// CollectHRAElevStates builds the map of elevator index -> HRAElevState from hubElement's world views.
func CollectHRAElevStates(wvs [3]hub.WorldView) map[string]hra.HRAElevState {
	states := make(map[string]hra.HRAElevState)
    for _, wv := range wvs {
        states[wv.IP] = hra.HRAElevState{
            Floor:      wv.SenderElevState.Floor,
            Direction:  wv.SenderElevState.Direction,
            Behavior:   wv.SenderElevState.Behavior,
            CabRequests: wv.SenderElevState.CabRequests,
        }
    }
    return states
}

// Function which sends the order until the order appears in the worldview of selected elevator
func broadcastOrder(elevatorIP string, event hub.ButtonEvent) {
	fmt.Println("Hub FSM: Sending order to elevator", elevatorIP)
	for{
		for _, e := range hubElement.WVs {
			if e.IP == elevatorIP {
				if e.HallRequests[event.Floor][int(event.Button)] {
					fmt.Println("Order was successfully distributed to elevator", elevatorIP)
					return
				}
			}
		}
		hub_network.SendOrder(elevatorIP, event)
	}
}



func OnHeartbeat(hb hub.Heartbeat) {
    now := time.Now()

    lastSeen[hb.IP] = now

    switch hubElement.State {
    case hub.Active:
        // If we have a designated backup, check if they're alive
        if backupIP != "" {
            last, known := lastSeen[backupIP]
            if known && now.Sub(last) > TIMEOUT {
                fmt.Printf("Active lost contact with backup %s; picking new backup.\n", backupIP)
                pickNewBackup()
            }
        } else {
            // If we have no backup, we should set up a new one
            pickNewBackup()
        }

    case hub.Backup:
        // If we have a known active, but haven't seen them in 2s => promote ourselves
        if activeIP != "" {
            last, known := lastSeen[activeIP]
            if known && now.Sub(last) > TIMEOUT {
                fmt.Printf("Backup sees active %s is dead; promoting self.\n", activeIP)
                becomeActive()
            }
        } else {
            // If active is not defined, go down to idle
        }

    case hub.Idle:
        // Do nothing
    }

    // Order handling
    switch hubElement.State {
    case hub.Active:
        // Do nothing. Active does not receive orders
    case hub.Backup:
        // Do nothing. Backup does not receive orders
        activeIP = hb.IP // Assume the sender is the active
    case hub.Idle:
        // Assign instruction (state) to idle if not nil
        if hb.Instruction != -1 {
            hubElement.State = hb.Instruction
            activeIP = hb.IP // Assume the sender is the active
        }
    }
}

//------------------------------------
// Helper functions for picking backup, becoming active, IP compare
//------------------------------------

func pickNewBackup() {
    candidateIP := idleIP
    if candidateIP == "" {
        fmt.Println("No idle node found to act as backup.")
        backupIP = ""
        return
    }
    backupIP = candidateIP
    hb := hub.Heartbeat{
        IP:         localIP,
        State:      hub.Active,
        Instruction: hub.Backup,
    }

    hub_network.SendHeartbeat(time.Now().Unix(), hb)
    fmt.Printf("New backup chosen: %s\n", backupIP)
}

func becomeActive() {
    hubElement.State = hub.Active
    activeIP = localIP
    backupIP = ""
    fmt.Printf("I am now Active. IP=%s\n", localIP)
    pickNewBackup()
}




func ElectActive() {
    ipt, err := hub_network.GetLocalIP()
    ip := ipt.String()
    if err != nil {
        fmt.Println("Failed to get local IP:", err)
        return
    }
    localIP = ip
    hubElement.State = hub.Idle
	discoveredIps := make(map[string]int)
	discoveredActive := false


    fmt.Printf("INIT: Starting as Idle at IP %s\n", localIP)

    heatbeatChan := make(chan hub.Heartbeat)
    hub_network.InitByHeartbeats(heatbeatChan, hub.Heartbeat{IP: localIP, State: hub.Idle})

    listenDuration := 2 * time.Second
    deadline := time.Now().Add(listenDuration)
    for time.Now().Before(deadline) {
        select {
        case hb := <-heatbeatChan: 
            handleInitHeartbeat(hb, discoveredIps, discoveredActive)
        case <-time.After(100 * time.Millisecond):
            // Tick, keep looping
        }
    }

    if discoveredActive {
        fmt.Println("INIT: Found an existing Active hub; remain Idle.")
        hubElement.State = hub.Idle
    } else {
        ips := []string{}
        for ipStr, role := range discoveredIps {
            if role == hub.Idle {
                ips = append(ips, ipStr)
            }
        }
        
        ips = append(ips, localIP)

        sort.Strings(ips) 
        if len(ips) > 0 && ips[0] == localIP {
            fmt.Println("INIT: No Active found, I'm the lowest IP among Idle => becoming Active.")
            hubElement.State = hub.Active
        } else {
            fmt.Println("INIT: Found no Active, but I'm not the lowest IP => remain Idle.")
            hubElement.State = hub.Idle
        }
    }

    if hubElement.State == hub.Active {
        fmt.Println("INIT: I am now Active.")
        pickNewBackup()
    } else {
        fmt.Println("INIT: I am Idle after init.")
    }
}

func handleInitHeartbeat(hb hub.Heartbeat, discoveredIps map[string]int, discoveredActive bool) (map[string]int, bool) {
    fmt.Printf("INIT: Received heartbeat from IP=%s, role=%d\n", hb.IP, hb.State)

    if hb.State == hub.Active {
        discoveredActive = true
    }
    discoveredIps[hb.IP] = hb.State
	return discoveredIps, discoveredActive
}

func GetLocalIPs() []string {
	stringSlice := []string{"r3r2r3, r3r2r3, r3r2r3"} // Placeholder, TODO: Fill in with actual IPs
	return stringSlice
}

func GetState() int {
    return hubElement.State
}