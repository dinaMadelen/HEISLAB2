

package main

type DirnBehaviourPair struct {
    dirn       MotorDirection
    state      ElevState
}

func requestsAbove(e Elevator) bool {
    for f := e.floor + 1; f < N_FLOORS; f++ {
        for btn := 0; btn < N_BUTTONS; btn++ {
            if e.requests[f][btn] {
                return true
            }
        }
    }
    return false
}

func requestsBelow(e Elevator) bool {
    for f := 0; f < e.floor; f++ {
        for btn := 0; btn < N_BUTTONS; btn++ {
            if e.requests[f][btn] {
                return true
            }
        }
    }
    return false
}

func requestsHere(e Elevator) bool {
    for btn := 0; btn < N_BUTTONS; btn++ {
        if e.requests[e.floor][btn] {
            return true
        }
    }
    return false
}

func requestsChooseDirection(e Elevator) DirnBehaviourPair {
    switch e.dirn {
    case DIRN_UP:
        if requestsAbove(e) {
            return DirnBehaviourPair{DIRN_UP, ELEVSTATE_MOVING}
        } else if requestsHere(e) {
            return DirnBehaviourPair{DIRN_DOWN, ELEVSTATE_DOOR_OPEN}
        } else if requestsBelow(e) {
            return DirnBehaviourPair{DIRN_DOWN, ELEVSTATE_MOVING}
        }
    case DIRN_DOWN:
        if requestsBelow(e) {
            return DirnBehaviourPair{DIRN_DOWN, ELEVSTATE_MOVING}
        } else if requestsHere(e) {
            return DirnBehaviourPair{DIRN_UP, ELEVSTATE_DOOR_OPEN}
        } else if requestsAbove(e) {
            return DirnBehaviourPair{DIRN_UP, ELEVSTATE_MOVING}
        }
    case DIRN_STOP:
        if requestsHere(e) {
            return DirnBehaviourPair{DIRN_STOP, ELEVSTATE_DOOR_OPEN}
        } else if requestsAbove(e) {
            return DirnBehaviourPair{DIRN_UP, ELEVSTATE_MOVING}
        } else if requestsBelow(e) {
            return DirnBehaviourPair{DIRN_DOWN, ELEVSTATE_MOVING}
        }
    }
    return DirnBehaviourPair{DIRN_STOP, ELEVSTATE_IDLE}
}

func requestsShouldStop(e Elevator) bool {
    switch e.dirn {
    case DIRN_DOWN:
        return e.requests[e.floor][BUTTON_HALL_DOWN] || e.requests[e.floor][BUTTON_CAB] || !requestsBelow(e)
    case DIRN_UP:
        return e.requests[e.floor][BUTTON_HALL_UP] || e.requests[e.floor][BUTTON_CAB] || !requestsAbove(e)
    case DIRN_STOP:
        return true
    }
    return false
}

func requestsShouldClearImmediately(e Elevator, btnfloor int, btnType ButtonType) bool {
    switch e.config.clearRequests {
    case CLEAR_ALL:
        return e.floor == btnfloor
    case CLEAR_DIRECTION:
        return e.floor == btnfloor && (e.dirn == DIRN_UP && btnType == BUTTON_HALL_UP || e.dirn == DIRN_DOWN && btnType == BUTTON_HALL_DOWN || e.dirn == DIRN_STOP || btnType == BUTTON_CAB)
    }
    return false
}

func requestsClearAtCurrentFloor(e Elevator) Elevator {
    switch e.config.clearRequests {
    case CLEAR_ALL:
        for btn := 0; btn < N_BUTTONS; btn++ {
            e.requests[e.floor][btn] = false
        }
    case CLEAR_DIRECTION:
        e.requests[e.floor][BUTTON_CAB] = false
        switch e.dirn {
        case DIRN_UP:
            if !requestsAbove(e) && !e.requests[e.floor][BUTTON_HALL_UP] {
                e.requests[e.floor][BUTTON_HALL_DOWN] = false
            }
            e.requests[e.floor][BUTTON_HALL_UP] = false
        case DIRN_DOWN:
            if !requestsBelow(e) && !e.requests[e.floor][BUTTON_HALL_DOWN] {
                e.requests[e.floor][BUTTON_HALL_UP] = false
            }
            e.requests[e.floor][BUTTON_HALL_DOWN] = false
        default:
            e.requests[e.floor][BUTTON_HALL_UP] = false
            e.requests[e.floor][BUTTON_HALL_DOWN] = false
        }
    }
    return e
}