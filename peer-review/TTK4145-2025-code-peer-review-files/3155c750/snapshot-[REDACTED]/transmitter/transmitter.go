package transmitter

import (
	"encoding/gob"
	"fmt"
	sharedData "root/SharedData"
	"net"
	"sync"
	"time"
)
var conn_lift1 net.Conn
//var conn_lift2 net.Conn

var sendMu sync.Mutex 



func Start_tcp_call(port string, ip string){
	var err error
	conn_lift1, err = net.Dial("tcp", ip+":"+port)//connects to the other elevatoe
	if err != nil {
		fmt.Println("Error connecting to pc:", ip, err)
		time.Sleep(5*time.Second)
		Start_tcp_call(port, ip)//trys again
	}

}


func Send_Elevator_data(data sharedData.Elevator_data) {
	sendMu.Lock() // Locking before sending
	defer sendMu.Unlock() // Ensure to unlock after sending
	time.Sleep(7*time.Millisecond)
	encoder := gob.NewEncoder(conn_lift1)
	err := encoder.Encode("elevator_data") // Type ID so the receiver kows what type of data to decode the next packat as 
	if err != nil {
		fmt.Println("Encoding error:", err)
		return
	}
	time.Sleep(7*time.Millisecond)
	err = encoder.Encode(data) //sendes the Elevator_data
	if err != nil {	
		fmt.Println("Error encoding data:", err)
		return
	}

}

func Send_update(update [3]int){
	sendMu.Lock() // Locking before sending
	defer sendMu.Unlock() // Ensure to unlock after sending
	time.Sleep(7*time.Millisecond)
	encoder := gob.NewEncoder(conn_lift1)
	err := encoder.Encode("int") // Type ID so the receiver kows what type of data to decode the next packat as 
	if err != nil {
		fmt.Println("Encoding error:", err)
		return
	}
	time.Sleep(7*time.Millisecond)
	err = encoder.Encode(update) //sendes the update
	if err != nil {
		fmt.Println("Error encoding data:", err)
		return
	}
}





