package networkOrders

import (
	"time"
	"sanntids/cmd/localElevator/structs"
	"fmt"
)

// NetworkOrderManager handles the conversion of local orders to network-ready format
// and manages incoming orders from other elevators

// Need to implement changing orders
// Need to implement removing orders
func NetworkOrderManager(
    localElevatorID string,
    localElevStateChan <-chan structs.HRAElevState,
    localOrdersChan <-chan structs.HallOrder,
    incomingDataChan <-chan structs.ElevatorDataWithID,
    outgoingDataChan chan<- structs.ElevatorDataWithID,
) {
    // Initialize data stores
    elevatorStates := make(map[string]structs.HRAElevState)
    hallOrders := make([]structs.HallOrder, 0)
    
    // Throttle broadcasts to prevent network flooding
    var lastBroadcast time.Time
    broadcastThreshold := 300 * time.Millisecond
    
    // Status ticker for debugging
    statusTicker := time.NewTicker(5 * time.Second)
    defer statusTicker.Stop()
    
    // Single goroutine to handle all channels
    go func() {
        for {
            select {
            case <-statusTicker.C:
            
            case incomingData, ok := <-incomingDataChan:
                if !ok {
                }
                
                // Skip processing our own messages
                if incomingData.ElevatorID == localElevatorID {
                    continue
                }
                
                // Update state map with received data
                for id, state := range incomingData.ElevatorState {
                    elevatorStates[id] = state
                }
                
                // Process incoming hall orders
                ordersAdded := 0
                for _, newOrder := range incomingData.HallOrders {
                    if !isDuplicateOrder(hallOrders, newOrder) {
                        hallOrders = append(hallOrders, newOrder)
                        ordersAdded++
                    }
                }
                
                // Only broadcast if we actually added new orders
                if ordersAdded > 0 {
                    sendNetworkData(localElevatorID, elevatorStates, hallOrders, outgoingDataChan)
                }
                
            case localState, ok := <-localElevStateChan:
                if !ok {
                    return
                }
                
                    
                elevatorStates[localElevatorID] = localState
                
                // Throttle broadcasts to prevent network flooding
                if time.Since(lastBroadcast) > broadcastThreshold {
                    sendNetworkData(localElevatorID, elevatorStates, hallOrders, outgoingDataChan)
                    lastBroadcast = time.Now()
                }
                
            case localOrder, ok := <-localOrdersChan:
                if !ok {
                    return
                }
                
                localOrder.DelegatedID = localElevatorID
                orderAdded := false
                if !isDuplicateOrder(hallOrders, localOrder) {
                    hallOrders = append(hallOrders, localOrder)
                    orderAdded = true
                } else {
                    // Update the existing order status if it already exists
                    for i := range hallOrders {
                        if hallOrders[i].Floor == localOrder.Floor && hallOrders[i].Dir == localOrder.Dir {
                            hallOrders[i].Status = localOrder.Status
                            hallOrders[i].DelegatedID = localOrder.DelegatedID
                            orderAdded = true
                            break
                        }
                    }
                }
                
                // Always broadcast when we get a local order, regardless of throttling
                if orderAdded {
                    sendNetworkData(localElevatorID, elevatorStates, hallOrders, outgoingDataChan)
                    lastBroadcast = time.Now()
                }
            }
        }
    }()
}

// Two orders are considered duplicates if they have the same floor and direction
func isDuplicateOrder(orders []structs.HallOrder, newOrder structs.HallOrder) bool {
    for _, order := range orders {
        if order.Floor == newOrder.Floor && order.Dir == newOrder.Dir {
			fmt.Printf("duplicate")
            return true
        }
    }
    return false
}

func sendNetworkData(
    localID string,
    states map[string]structs.HRAElevState,
    orders []structs.HallOrder,
    outChan chan<- structs.ElevatorDataWithID,
) {
    statesCopy := make(map[string]structs.HRAElevState)
    for id, state := range states {
        statesCopy[id] = state
    }
    
    ordersCopy := make([]structs.HallOrder, len(orders))
    copy(ordersCopy, orders)
    
    networkData := structs.ElevatorDataWithID{
        ElevatorID:    localID,
        ElevatorState: statesCopy,
        HallOrders:    ordersCopy,
    }
    
    select {
    case outChan <- networkData:
    default:
    }
}

// UpdateOrderStatus updates the status of a hall order in the order list
func UpdateOrderStatus(orders []structs.HallOrder, floor int, dir int, newStatus structs.OrderStatus) []structs.HallOrder {
    for i, order := range orders {
        if order.Floor == floor && int(order.Dir) == dir {
            orders[i].Status = newStatus
            break
        }
    }
    return orders
}

// RemoveCompletedOrders removes orders with Completed status from the order list
func RemoveCompletedOrders(orders []structs.HallOrder) []structs.HallOrder {
    result := make([]structs.HallOrder, 0)
    removed := 0
    
    for _, order := range orders {
        if order.Status != structs.Completed {
            result = append(result, order)
        } else {
            removed++
        }
    }
    
    if removed > 0 {
    }
    
    return result
}