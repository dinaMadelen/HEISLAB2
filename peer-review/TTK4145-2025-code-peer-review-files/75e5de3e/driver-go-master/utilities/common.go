package utilities

//Midlertidigløsning for å unngå loops

// Lagde en struct for hallcall køen for å øke lesbarhet
type HallCalls struct {
	//Men det er bare en helt vanlig 2D array
	Queue [2][4]int
}