package requests

import (
	"testing"

	m "group48.ttk4145.ntnu/elevators/models"
)

func TestRequestManager_OnePeerCycle(t *testing.T) {
	// Test that the request manager processes a cycle of messages from one peer

	// Setup
	var rm = newRequestManager()
	rm.alivePeers = []m.Id{1}
	var request = m.Request{Origin: m.Origin{Source: m.Hall{}, Floor: 1, ButtonType: m.HallUp}, Status: m.Unknown}
	var msg = m.RequestMessage{Source: 1, Request: request}
	var expected = request

	// With an unknown request, the request should be stored as is
	expected.Status = m.Unknown
	rm.process(msg)
	if rm.store[msg.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg.Request.Origin])
	}

	// Should change from unknown to absent
	msg.Request.Status = m.Absent
	expected.Status = m.Absent
	rm.process(msg)
	if rm.store[msg.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg.Request.Origin])
	}

	// As there is only one peer, the request should be confirmed immediately
	msg.Request.Status = m.Unconfirmed
	expected.Status = m.Confirmed
	res := rm.process(msg)
	if res != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg.Request.Origin])
	}

	// Should change from confirmed to absent
	msg.Request.Status = m.Absent
	expected.Status = m.Absent
	rm.process(msg)

	if rm.store[msg.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg.Request.Origin])
	}
}

func TestRequestManager_OnePeerFirstUnconfirmed(t *testing.T) {
	// Test that the request manager processes a unconfirmed request from one peer wihout a previous request correctly

	// Setup
	var rm = newRequestManager()
	rm.alivePeers = []m.Id{1}
	var request = m.Request{Origin: m.Origin{Source: m.Hall{}, Floor: 1, ButtonType: m.HallUp}, Status: m.Unknown}
	var msg = m.RequestMessage{Source: 1, Request: request}
	var expected = request

	// As there is only one peer, the request should be confirmed immediately
	msg.Request.Status = m.Unconfirmed
	expected.Status = m.Confirmed
	res := rm.process(msg)
	if res != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg.Request.Origin])
	}

	// Should change from confirmed to absent
	msg.Request.Status = m.Absent
	expected.Status = m.Absent
	rm.process(msg)

	if rm.store[msg.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg.Request.Origin])
	}
}

func TestRequestManager_TwoPeerCycle(t *testing.T) {
	// Test that the request manager processes a cycle of messages from two peers

	// Setup
	var rm = newRequestManager()
	rm.alivePeers = []m.Id{1, 2}
	var request = m.Request{Origin: m.Origin{Source: m.Hall{}, Floor: 1, ButtonType: m.HallUp}, Status: m.Unknown}
	var msg1 = m.RequestMessage{Source: 1, Request: request}
	var msg2 = m.RequestMessage{Source: 2, Request: request}
	var expected = request

	// With an unknown request, the request should be stored as is
	expected.Status = m.Unknown
	rm.process(msg1)
	rm.process(msg2)
	if rm.store[msg1.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg1.Request.Origin])
	}

	// When the first peer changes the request to unconfirmed, the request should stay unconfirmed
	msg1.Request.Status = m.Unconfirmed
	expected.Status = m.Unconfirmed
	rm.process(msg1)
	if rm.store[msg1.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg1.Request.Origin])
	}

	// When the second peer changes the request to unconfirmed, the request should be confirmed
	msg2.Request.Status = m.Unconfirmed
	expected.Status = m.Confirmed
	rm.process(msg2)
	if rm.store[msg1.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg1.Request.Origin])
	}

	// When the first peer changes the request to absent, the request should change to absent
	msg1.Request.Status = m.Absent
	expected.Status = m.Absent
	rm.process(msg1)
	if rm.store[msg1.Request.Origin] != expected {
		t.Errorf("Expected %v, got %v", expected, rm.store[msg1.Request.Origin])
	}
}
