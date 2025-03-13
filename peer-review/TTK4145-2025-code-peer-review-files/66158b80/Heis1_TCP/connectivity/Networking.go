package connectivity

//https://pkg.go.dev/net#KeepAliveConfig
import (
	"Driver-go/elevio"
	"Driver-go/fsm"
	"bytes"
	"encoding/binary"
	"encoding/gob"
	"fmt"
	"log"
	"net"
	"sync"
	"time"
)

const (
//Device ID, to make it use right ports and ip. For easyer development
//starts at 0

)

var (

	// What ID will given id listen to and dial to
	//row listens to column. column dailer til row

	TCP_world_view_send_ips_matrix [NR_OF_ELEVATORS - 1][NR_OF_ELEVATORS]string
	listen_dail_conn_matrix        [NR_OF_ELEVATORS - 1][NR_OF_ELEVATORS]net.Conn
	trying_to_setup_matrix         = [NR_OF_ELEVATORS - 1][NR_OF_ELEVATORS]bool{}
	rescever_running_matrix        = [NR_OF_ELEVATORS - 1][NR_OF_ELEVATORS]bool{}

	//these are gonna look like the matrixes above

	// Mutex for the matrixes
	mu_world_view_send_ips_matrix sync.Mutex
	mu_listen_dail_conn_matrix    sync.Mutex
	mu_trying_to_setup_matrix     sync.Mutex
	mu_rescever_running_matrix    sync.Mutex

	// World view sending UDP connection setup
	// Elevator (0-1, 1-2, 2-1), first is dialing, second is listening
)

// // World view sending TCP connection setup
func init() { // runs when imported

	//flag.IntVar(&ID, "id", ID, "Spesefy the id with -id")
	//flag.Parse()

	for i := 0; i < NR_OF_ELEVATORS-1; i++ {
		for j := i + 1; j < NR_OF_ELEVATORS; j++ {
			ip := "localhost:80" + fmt.Sprint(i) + fmt.Sprint(j)
			set_TCP_world_view_send_ips_matrix(i, j, ip)
		}
	}
}

func get_TCP_world_view_send_ips_matrix(i int, j int) string {
	mu_world_view_send_ips_matrix.Lock()
	defer mu_world_view_send_ips_matrix.Unlock()
	return TCP_world_view_send_ips_matrix[i][j]
}

func set_TCP_world_view_send_ips_matrix(i int, j int, ip string) {
	mu_world_view_send_ips_matrix.Lock()
	defer mu_world_view_send_ips_matrix.Unlock()
	TCP_world_view_send_ips_matrix[i][j] = ip
}

func get_listen_dail_conn_matrix(i int, j int) net.Conn {
	mu_listen_dail_conn_matrix.Lock()
	defer mu_listen_dail_conn_matrix.Unlock()
	return listen_dail_conn_matrix[i][j]
}

func set_listen_dail_conn_matrix(i int, j int, conn net.Conn) {
	mu_listen_dail_conn_matrix.Lock()
	defer mu_listen_dail_conn_matrix.Unlock()
	listen_dail_conn_matrix[i][j] = conn
}

func get_trying_to_setup_matrix(i int, j int) bool {
	mu_trying_to_setup_matrix.Lock()
	defer mu_trying_to_setup_matrix.Unlock()
	return trying_to_setup_matrix[i][j]
}

func set_trying_to_setup_matrix(i int, j int, b bool) {
	mu_trying_to_setup_matrix.Lock()
	defer mu_trying_to_setup_matrix.Unlock()
	trying_to_setup_matrix[i][j] = b
}

func get_rescever_running_matrix(i int, j int) bool {
	mu_rescever_running_matrix.Lock()
	defer mu_rescever_running_matrix.Unlock()
	return rescever_running_matrix[i][j]
}

func set_rescever_running_matrix(i int, j int, b bool) {
	mu_rescever_running_matrix.Lock()
	defer mu_rescever_running_matrix.Unlock()
	rescever_running_matrix[i][j] = b
}

// Serialize the struct
func SerializeElevator(wv Worldview_package) ([]byte, error) {
	var buf bytes.Buffer
	enc := gob.NewEncoder(&buf)
	err := enc.Encode(wv)
	return buf.Bytes(), err
}

func DeserializeElevator(data []byte) (Worldview_package, error) {
	var wv Worldview_package
	buf := bytes.NewBuffer(data)
	dec := gob.NewDecoder(buf)
	err := dec.Decode(&wv)
	return wv, err
}

func TCP_receving_setup(TCP_receive_channel chan Worldview_package) {
	fmt.Println("Starting TCP receving setup")
	for { //for loop to keep the function running
		//Server setup
		for j := ID + 1; j < NR_OF_ELEVATORS; j++ {
			//Server_conn_setup
			if !get_trying_to_setup_matrix(ID, j) && !IsOnline(j) {
				fmt.Println("Starting up TCP_server_setup. ", ID, "listening for: ", j)
				go TCP_server_setup(j)
			}

			//Server rescever setup
			if IsOnline(j) && !get_rescever_running_matrix(ID, j) {
				fmt.Println("Starting handle_receive for connected elevator")
				go handle_receive(get_listen_dail_conn_matrix(ID, j), TCP_receive_channel, j, ID, j)
			}
		}

		//Client setup
		for i := 0; i < ID; i++ {
			//Client_conn_setup
			if !get_trying_to_setup_matrix(i, ID) && !IsOnline(i) {
				fmt.Println("Starting up TCP_client_setup. ", ID, "dialing to: ", i)
				go TCP_client_setup(i)
			}

			//Client rescever setup
			if IsOnline(i) && !get_rescever_running_matrix(i, ID) {
				fmt.Println("Starting handle_receive for elevator we are connected to")
				go handle_receive(get_listen_dail_conn_matrix(i, ID), TCP_receive_channel, i, i, ID)
			}

		}
		time.Sleep(2 * time.Second)

	}

}

func TCP_server_setup(incoming_e_ID int) {
	//Setting up serfor for ID to listen to incoming_e_ID
	set_trying_to_setup_matrix(ID, incoming_e_ID, true)

	server_ip := get_TCP_world_view_send_ips_matrix(ID, incoming_e_ID)

	fmt.Println("Server listening on ip: ", server_ip)
	ln, err := net.Listen("tcp", server_ip)
	if err != nil {
		fmt.Println("Error in TCP_server_setup", server_ip, err)
	}

	fmt.Println("Waiting for Accept:", server_ip)
	conn, err := ln.Accept()
	if err != nil {
		fmt.Println("Error in TCP_server_setup:", server_ip, err)
	}

	//Set no delay til true

	fmt.Println("Elevator ", incoming_e_ID, " connected to elevator ", ID, ". Setting ", incoming_e_ID, " to online")

	set_listen_dail_conn_matrix(ID, incoming_e_ID, conn)

	SetElevatorOnline(incoming_e_ID)

	set_trying_to_setup_matrix(ID, incoming_e_ID, false)

	defer ln.Close()
}

func TCP_client_setup(e_dailing_to_ID int) {
	set_trying_to_setup_matrix(e_dailing_to_ID, ID, true)

	client_ip := get_TCP_world_view_send_ips_matrix(e_dailing_to_ID, ID)

	for {
		fmt.Printf("Trying to dail to ip: %s\n", client_ip)
		conn, err := net.Dial("tcp", client_ip)
		if err != nil {
			fmt.Println("Dailing id ", e_dailing_to_ID, "failed, retrying in 2 seconds...")
			time.Sleep(2 * time.Second)
			continue
		}

		fmt.Println("Connected to", client_ip)

		set_listen_dail_conn_matrix(e_dailing_to_ID, ID, conn)

		SetElevatorOnline(e_dailing_to_ID) //setting status of connected elevator to online

		break
	}
	set_trying_to_setup_matrix(e_dailing_to_ID, ID, false)
}

func handle_receive(conn net.Conn, TCP_receive_channel chan Worldview_package, ID_of_connected_elevator int, i int, j int) {
	defer conn.Close()
	set_rescever_running_matrix(i, j, true)

	fmt.Println("HANDLE RECEIVE STARTED, ID: " + fmt.Sprint(ID_of_connected_elevator))
	for {
		// Replace with actual receiving logic

		err := conn.SetReadDeadline(time.Now().Add(TIMEOUT * time.Second))
		if err != nil {
			fmt.Println("Conn not open")
			set_rescever_running_matrix(i, j, false)
			return
		}
		var packetLength uint32
		err = binary.Read(conn, binary.BigEndian, &packetLength)
		if err != nil {
			fmt.Println("failed to read packetLength:", err)
			set_rescever_running_matrix(i, j, false)
			return
		}
		buffer := make([]byte, packetLength)
		_, err = conn.Read(buffer)
		if err != nil {
			fmt.Println("Error receiving or timedout, closing receive goroutine and conn")
			SetElevatorOffline(ID_of_connected_elevator) //setting status of connected elevator to offline
			set_rescever_running_matrix(i, j, false)
			return
		}
		//fmt.Println("DATA MOTATT! ")

		// Remove padding before deserializing
		//trimmedData := bytes.TrimRight(buffer, "\x00")

		//fmt.Printf("trimmedData: %x\n", trimmedData)

		//deserialize the buffer to worldview package
		receved_world_view_package, err := DeserializeElevator(buffer)
		if err != nil {
			log.Fatal("failed to deserialize:", err)
		}

		//Store backup worldview from incomming elevator
		Store_worldview(receved_world_view_package.Elevator_ID, receved_world_view_package)

		TCP_receive_channel <- receved_world_view_package
	}

}

func Send_world_view() {
	send_world_view_package := New_Worldview_package(ID, fsm.GetElevatorStruct())
	serialized_world_view_package, err := SerializeElevator(send_world_view_package)
	if err != nil {
		log.Fatal("failed to serialize:", err)
	}

	if len(serialized_world_view_package) > PACKAGE_SIZE {
		log.Fatal("error: serialized data too large")
	}

	// Pad data with zeros to make it exactly 1024 bytes
	//paddedData := make([]byte, PACKAGE_SIZE)
	//copy(paddedData, serialized_world_view_package)

	//Finding package length
	packetLength := uint32(len(serialized_world_view_package)) //uint32 is 4 bytes

	// Sending to all online that are connected to us
	for connected_e_ID := ID + 1; connected_e_ID < NR_OF_ELEVATORS; connected_e_ID++ {
		if IsOnline(connected_e_ID) {

			//sending first packetLength, before actual packet. Preventing packet stacking
			server_conn := get_listen_dail_conn_matrix(ID, connected_e_ID)
			err = binary.Write(server_conn, binary.BigEndian, packetLength)
			if err != nil {
				fmt.Println("Error sending packetlength to connected elevator, connection lost.")
				SetElevatorOffline(connected_e_ID) //setting status of connected elevator to offline
			}

			//writing acctual package
			_, err = server_conn.Write(serialized_world_view_package)
			if err != nil {
				fmt.Println("Error sending, connection lost.")
				SetElevatorOffline(connected_e_ID) //setting status of connected elevator to offline
			} else {
				//fmt.Println("Succes sending worldview")
			}
		}
	}
	for connected_e_ID := 0; connected_e_ID < ID; connected_e_ID++ {
		if IsOnline(connected_e_ID) {
			//sending first packetLength, before actual packet. Preventing packet stacking
			client_conn := get_listen_dail_conn_matrix(connected_e_ID, ID)
			err = binary.Write(client_conn, binary.BigEndian, packetLength)
			if err != nil {
				fmt.Println("Error sending packetlength to connected elevator, connection lost.")
				SetElevatorOffline(connected_e_ID) //setting status of connected elevator to offline
			}

			//writing acctual package
			_, err = client_conn.Write(serialized_world_view_package)
			if err != nil {
				fmt.Println("Error sending, connection lost.")
				SetElevatorOffline(connected_e_ID) //setting status of connected elevator to offline
			}
		}
	}
}

func Send_order_to_spesific_elevator(recever_e int, order elevio.ButtonEvent) bool {
	fmt.Println("Inside send order to spesific elevator!")
	//find correct conn
	var conn net.Conn
	if IsOnline(recever_e) {

		if get_listen_dail_conn_matrix(ID, recever_e) != nil {
			conn = get_listen_dail_conn_matrix(ID, recever_e)
			fmt.Println("Here1")

		} else if get_listen_dail_conn_matrix(recever_e, ID) != nil {
			conn = get_listen_dail_conn_matrix(recever_e, ID)
			fmt.Println("Here2")
		} else {
			fmt.Println("No valid conn to send ORDER")
			fmt.Println("Here3")
			return false
		}
	} else {
		fmt.Println("Here4")
		return false
	}
	fmt.Println("Here5")
	//Try to send order to that conn
	a := fsm.GetElevatorStruct()
	fmt.Println("After get elv struct")

	_ = New_Worldview_package(ID, a)
	fmt.Println("After get WV")

	send_world_view_package := New_Worldview_package(ID, fsm.GetElevatorStruct())
	send_world_view_package.Order_bool = true
	send_world_view_package.Order = order

	fmt.Println("Here6")
	serialized_world_view_package, err := SerializeElevator(send_world_view_package)
	if err != nil {
		log.Fatal("failed to serialize:", err)
	}
	fmt.Println("Here7")

	// Set the write deadline for both write operations (2 seconds)
	err = conn.SetWriteDeadline(time.Now().Add(TIMEOUT * time.Second))
	if err != nil {
		fmt.Println("Failed to set write deadline:", err)
		return false
	}
	fmt.Println("Here8")

	//Finding package length
	packetLength := uint32(len(serialized_world_view_package)) //uint32 is 4 bytes

	err = binary.Write(conn, binary.BigEndian, packetLength)
	if err != nil {
		fmt.Println("Error sending packetlength for ORDER to connected elevator, connection lost or timedout.")
		return false
	}
	fmt.Println("Success sending ORDER packetlength")

	//writing acctual package
	_, err = conn.Write(serialized_world_view_package)
	if err != nil {
		fmt.Println("Error sending ORDER, connection lost.  or timedout")
		return false
	}
	//disable SetwriteDeadline
	conn.SetWriteDeadline(time.Time{})
	fmt.Println("Success sending ORDER")

	//everyting worked!
	return true
}
