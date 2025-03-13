//go:build testmode

package main

import (
	"context"
	"fmt"
	fsm "realtime_systems/state_machine"
	"time"
)

// Define our event types
type StartEvent struct{}
type DocumentEvent struct {
	Content string
}
type ErrorEvent struct {
	Err error
}
type StopEvent struct{}

// Global instance to access from state functions
var stateMachine *fsm.FSM

func main() {
	// Create our state machine with the initial state
	stateMachine = fsm.NewFSM(idleState)

	// Start the state machine
	ctx, cancel := context.WithCancel(context.Background())
	stateMachine.Start(ctx)
	defer cancel()

	// Create a channel for receiving processed documents
	results := make(chan string)
	stateMachine.SetData("results", results)

	// Start a goroutine to handle processed documents
	go func() {
		for result := range results {
			fmt.Println("Processed document:", result)
		}
	}()

	// Send events to the state machine
	stateMachine.Send(StartEvent{})

	// Simulate receiving documents from another source
	go func() {
		docs := []string{
			"Document 1: Important content",
			"Document 2: Critical information",
			"Document 3: Confidential data",
		}

		for _, content := range docs {
			time.Sleep(100 * time.Millisecond)
			stateMachine.Send(DocumentEvent{Content: content})
		}

		time.Sleep(1 * time.Second)
		stateMachine.Send(StopEvent{})
	}()

	// Wait for a bit to let everything process
	time.Sleep(5 * time.Second)
}

// State functions - now they return fsm.StateFunc, not StateFunc

// idleState handles events when the system is idle
func idleState(event interface{}) fsm.StateFunc {
	switch e := event.(type) {
	case StartEvent:
		fmt.Println("Starting document processing...")
		return processingState
	default:
		fmt.Printf("Idle state: ignoring event %T\n", e)
	}
	return nil // Stay in current state
}

// processingState handles events during document processing
func processingState(event interface{}) fsm.StateFunc {
	switch e := event.(type) {
	case DocumentEvent:
		fmt.Printf("Processing document: %s\n", e.Content)

		// Simulate processing and send to results channel
		go func(content string) {
			// Simulate processing time
			time.Sleep(2000 * time.Millisecond)

			// Get the results channel from FSM data
			if resultsChannel, ok := stateMachine.GetData("results").(chan string); ok {
				resultsChannel <- "PROCESSED: " + content
			}
		}(e.Content)

		return nil // Stay in processing state

	case ErrorEvent:
		fmt.Printf("Error occurred: %v\n", e.Err)
		return errorState

	case StopEvent:
		fmt.Println("Stopping document processing...")
		return finishedState
	}

	return nil // Stay in current state
}

// errorState handles events when an error has occurred
func errorState(event interface{}) fsm.StateFunc {
	switch e := event.(type) {
	case StartEvent:
		fmt.Println("Restarting after error...")
		return processingState

	case StopEvent:
		fmt.Println("Stopping after error...")
		return finishedState

	default:
		fmt.Printf("Error state: ignoring event %T\n", e)
	}

	return nil // Stay in current state
}

// finishedState is the terminal state
func finishedState(event interface{}) fsm.StateFunc {
	fmt.Printf("Finished state: ignoring event %T\n", event)
	return nil // Stay in finished state
}
