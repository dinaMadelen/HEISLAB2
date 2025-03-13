package connectivity

import "flag"

const (
	NR_OF_ELEVATORS = 3
	// Timeout for receiving UDP messages
	TIMEOUT = 3

	// Worldview max package size
	PACKAGE_SIZE = 1500
)

var (
	ID int //deafult 0
)

func init() {
	flag.IntVar(&ID, "id", 0, "Specify the id with -id")
	flag.Parse()
}
