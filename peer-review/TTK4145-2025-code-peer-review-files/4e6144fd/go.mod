module github.com/mathiahb/TTK4145_V2025_Gruppe4

require tests v0.0.0

require (
	Constants v0.0.0
	golang.org/x/net v0.35.0 // indirect
)

replace tests => ./tests/

replace Constants => ./Constants/

replace Network-Protocol => ./shared_state/Network_Protocol/

replace Driver-Elevator => ./elevator/

replace Driver-Elevio => ./elevator/elevio/

go 1.16
