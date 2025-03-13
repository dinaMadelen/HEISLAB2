package fsm

import (
	"context"
)

// StateFunc represents a state in our state machine
// It takes an event and returns the next state function
type StateFunc func(event interface{}) StateFunc

// FSM represents our function-based state machine
type FSM struct {
	// Channel for sending events to the state machine
	events chan interface{}
	// Channel for graceful shutdown
	done chan struct{}
	// Current state function
	currentState StateFunc
	// Optional context data for the state machine
	data map[string]interface{}
}

// NewFSM creates a new state machine starting in the given initial state
func NewFSM(initialState StateFunc) *FSM {
	return &FSM{
		events:       make(chan interface{}),
		done:         make(chan struct{}),
		currentState: initialState,
		data:         make(map[string]interface{}),
	}
}

// Start begins processing events in a separate goroutine
func (f *FSM) Start(ctx context.Context) {
	go func() {
		for {
			select {
			case event := <-f.events:
				// Process the event with the current state function
				nextState := f.currentState(event)
				if nextState != nil {
					// Transition to the next state if one was returned
					f.currentState = nextState
				}
			case <-ctx.Done():
				// Handle context cancellation
				close(f.done)
				return
			}
		}
	}()
}

// Send sends an event to the state machine
func (f *FSM) Send(event interface{}) {
	f.events <- event
}

// GetData retrieves data from the state machine context
func (f *FSM) GetData(key string) interface{} {
	return f.data[key]
}

// SetData stores data in the state machine context
func (f *FSM) SetData(key string, value interface{}) {
	f.data[key] = value
}

// Stop gracefully stops the state machine
func (f *FSM) Stop() {
	close(f.events)
	<-f.done
}
