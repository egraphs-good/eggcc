#ifndef JSON2EGRAPHIN_H
#define JSON2EGRAPHIN_H

// provides parsing eggcc's egglog output with basic pruning

#include "egraphin.h"

pair<EGraph, vector<EClassId> > parse_egglog_json();

#endif // JSON2EGRAPHIN_H