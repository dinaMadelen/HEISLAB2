Project is written in Golang. As such, concurrent goroutines and channels are used.

Project should run as is, although a server is needed.
We are currently working on sharing variables. For an example of how we do this for maps, see primary.go:124

Some of what we are doing next:
- The functionality regarding order reassignment and obstructions is due to change. 
- We plan to move as much logic as possible out of the primary.Run main loop.
- We aim to reduce the number of channels in the main.go file drastically, we just haven't gotten around to doing that yet.
- Moving forward, we also aim to safeguard essential logic from packet loss.
    - One way to do this is by acknowledging orders implicitly by checking elevator updates for orders we (primary) has sent to each elevator slave. 
    - When the worldviews mismatch for reasons beyond disconnection or obstruction, the order will be resent.