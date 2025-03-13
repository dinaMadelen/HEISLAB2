
EXECUTABLE = ttk4145demoelevator

SOURCES := $(shell find . -name '*.go' ! -name '*_test.go')

all: $(EXECUTABLE)

rebuild: clean all

clean:
	rm -f $(EXECUTABLE)

$(EXECUTABLE): $(SOURCES)
	go build -o $(EXECUTABLE) $(SOURCES)

.PHONY: all rebuild clean
