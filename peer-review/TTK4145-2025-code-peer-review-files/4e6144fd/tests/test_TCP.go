package tests

import (
	"Constants"
	"fmt"
	"time"

	"Network-Protocol/TCP"
)

func listener(connection TCP.TCP_Connection, kill chan bool) {
	for {
		select {
		case message := <-connection.Read_Channel:
			fmt.Printf("%s received message: %s\n", connection.Get_Name(), message)
		case <-kill:
			return
		}
	}
}

func sender(connection TCP.TCP_Connection, kill chan bool) {
	ticker_send := time.NewTicker(time.Second)

	for {
		select {
		case <-ticker_send.C:
			connection.Write_Channel <- connection.Get_Name()
		case <-kill:
			return
		}
	}
}

func Test_TCP() {
	var connection_manager TCP.TCP_Connection_Manager = *TCP.New_TCP_Connection_Manager()

	connection_manager.Open_Server(Constants.TCP_PORT)

	<-time.After(time.Second)
	connection_manager.Connect_Client("192.168.50.70" + Constants.TCP_PORT)

	<-time.After(time.Second * 5)

	timer_kill := time.NewTimer(time.Minute)
	kill_channel := make(chan bool)

	for _, connection := range connection_manager.Connections {
		go listener(connection, kill_channel)
		go sender(connection, kill_channel)

		fmt.Printf("Added %s\n", connection.Get_Name())
	}

	<-timer_kill.C
	close(kill_channel)
}
