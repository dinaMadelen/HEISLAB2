package timer

import "time"

type Timer struct {
	endTime time.Time // Time when the timer will end
	active  bool      // Flag to indicate whether the timer is active
}

// NewTimer creates a new Timer instance.
func NewTimer() *Timer {
	return &Timer{active: false}
}

// Start sets the timer for a given duration (in seconds).
func (t *Timer) Start(duration float64) {
	t.endTime = time.Now().Add(time.Duration(duration * float64(time.Second)))
	t.active = true
}

// Stop deactivates the timer.
func (t *Timer) Stop() {
	t.active = false
}

// TimedOut checks if the timer has expired.
func (t *Timer) TimedOut() bool {
	return t.active && time.Now().After(t.endTime)
}
