package network

import (
	"datatype"
	"encoding/json"
)

type Message struct {
	Header  string      			`json:"header"`
	Payload datatype.DataPayload	`json:"payload,omitempty"`
	Addr	string					`json:"addr,omitempty"`
}

func Encode_message(header string, payload datatype.DataPayload) ([]byte, error) {
	message := Message{
		Header:  header,
		Payload: payload,
	}
	return json.Marshal(message)
}

func Decode_message(data []byte) (Message, error) {
	// Split message into header and payload
	var message Message
	err := json.Unmarshal(data, &message)
	if err != nil {
		return Message{}, err
	}
	return message, nil
}
