package utilities

import (
	"fmt"
	"net"
	"time"
)

// Sender en melding over UDP til en spesifisert adresse
func UtilitiesSendMessage(msg string, SendAddr string) {

	//Løser opp adressen for UDP-kommunikasjon
	sendUDPAddr, err := net.ResolveUDPAddr("udp", SendAddr)
	if err != nil {
		fmt.Println(err)
		return
	}

	//Oppretter en UDP-tilkobling til den spesifiserte adressen
	conn, err := net.DialUDP("udp", nil, sendUDPAddr)
	if err != nil {
		fmt.Println(err)
		return
	}
	defer conn.Close()

	//Sender melding som en byte-strøm over UDP
	_, err2 := conn.Write([]byte(msg))
	if err2 != nil {
		fmt.Println("Failed", err)
		return
	}
	fmt.Println("sent message")

}

// Sender et signal til en kanal hvert 100 millisekund
// Brukes for å indikere at en prosess er aktiv/ "heartbeat"-mekansisme
func UtilitiesSendHeartbeat(ch chan<- bool) {
	for {
		ch <- true
		time.Sleep(100 * time.Millisecond)
	}
}
