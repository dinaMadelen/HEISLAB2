package config

import(
	"time"
)
const N_floors int = 4
const N_buttons int = 3

type ElevatorConfig int

const (

	ElevatorPeerPort ElevatorConfig = 16572
	ElevatorRxPort ElevatorConfig = 16569
	ElevatorTxPort ElevatorConfig = 16570
)

//Denne er usikkert om jeg har implementert riktig, men det funker ikke helt på en annen måte. 
type TimeVariables time.Duration

const (
	MaxDuration time.Duration = 1<<63 - 1
	MasterTimeout time.Duration = time.Second
	MessageTimeout time.Duration = 200 * time.Millisecond
	MasterMessageTimeout time.Duration = 20 * time.Millisecond
)

type MasterConfig int

const(
	MasterRxPort MasterConfig = 16570
	MasterTxPort MasterConfig = 16569
	MasterRxPeerPort MasterConfig = 16572
)


