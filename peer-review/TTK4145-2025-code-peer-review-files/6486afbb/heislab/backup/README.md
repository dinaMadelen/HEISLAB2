Backup module
===================

Contains a single go routine responsible for storing information from the master and broadcasting it back, in addition to transitioning to master state if it loses connection with the current master. 