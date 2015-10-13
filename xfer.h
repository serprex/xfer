#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <ctype.h>
#include <stdint.h>
#include <inttypes.h>
#include <string.h>
#include <unistd.h>
#include "linenoise/linenoise.h"

struct stack;
void vmexec(struct stack*st,const char*code);
void vmstart(const char*);
