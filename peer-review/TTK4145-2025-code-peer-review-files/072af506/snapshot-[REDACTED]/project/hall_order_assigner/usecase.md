In this module we go under the assumtion that we have the variable "statuses":

var statuses // where this variable is a slice filled with statusmessages for each healthy elevator 

In build_assigner_input.go we send the variable statuses into the function:
BuildAssignerInputFromStatusMessages(statuses)
and this function returns an input that the AssignHallRequests function in order_assigner accepts 

AssignHallRequests(BuildAssignerInputFromStatusMessages(statuses)) does not return the orders the way we want them,
and therefore we use order_distrubtion_message to convert this output to the way want it. That is on the form:

type OrderDistributionMessage struct {
	Label string
	Orderlines [3][elevator.N_FLOORS][elevator.N_BUTTONS - 1]bool
}

This is done as follows: 

order_msg := OrderDistributionMessage(AssignHallRequests(BuildAssignerInputFromStatusMessages(statuses)))
