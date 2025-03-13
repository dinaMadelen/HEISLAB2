package shared_state

// Implements the shared_state between all nodes.
// Provides communication with the elevator:
// Commands TO elevator
//		Turn on lights
//		Go to floor
//
// Messages FROM elevator
//		Button pressed
//		Floor reached
//

// Internal functions
//		Initialize
//


/*

struct Shared_State
 - Contains the shared state
 - Contains alive nodes
 
 - Accepts updates & synchronization result from Network via channel made by main
 - Informs HRA about new shared state	   (State information about alive nodes)
 - Informs elevator about new shared state (lights)

 - Defers updates from elevator to network (which will start an update protocol sequence which should end in an update to shared state.)
*/
