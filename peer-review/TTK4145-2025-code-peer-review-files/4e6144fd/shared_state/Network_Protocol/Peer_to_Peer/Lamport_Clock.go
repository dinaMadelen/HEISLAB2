package peer_to_peer

import (
	"Constants"
	"fmt"
	"strconv"
)

type Lamport_Clock struct {
	time int
}

func New_Lamport_Clock() Lamport_Clock {
	return Lamport_Clock{
		time: 0,
	}
}

func New_Lamport_Clock_From_String(str string) Lamport_Clock {
	time, err := strconv.Atoi(str)

	if err != nil {
		fmt.Printf("Error in P2P Lamport Clock creation: %s\n", err)
		return New_Lamport_Clock()
	}

	return Lamport_Clock{
		time: time,
	}
}

func (clock Lamport_Clock) String() string {
	return strconv.Itoa(clock.time)
}

func (clock Lamport_Clock) Get_Time() int {
	return clock.time
}

// Advances the clock after an event.
func (clock *Lamport_Clock) Event() {
	clock.time = clock.time + 1
}

func (receiver_clock *Lamport_Clock) Update(sender_clock Lamport_Clock) {
	if receiver_clock.Is_Less_Than(sender_clock) {
		receiver_clock.time = sender_clock.time
	}

	receiver_clock.Event()
}

// If a -> b, then b.time < a.time
func (lesser Lamport_Clock) Is_Less_Than(greater Lamport_Clock) bool {
	// Check if the clocks have wrapped around
	const lower_edge int = Constants.LAMPORT_CLOCK_WRAPAROUND_LOWER_EDGE
	const upper_edge int = Constants.LAMPORT_CLOCK_WRAPAROUND_UPPER_EDGE

	if greater.time < lower_edge && lesser.time > upper_edge {
		return true
	}
	if lesser.time < lower_edge && greater.time > upper_edge {
		return false
	}

	// Otherwise just check the timers directly.
	return lesser.time < greater.time
}
