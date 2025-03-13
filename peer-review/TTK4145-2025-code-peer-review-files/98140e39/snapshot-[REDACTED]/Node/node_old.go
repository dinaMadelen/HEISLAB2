package node

import "elevatorproject/driver-go/elevio"

type NodeAttributes struct {
	BtnEventTx chan elevio.ButtonEvent
	BtnEventRx chan elevio.ButtonEvent
}

type NodeState int

const (
	disconnected NodeState	= iota
	slave
	master
	inactive
)

func NodeFsm(nodeAttributes NodeAttributes) {
	state := disconnected

	for {
		switch state {
		case disconnected:
			state = nodeDisconnected(nodeAttributes)
		case slave:
			state = nodeSlave(nodeAttributes)
		case master:
			state = nodeMaster(nodeAttributes)
		case inactive:
			state = nodeInactive(nodeAttributes)
		}
	}

	// her skal det skje masse greier
	// lag en liste over hva som skal skje her
}

func nodeDisconnected(nodeAttributes NodeAttributes) NodeState {
	return inactive
}

func nodeSlave(nodeAttributes NodeAttributes) NodeState {
	return inactive
}

func nodeMaster(nodeAttributes NodeAttributes) NodeState {
	return inactive
}

func nodeInactive(nodeAttributes NodeAttributes) NodeState {
	return inactive
}


