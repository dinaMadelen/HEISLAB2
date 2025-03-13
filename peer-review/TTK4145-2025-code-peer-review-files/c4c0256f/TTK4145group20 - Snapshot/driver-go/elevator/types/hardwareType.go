package types

type MotorDirection int 
const (
	MD_Up MotorDirection = 1
	MD_Down MotorDirection = -1
	MD_Stop MotorDirection = 0
)

type ButtonType int
const (
	BT_HallUp ButtonType = iota
	BT_HallDown
	BT_Cab
)

type ButtonEvent struct{
	Floor int
	Button ButtonType
}