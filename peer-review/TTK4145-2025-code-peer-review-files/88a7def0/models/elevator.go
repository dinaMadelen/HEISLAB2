package models

type ElevatorState struct {
	Id        Id
	Floor     int
	Behavior  ElevatorBehavior
	Direction MotorDirection
}

type ElevatorBehavior int

const (
	Idle ElevatorBehavior = iota
	DoorOpen
	Moving
)

type Orders [][3]bool

type MotorDirection int

const (
	Up   MotorDirection = 1
	Down                = -1
	Stop                = 0
)

type ButtonType int

const (
	HallUp   ButtonType = 0
	HallDown            = 1
	Cab                 = 2
)
