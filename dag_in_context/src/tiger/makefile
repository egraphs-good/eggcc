.PHONY: all main clean json2egraph egraph2json

CPPFLAGS = -O2 -Wno-unused-result

json2egraph:
	g++ $(CPPFLAGS) json2egraph.cpp -o json2egraph

main:
	g++ $(CPPFLAGS) main.cpp ilp.cpp -o main

egraph2json:
	g++ $(CPPFLAGS) egraph2json.cpp -o egraph2json

clean: 
	rm -f json2egraph main

all: json2egraph main
