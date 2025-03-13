package network

import (
	"bufio"
	"config"
	"log"
	"net"
	"strconv"
	"strings"
	"time"
	"datatype"
	"io"
	"fmt"
	"color"
)

func Start_server(connLoss chan<- *net.TCPConn, connNew chan<- *net.TCPConn, msgChan chan<- Message) {
	fmt.Print("Starting server... \n")
	
	go Broadcast_server_address()

	addr, err := net.ResolveTCPAddr("tcp", ":" + strconv.Itoa(config.TCP_port))
	if err != nil {
		log.Fatalf(color.Red + "Failed to resolve TCP address: %v" + color.Reset, err)
	}

	listener, err := net.ListenTCP("tcp", addr)
	if err != nil {
		log.Fatalf(color.Red + "Failed to start server: %v" + color.Reset, err)
	}
	defer listener.Close()

	for {
		conn, err := listener.AcceptTCP()
		if err != nil {
			log.Printf(color.Red + "Failed to accepting connection: %v" + color.Reset, err)
			continue
		}

		// Enable keep-alive for connection
		conn.SetKeepAlive(true)
		conn.SetKeepAlivePeriod(config.Keep_alive_period)

		// Handle client
		connNew <- conn
		go Listen_for_message(conn, msgChan, connLoss)
	}
}

func Connect_to_server(serverAddr string) *net.TCPConn {
	addr, err := net.ResolveTCPAddr("tcp", serverAddr + ":" + strconv.Itoa(config.TCP_port))
	if err != nil {
		log.Fatalf(color.Red + "Failed to resolve server address: %v" + color.Reset, err)
	}

	conn, err := net.DialTCP("tcp", nil, addr)
	if err != nil {
		log.Fatalf(color.Red + "Failed to connect to server: %v" + color.Reset, err)
	}

	// Enable keep-alive on the connection
	conn.SetKeepAlive(true)
	conn.SetKeepAlivePeriod(config.Keep_alive_period)

	return conn
}

func Send_message(conn *net.TCPConn, header string, payload datatype.DataPayload) {
	// Create write buffer
	writer := bufio.NewWriter(conn)

	// Message to JSON
	message, err := Encode_message(header, payload)
	if err != nil {
		log.Fatalf(color.Red + "Failed to serialize payload: %v" + color.Reset, err)
	}

	_, err = writer.WriteString(string(message) + "\n")
	if err != nil {
		log.Fatalf(color.Red + "Failed to send message: %v" + color.Reset, err)
	}

	writer.Flush()
}

func Listen_for_message(conn *net.TCPConn, msgChan chan<- Message, connLoss chan<- *net.TCPConn) {
	// Create read buffer
	reader := bufio.NewReader(conn)

	for {
		messageData, err := reader.ReadString('\n')
		if err != nil {
			if err == io.EOF {
				log.Printf(color.Orange + "Client %s disconnected \n" + color.Reset, conn.RemoteAddr().String())
			} else if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
				log.Printf(color.Orange + "Read timeout for client %s" + color.Reset, conn.RemoteAddr().String())
			} else {
				log.Printf(color.Red + "Failed to read message from %s: %v" + color.Reset, conn.RemoteAddr().String(), err)
			}
			
			connLoss<- conn
			conn.Close()
			return
		}

		messageData = strings.TrimSpace(messageData)
		message, err := Decode_message([]byte(messageData))
		if err != nil {
			log.Printf(color.Red + "Failed to decode message: %v" + color.Reset, err)
		}

		message.Addr = Get_addr_from_conn(conn)

		msgChan <- message
	}
}

func Ping_google() bool {
	_, err := net.DialTimeout("tcp", "8.8.8.8:53", 2*time.Second)
	return err == nil
}

func Get_addr_from_conn(conn *net.TCPConn) string {
	addr := (*conn).RemoteAddr()

	// assert to *net.TCPAddr to access the IP and port
	tcpAddr, ok := addr.(*net.TCPAddr); 
	if !ok {
		log.Fatalf(color.Red + "Connection is not a TCP connection. \n" + color.Reset)
	}

	return tcpAddr.IP.String() + ":" + strconv.Itoa(tcpAddr.Port)
}
