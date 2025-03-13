package TCP

import (
	"Constants"
	"fmt"
	"io"
	"time"
)

// Protected functions

// Reads a TCP message from the TCP Connection and puts all detected messages in the TCP message onto the Read Channel.
// Handles splitting messages using the TCP Split Handler.
func (TCP_Connection *TCP_Connection) read() {
	deadline := time.Now().Add(Constants.TCP_READ_DEADLNE)
	TCP_Connection.connection.SetReadDeadline(deadline)

	data := make([]byte, 4096)

	bytes_received, err := TCP_Connection.connection.Read(data)

	if err == io.EOF {
		// Connection terminated from the other side. Disconnect.
		TCP_Connection.Close()
	}

	if err == nil {
		message := string(data[0:bytes_received])

		split_messages := TCP_Connection.split_handler.Split_Null_Terminated_Tcp_Message(message)

		for _, split_message := range split_messages {
			TCP_Connection.Read_Channel <- split_message
		}
	}
}

// Writes a string onto the TCP Connection, function handles necessary
func (TCP_Connection *TCP_Connection) write(message string) {
	deadline := time.Now().Add(Constants.TCP_READ_DEADLNE)
	TCP_Connection.connection.SetWriteDeadline(deadline)

	tcp_message := TCP_Connection.split_handler.Make_Null_Terminated_TCP_Message(message)
	data := []byte(tcp_message)

	_, err := TCP_Connection.connection.Write(data)

	if err != nil {
		fmt.Println("Write didn't succeed, error: ", err)
	}
}

// Handles a TCP Connection, writing any data from the Write Channel onto the connection.
// And reads any data from the connection onto the Read Channel.
//
// Will self-remove from the connection manager should close be called.
func (connection *TCP_Connection) handle_TCP_Connection(connection_manager *TCP_Connection_Manager) {
	defer connection.connection.Close()
	defer connection_manager.Remove_Connection(*connection)

	when_to_read_ticker := time.NewTicker(Constants.TCP_WAIT_BEFORE_READING_AGAIN)

	for {
		select {
		case <-when_to_read_ticker.C:
			connection.read()

		case message := <-connection.Write_Channel:
			connection.write(message)

		case <-connection.close_channel:
			return
		}

		time.Sleep(time.Microsecond)
	}
}
