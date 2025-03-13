package config

const N_FLOORS = 4
const N_BUTTONS = 3
const DoorOpenDuration_s = 3.0

type ClearRequestVariant int
const (
    CV_All ClearRequestVariant = iota
    CV_InDirn
)



