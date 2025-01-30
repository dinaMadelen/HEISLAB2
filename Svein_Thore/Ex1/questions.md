Exercise 1 - Theory questions
-----------------------------

### Concepts

What is the difference between *concurrency* and *parallelism*?
> Concurrency is running multiple threads by swapping between the threads while paralellism is running multiple threads at the same time on multiple cores

What is the difference between a *race condition* and a *data race*? 
> A race condition is when the behavior is dependant on which thread finishes first.
a data race is when one or more threads wants to write to the same variable at the same time as another thread interacts with it, which might cause the variable to be in the middle of being changed when it is read or written to.  
 
*Very* roughly - what does a *scheduler* do, and how does it do it?
>The Scheduler decides the order in which operations for each thread is done. it allocates processing time for each thread and swaps between running each thread.


### Engineering

Why would we use multiple threads? What kinds of problems do threads solve?
> If we have tasks that require us to do multiple things at the same time we can use threads to split the taskes into multiple threads and swap between the treads. threads solve the problem of wasting time on waiting for processes to finish. 

Some languages support "fibers" (sometimes called "green threads") or "coroutines"? What are they, and why would we rather use them over threads?
> They are threads within threads that unlike normal threads are not planned by the scheduler but manually started and stopped by the programmer. 

Does creating concurrent programs make the programmer's life easier? Harder? Maybe both?
> It depends on the program, not all programs have to be concurent. Threads are a tool to solve specific types of problems. but using it might also cause issues with deadlock or raceconditions but might also make the program simpler to write than a program that does not use threads. 

What do you think is best - *shared variables* or *message passing*?
> They both have their use cases but i think Message passing is better overall as it is more scaleable for larger projects and keeps memory separate. but i would imagine shared variables to be faster and simpler(for smaller projects).


