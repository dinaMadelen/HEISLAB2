package order_handler

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os/exec"
)


// this function runs the executable hall_request_assigner. It's intended input is created and explained in build_assinger_input
func AssignHallRequests(input map[string]interface{}) (map[string]interface{}, error) {
	
	// to JSON format
	inputJSON, err := json.Marshal(input)
	if err != nil {
		return nil, fmt.Errorf("error marshalling input JSON: %v", err)
	}

	fmt.Println("Sending to hall_request_assigner:", string(inputJSON))

	// cmd is our executable, where the orders are truly assigned
	cmd := exec.Command("./hall_request_assigner", "-i", string(inputJSON))

	var outputBuffer bytes.Buffer
	cmd.Stdout = &outputBuffer
	cmd.Stderr = &outputBuffer

	fmt.Println("Running hall_request_assigner...")

	err = cmd.Run()
	if err != nil {
		fmt.Println("Error output:", outputBuffer.String()) 
		return nil, fmt.Errorf("error running hall_request_assigner: %v", err)
	}

	fmt.Println("Raw output from hall_request_assigner:", outputBuffer.String())


	var output map[string]interface{}
	// back to Go from JSON format
	err = json.Unmarshal(outputBuffer.Bytes(), &output)
	if err != nil {
		return nil, fmt.Errorf("error parsing output JSON: %v", err)
	}

	return output, nil
}