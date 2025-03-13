package utilities

import (
	. "Driver-go/elevator"
	"encoding/json"
	"fmt"
)

var message ElevatorMessage

// Tag representerer de ulike meldingskategoriene som kan sendes over nettverket
type Tag string

const (
	Acknowledgement Tag = "Acknowledgement" //Bekreftelse på mottatt melding
	ButtonPress     Tag = "ButtonPress"     //Når en knapp blir trykket
	HeartbeatSlave  Tag = "HeartbeatSlave"  //Statusmelding fra slave
	HeartbeatBackup Tag = "HeartbeatBackup" //Statusmelding fra backup
	HeartbeatMaster Tag = "HeartbeatMaster" //Statusmelding fra master
)

// ElevatorMessage representererer hele "Verdensbilde" som sendes over nettverket
type ElevatorMessage struct {
	Tag        Tag       `json:"tag"`
	Checkpoint int       `json:"checkpoint"`
	ElevatorID int       `json:"elevatorid,omitempty"` //Hvilken heis som sender meldingen
	Floor      int       `json:"floor,omitempty"`      //Hvilken etasje som er "bestilt"
	Button     int       `json:"button,omitempty"`     //Hvilken knapp som er trykket
	State      Elevator  `json:"state,omitempty"`
	HallCalls  HallCalls `json:"hallcalls,omitempty"`
}

// Returnerer en JSON-string som skal sendes som bekreftelse på at melding er mottatt
func UtilitiesJsonAcknowledgement(tag Tag, checkpoint int, elevatorID int) (string, error) {
	message = ElevatorMessage{Tag: tag, ElevatorID: elevatorID}
	jsonData, err := json.Marshal(message)
	if err != nil {
		return "", err
	}

	return string(jsonData), nil
}

// Returnerer en JSON-string som inneholder info om ny bestilling
func UtilitiesJsonButtonPress(tag Tag, floor int, button int) (string, error) {
	message = ElevatorMessage{Tag: tag, Floor: floor, Button: button}
	jsonData, err := json.Marshal(message)
	if err != nil {
		return "", err
	}

	return string(jsonData), nil
}

// Returnerer en JSON-string som inneholder "Verdensbilde"
func UtilitiesJsonHeartbeat(tag Tag, checkpoint int, elevatorID int, e Elevator, floor int, button int, hallcalls HallCalls)(string, error) {
	message = ElevatorMessage{
		Tag:        tag,
		Checkpoint: checkpoint,
		ElevatorID: elevatorID,
		Floor:      floor,
		Button:     button,
		HallCalls: hallcalls,
		State:      e,
	}

	jsonData, err := json.Marshal(message)
	if err != nil {
		return "", err
	}

	return string(jsonData), nil
}

//Leser en JSON-string og dekoder den til en ElevatorMessage-struktur

func UtilitiesRecieveJsonString(jsonStr string) (ElevatorMessage, error) {
	var message ElevatorMessage

	err := json.Unmarshal([]byte(jsonStr), &message)

	if err != nil {

		fmt.Println("Feil ved parsing", err)

	}

	return message, nil
}
