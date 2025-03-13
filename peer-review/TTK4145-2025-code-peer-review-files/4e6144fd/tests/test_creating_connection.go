package tests

import (
	"Network-Protocol/TCP"
	"Network-Protocol/UDP"
	"fmt"
	"math/rand"
	"time"
)

func howl_address(address string, channel UDP.UDP_Channel, kill chan bool) {
	for {
		select {
		case <-kill:
			return

		default:
			channel.Broadcast(address)
		}

		time.Sleep(time.Second)
	}
}

func accept_connections(channel UDP.UDP_Channel, connection_manager *TCP.TCP_Connection_Manager, kill chan bool) {
	defer connection_manager.Close_All()

	var known_connections map[string]bool = make(map[string]bool)

	var timer_stop_reading *time.Timer = time.NewTimer(time.Minute)
	timer_stop_reading.Stop()

	timer_start_reading := time.NewTimer(time.Second)

	for {
		select {
		case <-timer_start_reading.C:
			err := channel.Start_Reading()

			if err == nil {
				fmt.Print("Started reading UDP!\n")
				timer_stop_reading.Reset(3 * time.Second)
			} else {
				timer_start_reading.Reset(time.Duration(rand.Intn(1000000)) + time.Millisecond)
			}

		case <-timer_stop_reading.C:
			channel.Stop_Reading()
			fmt.Print("Stopped reading UDP!\n")
			timer_start_reading.Reset(time.Duration(rand.Intn(10000)) + 5*time.Second)

		case message := <-channel.Read_Channel:
			fmt.Printf("Received UDP broadcast message: %s\n", message)

			_, ok := known_connections[message]
			if !ok {
				known_connections[message] = true
				connection_manager.Connect_Client(message)
			}

		case <-kill:
			return
		}

		time.Sleep(time.Microsecond)
	}
}

func listener_sender_connection_manager(port string, connection_manager *TCP.TCP_Connection_Manager, kill chan bool) {
	ticker_send_status := time.NewTicker(time.Second * 5)

	for {
		select {
		case message := <-connection_manager.Global_Read_Channel:
			fmt.Printf("Received: %s\n", message)
		case <-ticker_send_status.C:
			fmt.Printf("There are currently %d connections\n", len(connection_manager.Connections))

			connection_manager.Broadcast("Hello all from " + UDP.Get_local_IP().String() + ":" + port + "!")
		case <-kill:
			return
		default:
		}

		time.Sleep(time.Microsecond)
	}
}

// Expected to run this test in multiple instances.
func Test_Creating_Connection(port string) {
	var broadcast_channel UDP.UDP_Channel = UDP.New_UDP_Channel()

	var tcp_connection_manager TCP.TCP_Connection_Manager = *TCP.New_TCP_Connection_Manager()
	tcp_connection_manager.Open_Server(port)

	fmt.Printf("Local IP: %s\n", UDP.Get_local_IP().String())

	kill_channel := make(chan bool)

	go howl_address(UDP.Get_local_IP().String()+":"+port, broadcast_channel, kill_channel)
	go accept_connections(broadcast_channel, &tcp_connection_manager, kill_channel)
	go listener_sender_connection_manager(port, &tcp_connection_manager, kill_channel)

	time.Sleep(time.Minute)
	close(kill_channel)
	time.Sleep(time.Second)
}
