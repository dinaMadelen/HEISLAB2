package peer_to_peer

import (
	"Constants"
	"fmt"
	"strings"
)

type Dependency struct {
	Dependency_Owner string
	Dependency_Clock Lamport_Clock
}

func New_Dependency(owner string, clock Lamport_Clock) Dependency {
	return Dependency{
		Dependency_Owner: owner,
		Dependency_Clock: clock,
	}
}

// Format Dependency:
// DEPENDENCYOWNER/LAMPORTCLOCK
func (dependency Dependency) To_String() string {
	return dependency.Dependency_Owner + Constants.P2P_DEP_DELIMINATOR + dependency.Dependency_Clock.String()
}

func Dependency_From_String(str string) Dependency {
	fields := strings.Split(str, Constants.P2P_DEP_DELIMINATOR)

	if len(fields) != 2 {
		fmt.Printf("Error parsing dependency! %s\n", str)
		return Dependency{}
	}

	owner := fields[0]
	time := New_Lamport_Clock_From_String(fields[1])

	return Dependency{
		Dependency_Owner: owner,
		Dependency_Clock: time,
	}
}

func (lesser Dependency) Is_Less_Than(greater Dependency) bool {
	return lesser.Dependency_Clock.Is_Less_Than(greater.Dependency_Clock)
}
