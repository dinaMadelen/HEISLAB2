package main

import (
	"Project/assigner"
	. "Project/dataenums"
	"Project/driver"
	"Project/hwelevio"
	"Project/lights"
	"Project/network"
	"flag"
	"strconv"
)

func main() {

	nodeID := parseArgs()

	hwelevio.Init(Addr)

	var (
		newOrderChannel       = make(chan [NFloors][NButtons]bool, 100)
		fromDriverToAssigner  = make(chan FromDriverToAssigner, 100)
		fromAssignerToNetwork = make(chan FromAssignerToNetwork, 100)
		fromNetworkToAssigner = make(chan FromNetworkToAssigner, 100)
		fromDriverToLight     = make(chan FromDriverToLight, 100)
		fromAssignertoLight   = make(chan [NFloors][NButtons]ButtonState, 100)
	)

	go assigner.Assigner(
		newOrderChannel,
		fromDriverToAssigner,
		fromAssignerToNetwork,
		fromNetworkToAssigner,
		fromAssignertoLight,
		nodeID,
	)

	go driver.ElevatorDriver(
		newOrderChannel,
		fromDriverToAssigner,
		fromDriverToLight,
	)

	go network.Network(
		fromAssignerToNetwork,
		fromNetworkToAssigner,
		nodeID,
	)

	go lights.LightsHandler(
		fromAssignertoLight,
		fromDriverToLight,
	)
	// TODO is the select needed ?
	select {}
}

func parseArgs() string {
	var nodeID int
	flag.IntVar(&nodeID, "id", 0, "Node ID")
	flag.Parse()
	return strconv.Itoa(nodeID)
}
