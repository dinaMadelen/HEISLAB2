module sanntids

go 1.16

require (
	Driver-go v0.0.0
)

replace Driver-go => ./lib/driver-go

require Network-go v0.0.0
replace Network-go => ./lib/network-go
