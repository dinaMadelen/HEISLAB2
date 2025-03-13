package peer_to_peer

import (
	"Constants"
	"strconv"
	"testing"
	"time"
)

func Test_Dependency_Horizon(t *testing.T) {
	handler := New_Dependency_Handler()
	clock := New_Lamport_Clock()

	owner := "OWNER"

	clock.Event()

	first_dependency := Dependency{owner, clock}

	// 1 more than dependency horizon, first dependency on list should be 1, but should be gone after crossing horizon.
	for i := 0; i < Constants.P2P_DEP_TIME_HORIZON+1; i++ {
		dependency := Dependency{owner, clock}

		handler.Add_Dependency(dependency)

		if !handler.Has_Dependency(dependency) {
			t.Fatal("Handler did not add dependency to list!\n")
		}

		clock.Event()
	}

	if handler.Has_Dependency(first_dependency) {
		t.Fatalf("First dependency was not removed! Map length: %d, Heap length: %d\n", len(handler.lookup_map), handler.min_heap.Len())
	}

	second_dependency := Dependency{owner, New_Lamport_Clock_From_String("2")}

	if !handler.Has_Dependency(second_dependency) {
		t.Fatalf("Second dependency was removed! Map length %d, Heap length %d\n", len(handler.lookup_map), handler.min_heap.Len())
	}

}

func Test_Lamport_Clock_Wraparound(t *testing.T) {
	clock_high := Lamport_Clock{Constants.LAMPORT_CLOCK_WRAPAROUND_UPPER_EDGE + 1}
	clock_low := Lamport_Clock{Constants.LAMPORT_CLOCK_WRAPAROUND_LOWER_EDGE - 1}

	if !clock_high.Is_Less_Than(clock_low) {
		t.Fatal("Wraparound clock is not returning true on wraparound!")
	}
}

func Test_Dependency_Wraparound(t *testing.T) {
	handler := New_Dependency_Handler()
	clock := New_Lamport_Clock()

	clock.time = Constants.LAMPORT_CLOCK_WRAPAROUND_UPPER_EDGE + 1

	low_time := Constants.LAMPORT_CLOCK_WRAPAROUND_LOWER_EDGE - Constants.P2P_DEP_TIME_HORIZON // Avoid cyclical dependency

	for i := 0; i < Constants.P2P_DEP_TIME_HORIZON+1; i++ {
		dependency := Dependency{strconv.Itoa(clock.time), clock}

		handler.Add_Dependency(dependency)

		if !handler.Has_Dependency(dependency) {
			t.Fatal("Handler did not add dependency to list!\n")
		}

		clock.Event()
	}

	clock.time = low_time

	for i := 0; i < Constants.P2P_DEP_TIME_HORIZON+1; i++ {
		dependency := Dependency{strconv.Itoa(clock.time), clock}

		handler.Add_Dependency(dependency)
		old_dependency := Dependency{strconv.Itoa(clock.time - 1), Lamport_Clock{clock.time - 1}}

		if !handler.Has_Dependency(dependency) {
			t.Fatal("Handler did not add dependency to list!\n")
		}

		// Old dependency doesn't exist for i = 0.
		if i != 0 && !handler.Has_Dependency(old_dependency) {
			t.Fatalf("Handler did not keep the new dependency: %s!\n", old_dependency.To_String())
		}

		clock.Event()
	}
}

func Test_P2P_Message_String(t *testing.T) {
	sender_field := "SENDER"
	type_field := MESSAGE
	time_field := New_Lamport_Clock_From_String("6")
	dependency_field := New_Dependency("OTHER", New_Lamport_Clock_From_String("3"))
	body_field := "Hello from body!"

	test_tcp_message :=
		sender_field + Constants.P2P_FIELD_DELIMINATOR +
			string(type_field) + Constants.P2P_FIELD_DELIMINATOR +
			time_field.String() + Constants.P2P_FIELD_DELIMINATOR +
			dependency_field.To_String() + Constants.P2P_FIELD_DELIMINATOR +
			body_field

	p2p_message := P2P_Message_From_String(test_tcp_message)

	t.Logf("P2P_message generated: %s\n", p2p_message.To_String())

	if p2p_message.Sender != sender_field {
		t.Fatalf("Sender field mismatch!\n%s != %s\n", p2p_message.Sender, sender_field)
	}

	if string(p2p_message.Type) != string(type_field) {
		t.Fatalf("Type field mismatch!\n%s != %s\n", p2p_message.Type, type_field)
	}

	if p2p_message.Time.String() != time_field.String() {
		t.Fatalf("Time field mismatch!\n%s != %s\n", p2p_message.Time.String(), time_field.String())
	}

	if p2p_message.dependency.To_String() != dependency_field.To_String() {
		t.Fatalf("Dependency field mismatch!\n%s != %s\n", p2p_message.dependency.To_String(), dependency_field.To_String())
	}

	if p2p_message.Message != body_field {
		t.Fatalf("Body field mismatch!\n%s != %s\n", p2p_message.Message, body_field)
	}

	stringed_p2p_message := p2p_message.To_String()

	if stringed_p2p_message != test_tcp_message {
		t.Fatalf("Stringed message:\n%s\n---\nDid not match origin string %s\n", stringed_p2p_message, test_tcp_message)
	}
}

func Test_Message_Horizon(t *testing.T) {
	resolver := New_Dependency_Resolver()
	clock := New_Lamport_Clock()

	// 1 more than horizon
	for i := 0; i < Constants.P2P_MSG_TIME_HORIZON+1; i++ {
		clock.Event()

		p2p_message := New_P2P_Message("SENDER", MESSAGE, clock, "BODY")
		resolver.Emplace_New_Message(p2p_message)

		dependency := New_Dependency("SENDER", clock)

		resolved_p2p_message, ok := resolver.Get_Message(dependency)

		if !ok {
			t.Fatalf("Failed to get an ok on sending %s:\n", p2p_message.To_String())
		}

		if p2p_message.To_String() != resolved_p2p_message.To_String() {
			t.Fatalf("Returned string was ok, but not correct! %s != %s\n",
				p2p_message.To_String(), resolved_p2p_message.To_String())
		}
	}

	first_dependency := New_Dependency("SENDER", New_Lamport_Clock_From_String("1"))
	second_dependency := New_Dependency("SENDER", New_Lamport_Clock_From_String("2"))

	message, ok := resolver.Get_Message(first_dependency)

	if ok {
		t.Fatalf("Received a message that was supposed to be out of horizon!\n Resolver returned: %s\n",
			message.To_String())
	}

	message, ok = resolver.Get_Message(second_dependency)

	if !ok {
		t.Fatal("Did not find the second message that was supposed to be in horizon!")
	}

	p2p_message := New_P2P_Message("SENDER", MESSAGE, New_Lamport_Clock_From_String("2"), "BODY")
	if message.To_String() != p2p_message.To_String() {
		t.Fatalf("Returned second dependency was ok, but not correct! %s != %s\n",
			message.To_String(), p2p_message.To_String())
	}
}

func Test_Network(t *testing.T) {
	network_1 := New_P2P_Network()
	network_2 := New_P2P_Network()
	defer network_1.Close()
	defer network_2.Close()

	time.Sleep(time.Second)

	p2p_message := network_1.Create_Message("Hello!", MESSAGE)

	network_1.Broadcast(p2p_message)

	time.Sleep(time.Second)

	select {
	case received_message := <-network_2.Read_Channel:
		if p2p_message.To_String() != received_message.To_String() {
			t.Fatalf("Error Peer2Peer! Message received did not match sent!\n%s != %s\n",
				p2p_message.To_String(), received_message.To_String())
		}
	default:
		t.Fatal("Error Peer2Peer! Message was never received!")
	}
}

func Test_Dependency_Resend(t *testing.T) {
	network_1 := New_P2P_Network()
	network_2 := New_P2P_Network()
	defer network_1.Close()
	defer network_2.Close()

	time.Sleep(time.Second) // Let the networks connect to each other. Takes about 1.2x UDP SERVER LIFETIME (~300ms)

	p2p_message := network_1.Create_Message("Hello!", MESSAGE)

	network_1.Broadcast(p2p_message)

	network_3 := New_P2P_Network()
	defer network_3.Close()

	time.Sleep(time.Second)

	select {
	case received_message := <-network_2.Read_Channel:
		if p2p_message.To_String() != received_message.To_String() {
			t.Fatalf("Error Peer2Peer! Message received did not match sent!\n%s != %s\n",
				p2p_message.To_String(), received_message.To_String())
		}

		p2p_depended_message := network_2.Create_Message("Hello 2!", MESSAGE)
		p2p_depended_message.Depend_On(received_message)

		network_2.Broadcast(p2p_depended_message)

		time.Sleep(time.Second)

		select {
		case received_message_network_3 := <-network_3.Read_Channel:
			if p2p_message.To_String() != received_message_network_3.To_String() {
				t.Fatalf("Error Peer2Peer! Depended message received did not match sent!\n%s != %s\n",
					p2p_message.To_String(), received_message.To_String())
			}

			select {
			case received_depended_message := <-network_3.Read_Channel:
				if p2p_depended_message.To_String() != received_depended_message.To_String() {
					t.Fatalf("Error Peer2Peer! Depended message received did not match sent!\n%s != %s\n",
						p2p_message.To_String(), received_message.To_String())
				}
			default:
				t.Fatal("Error Peer2Peer Network 3! Hello 2 Message was never received!")
			}

		default:
			t.Fatal("Error Peer2Peer Network 3! Hello Message was never received!")
		}

	default:
		t.Fatal("Error Peer2Peer Network 2! Message was never received!")
	}
}

func Test_Double_Send(t *testing.T) {
	network_1 := New_P2P_Network()
	network_2 := New_P2P_Network()
	defer network_1.Close()
	defer network_2.Close()

	time.Sleep(time.Second)

	p2p_message := network_1.Create_Message("Hello!", MESSAGE)

	network_1.Broadcast(p2p_message)
	network_1.Broadcast(p2p_message)

	time.Sleep(time.Second)

	select {
	case received_message := <-network_2.Read_Channel:
		if p2p_message.To_String() != received_message.To_String() {
			t.Fatalf("Error Peer2Peer! Message received did not match sent!\n%s != %s\n",
				p2p_message.To_String(), received_message.To_String())
		}

		select {
		case spurious_message := <-network_2.Read_Channel:
			t.Fatalf("Error Peer2Peer, a message got sent twice! %s", spurious_message.To_String())
		default:
		}
	default:
		t.Fatal("Error Peer2Peer! Message was never received!")
	}
}
