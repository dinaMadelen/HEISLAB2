package protocols

import "sync"

type Synchronization_Result struct {
	Voters        []string          // Who exists?
	Shared_States map[string]string // Shared_States[Elevator_ID] = Elevator_Shared_State, to be used by shared state server for synchro update.
}

type Synchronization_Vote struct {
	mu sync.Mutex

	result Synchronization_Result
}

func New_Synchronization_Vote() *Synchronization_Vote {
	return &Synchronization_Vote{
		result: Synchronization_Result{
			Voters:        make([]string, 0),
			Shared_States: make(map[string]string),
		},
	}
}

func (vote *Synchronization_Vote) Someone_Said_Hello(name string, state string) {
	vote.mu.Lock()
	defer vote.mu.Unlock()

	_, ok := vote.result.Shared_States[name]

	if ok {
		return
	}

	vote.result.Voters = append(vote.result.Voters, name)
	vote.result.Shared_States[name] = state
}

func (vote *Synchronization_Vote) Get_Result() Synchronization_Result {
	return vote.result
}
