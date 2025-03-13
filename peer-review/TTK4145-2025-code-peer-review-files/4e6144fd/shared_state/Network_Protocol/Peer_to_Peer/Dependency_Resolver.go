package peer_to_peer

import (
	"Constants"
	"fmt"
	"sync"
)

type Dependency_Resolver struct {
	mu sync.Mutex

	queue_list []Dependency
	queue_head int

	Saved_Messages map[Dependency]P2P_Message
}

func New_Dependency_Resolver() *Dependency_Resolver {
	return &Dependency_Resolver{
		queue_list: make([]Dependency, Constants.P2P_MSG_TIME_HORIZON),
		queue_head: 0,

		Saved_Messages: make(map[Dependency]P2P_Message, Constants.P2P_MSG_TIME_HORIZON),
	}
}

func (controller *Dependency_Resolver) advance_head() {
	controller.queue_head = (controller.queue_head + 1) % Constants.P2P_MSG_TIME_HORIZON
}

func (controller *Dependency_Resolver) Emplace_New_Message(message P2P_Message) {
	if message.Type != MESSAGE {
		return
	}

	controller.mu.Lock()
	defer controller.mu.Unlock()

	controller.advance_head()
	key := New_Dependency(message.Sender, message.Time)

	delete(controller.Saved_Messages, controller.queue_list[controller.queue_head])

	controller.queue_list[controller.queue_head] = key
	controller.Saved_Messages[key] = message
}

func (controller *Dependency_Resolver) Get_Message(dependency Dependency) (P2P_Message, bool) {
	controller.mu.Lock()
	defer controller.mu.Unlock()

	message, ok := controller.Saved_Messages[dependency]
	return message, ok
}

func (network *P2P_Network) handle_special_case(message P2P_Message) {
	switch message.Type {
	case REQUEST_MISSING_DEPENDENCY:
		requested_dependency := Dependency_From_String(message.Message)
		response, ok := network.dependency_resolver.Get_Message(requested_dependency)
		if ok {
			network.Send(response, message.Sender)
		} else {
			fmt.Printf("Error in special cases, a requested dependency was not found: %s", message.To_String())
		}
		// Other special cases?
	case MESSAGE:
		fmt.Printf("Error in special cases, a message got handled as special case: %s", message.To_String())
		return
	}
}
