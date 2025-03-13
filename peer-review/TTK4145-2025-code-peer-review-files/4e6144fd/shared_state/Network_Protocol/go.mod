module Network-Protocol

require (
	Constants v0.0.0
	golang.org/x/net v0.35.0
)

require golang.org/x/sys v0.30.0 // indirect

replace Constants => ./../../Constants/

go 1.18
