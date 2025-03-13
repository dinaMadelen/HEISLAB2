package localip

import (
	"fmt"
	"net"
	"os"
	"strings"
)

var localIP string

func LocalIP() (string, error) {
	if localIP == "" {
		conn, err := net.DialTCP("tcp4", nil, &net.TCPAddr{IP: []byte{8, 8, 8, 8}, Port: 53})
		if err != nil {
			return "", err
		}
		defer conn.Close()
		localIP = strings.Split(conn.LocalAddr().String(), ":")[0]
	}
	return localIP, nil
}

func MyId() (string, error) {
	localIP, err := LocalIP()
	var MyId string
	if err != nil {
		MyId = ""
	} else {
		MyId = fmt.Sprintf("%s:%d", localIP, os.Getpid())
	}
	return MyId, err
}

func IdToIp(id string) (string, error) {
	parts := strings.Split(id, ":")
	if len(parts) != 2 {
		return "", fmt.Errorf("invalid id format")
	}
	return parts[0], nil
}