// Compile with `gcc foo.c -Wall -std=gnu99 -lpthread`, or use the makefile
// The executable will be named `foo` if you use the makefile, or `a.out` if you use gcc directly

#include <pthread.h>
#include <stdio.h>
#include <stdint.h>

pthread_mutex_t mutex;
int i = 0;

// Note the return type: void*
void* incrementingThreadFunction(){
    // TODO: increment i 1_000_000 times
    for (int32_t k = 0; k<1000000; k++ ){
        pthread_mutex_lock(&mutex);
        i++;
        pthread_mutex_unlock(&mutex);
    }
    return NULL;
}

void* decrementingThreadFunction(){
    // TODO: decrement i 1_000_000 times
        for (int32_t k = 0; k<1000000; k++ ){
        pthread_mutex_lock(&mutex);
        i--;
        pthread_mutex_unlock(&mutex);
    }
    return NULL;
}


int main(){

    pthread_mutex_init(&mutex, NULL);

    pthread_t incrementingThread, decrementingThread;
    // TODO: 
    // start the two functions as their own threads using `pthread_create`
    // Hint: search the web! Maybe try "pthread_create example"?


    pthread_create(&incrementingThread,NULL,incrementingThreadFunction,NULL);
    pthread_create(&decrementingThread,NULL,decrementingThreadFunction,NULL);
    
    
    // TODO:
    // wait for the two threads to be done before printing the final result
    // Hint: Use `pthread_join`    

    pthread_join(incrementingThread,NULL);
    pthread_join(decrementingThread,NULL);

    pthread_mutex_destroy(&mutex);

    printf("The magic number is: %d\n", i);
    return 0;
}
