# Code Snapshot 
**Primary/Backup**-network: Primary distributes all orders, Backups waits for order assignments. 

Channels implemented in **main.go**
```
localOrderRequest := make(chan elevator.Order)
addToLocalQueue := make(chan elevator.Order)
assignOrder := make(chan elevator.OrderUpdate)
```

 will be used to synchronize the `elevator`-package and the `network`-package. 
 
 - A local button press is passed to `localOrderRequest`, and based on whether a node is 
   - **Primary**: The order will be assigned to the best distributed elevator (using `TimeToServeRequest()`),
   - or **Backup**: The order will be sent to **Primary**.
