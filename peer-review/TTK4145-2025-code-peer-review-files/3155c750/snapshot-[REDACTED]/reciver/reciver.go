package reciver

import (
	"encoding/gob"
	"fmt"
	"net"
	"time"
	"root/SharedData"

	
)

var lis_lift1 net.Conn
//var lis_lift2 net.Conn

var a = []bool{true, false, false, false}
var data = sharedData.Elevator_data{Behavior: "doorOpen",Floor: 0,Direction: "down",CabRequests: a}
func Start_tcp_listen(port string) {
	ln, err := net.Listen("tcp", ":"+port)
	if err != nil {
		fmt.Println("Error starting listen:", err)
	}
	lis_lift1, err = ln.Accept()
	if err != nil {
		fmt.Println("Error accepting connection:", err)
	}

}

func Listen_recive(receiver chan<- [3]int) {
	for {
		Decode(receiver)
	}
}

func Decode(receiver chan<- [3]int) {
	decoder := gob.NewDecoder(lis_lift1)

	var typeID string
	err := decoder.Decode(&typeID) // Read type identifier to kono what type of data to decode next
	if err != nil {
		fmt.Println("Error decoding type:", err)
		time.Sleep(1*time.Second)
		return
	}

	switch typeID {//chooses what decoder to use based on what type that needs to be decoded 
	case "elevator_data":
		var data sharedData.Elevator_data

		err = decoder.Decode(&data)
		if err != nil {
			fmt.Println("Error decoding Elevator_data:", err)
	
			return
		}
		if data.Floor != -1 && !(data.Floor == 0 && data.Direction == "down") && !(data.Floor == 3 && data.Direction == "up") {//stops the elavator data form crashing the assigner 
		sharedData.ChangeRemoteElevatorData(data)
		//fmt.Println("Received Elevator_data:", data)
		}
			
		//fmt.Println("Received Elevator_data:", data)
		


	case "int":
		var num [3]int
		err = decoder.Decode(&num)
		if err != nil {
			fmt.Println("Error decoding int:", err)
			return
		}
		//fmt.Println("Received int:", num)
		receiver <- num //sends signal to main that hall requests have been updated and that the lights need to be updated

	default:
		fmt.Println("Unknown type received:", typeID)
	}
}

func GetRemoteElevatorData() sharedData.Elevator_data {
	return data
}
