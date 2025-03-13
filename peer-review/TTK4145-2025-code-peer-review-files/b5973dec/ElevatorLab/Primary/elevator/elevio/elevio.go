package elevio

import (
	"fmt"
	"net"
	"sync"
	"time"
)

// --------------- HARDWARE INTERACTION FUNCTIONS --------------- //

// ---LOCAL FUNCTIONS --- //

func (e *Ele) read(in [4]byte) [4]byte {
	e.Mtx.Lock()
	defer e.Mtx.Unlock()

	_, err := e.Conn.Write(in[:])
	if err != nil {
		panic("Lost connection to Elevator Server")
	}

	var out [4]byte
	_, err = e.Conn.Read(out[:])
	if err != nil {
		panic("Lost connection to Elevator Server")
	}

	return out
}

func (e *Ele) write(in [4]byte) {
	e.Mtx.Lock()
	defer e.Mtx.Unlock()

	_, err := e.Conn.Write(in[:])
	if err != nil {
		panic("Lost connection to Elevator Server")
	}
}

func toByte(a bool) byte {
	var b byte = 0
	if a {
		b = 1
	}
	return b
}

func toBool(a byte) bool {
	var b bool = false
	if a != 0 {
		b = true
	}
	return b
}

// --- GLOBAL FUNCTIONS --- //

func NewEle(id int, addr string, numFloors int) (*Ele, error) {
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to elevator server: %w", err)
	}

	return &Ele{
		ID:        id,
		Conn:      conn,
		NumFloors: numFloors,
		Mtx:       sync.Mutex{},
	}, nil
}

func (e *Ele) SetMotorDirection(dir MotorDirection) {
	e.write([4]byte{1, byte(dir), 0, 0})
}

func (e *Ele) SetButtonLamp(button ButtonType, floor int, value bool) {
	e.write([4]byte{2, byte(button), byte(floor), toByte(value)})
}

func (e *Ele) SetFloorIndicator(floor int) {
	e.write([4]byte{3, byte(floor), 0, 0})
}

func (e *Ele) SetDoorOpenLamp(value bool) {
	e.write([4]byte{4, toByte(value), 0, 0})
}

func (e *Ele) SetStopLamp(value bool) {
	e.write([4]byte{5, toByte(value), 0, 0})
}

func (e *Ele) PollButtons(receiver chan<- ButtonEvent) {
	prev := make([][3]bool, e.NumFloors)
	for {
		time.Sleep(PollRate)
		for f := 0; f < e.NumFloors; f++ {
			for b := ButtonType(0); b < 3; b++ {
				v := e.GetButton(b, f)
				if v != prev[f][b] && v != false {
					receiver <- ButtonEvent{f, ButtonType(b)}
				}
				prev[f][b] = v
			}
		}
	}
}

func (e *Ele) PollFloorSensor(receiver chan<- int) {
	prev := -1
	for {
		time.Sleep(PollRate)
		v := e.GetFloor()
		if v != prev && v != -1 {
			receiver <- v
		}
		prev = v
	}
}

func (e *Ele) PollStopButton(receiver chan<- bool) {
	prev := false
	for {
		time.Sleep(PollRate)
		v := e.GetStop()
		if v != prev {
			receiver <- v
		}
		prev = v
	}
}

func (e *Ele) PollObstructionSwitch(receiver chan<- bool) {
	prev := false
	for {
		time.Sleep(PollRate)
		v := e.GetObstruction()
		if v != prev {
			receiver <- v
		}
		prev = v
	}
}

func (e *Ele) GetButton(button ButtonType, floor int) bool {
	a := e.read([4]byte{6, byte(button), byte(floor), 0})
	return toBool(a[1])
}

func (e *Ele) GetFloor() int {
	a := e.read([4]byte{7, 0, 0, 0})
	if a[1] != 0 {
		return int(a[2])
	} else {
		return -1
	}
}

func (e *Ele) GetStop() bool {
	a := e.read([4]byte{8, 0, 0, 0})
	return toBool(a[1])
}

func (e *Ele) GetObstruction() bool {
	a := e.read([4]byte{9, 0, 0, 0})
	return toBool(a[1])
}

func Elevio_button_toString(eb ButtonType) string {
	if eb == BT_HallUp {
		return "BT_HallUp"
	} else if eb == BT_Cab {
		return "BT_Cab"
	} else if eb == BT_HallDown {
		return "BT_HallDown"
	} else {
		return "BT_UNDEFINED"
	}
}

func Elevio_dirn_toString(dr MotorDirection) string {
	if dr == MD_Down {
		return "D_Down"
	} else if dr == MD_Stop {
		return "D_Stop"
	} else if dr == MD_Up {
		return "D_Up"
	} else {
		return "DR_UNDEFINED"
	}
}
