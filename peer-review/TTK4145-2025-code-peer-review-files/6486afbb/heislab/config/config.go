package config

const (
	N_FLOORS    = 4
	N_BUTTONS   = 3
	N_ELEVATORS = 3
)

// network
const (
	WatchdogPort               = 15500
	MasterUpdatePort           = 30019
	BackupsUpdatePort          = 30029
	MasterCallsPort            = 30039
	BackupsCallsPort           = 30049
	SlaveBasePort              = 40000
	BackupMessagePeriodSeconds = 1
	MasterMessagePeriodSeconds = 1
	ResendPeriodMs             = 500
	ResendTimeoutMs            = 5000
)
