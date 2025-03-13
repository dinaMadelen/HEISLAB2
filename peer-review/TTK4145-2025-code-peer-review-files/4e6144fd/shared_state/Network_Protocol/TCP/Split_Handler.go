package TCP

import (
	"Constants"
	"strings"
)

// Struct that handles the fact that TCP messages can be cut short in a read statement.
type TCP_Split_Handler struct {
	split_message_to_be_handled string
}

func (handler *TCP_Split_Handler) Split_Null_Terminated_Tcp_Message(tcp_message string) []string {
	split_messages := strings.Split(tcp_message, Constants.NULL)

	// Handle message that has been split due to buffer size or partial TCP transmission.
	//---
	// Add previous split message to the first message:
	split_messages[0] = handler.split_message_to_be_handled + split_messages[0]

	// Store next split message to be handled on next read:
	last_split_id := len(split_messages) - 1
	handler.split_message_to_be_handled = split_messages[last_split_id]

	//---
	// Return all except the incomplete split.
	return split_messages[0:last_split_id]
}

func (handler *TCP_Split_Handler) Make_Null_Terminated_TCP_Message(message string) string {
	return message + Constants.NULL
}

func New_TCP_Split_Handler() TCP_Split_Handler {
	return TCP_Split_Handler{
		split_message_to_be_handled: "",
	}
}
