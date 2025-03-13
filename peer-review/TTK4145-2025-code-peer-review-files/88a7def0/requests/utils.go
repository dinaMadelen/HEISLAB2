package requests

import m "group48.ttk4145.ntnu/elevators/models"

func isSetEqual(a, b []m.Id) bool {
	if len(a) != len(b) {
		return false
	}

	for _, id := range a {
		if !contains(b, id) {
			return false
		}
	}

	return true
}

func contains(s []m.Id, e m.Id) bool {
	for _, a := range s {
		if a == e {
			return true
		}
	}
	return false
}
