.PHONY: all main debug clean

CPPFLAGS = -std=c++17 -Wno-unused-result

SRC = egraphin.cpp greedy.cpp json2egraphin.cpp main.cpp regionalize.cpp statewalkdp.cpp tiger.cpp toegglog.cpp debug.cpp 

main:
	g++ -O3 $(CPPFLAGS) $(SRC) -o main

debug:
	g++ -O0 $(CPPFLAGS) -DDEBUG $(SRC) -o main

clean: 
	rm -f main

all: main
