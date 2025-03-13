package models

type RequestMessage struct {
	Source  Id
	Request Request
}
type Request struct {
	Origin Origin
	Status RequestStatus
}
type Origin struct {
	Source     Source
	Floor      int
	ButtonType ButtonType
}
type Source interface {
	isSource()
}

type Hall struct{}

func (Hall) isSource() {}

type Elevator struct {
	Id Id
}

func (Elevator) isSource() {}

type Id uint8

type RequestStatus int

const (
	Absent RequestStatus = iota
	Unconfirmed
	Confirmed
	Unknown
)
