//go:build testmode

package main

import (
	"context"
	"os"
	"os/signal"

	"realtime_systems/elevator"
	"realtime_systems/elevio"
)

func main() {
	// Initialize elevator hardware
	elevio.Init("localhost:15657", 4)

	// Create context that can be cancelled
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Create and start elevator
	elev := elevator.New()
	go elev.Run(ctx)

	// Wait for CTRL+C
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt)
	<-c

	// Cleanup
	cancel()
}
