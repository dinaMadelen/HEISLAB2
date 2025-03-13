package main

import (
	backupprocess "Driver-go/backupProcess"
	"Driver-go/elevio"
)

const (
	heisID = 0
	flag  = "localhost:15657" //Adresse til heis
)

func main() {

	numFloors := 4 
	elevio.Init(flag, numFloors)

	if heisID == 0 { 
		backupprocess.BackupProcess() //Starter en ny heis som backup. Skal konfigureres slik at det er mulig å init en slave også
	}

}
