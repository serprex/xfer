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
struct vmscratch;
extern const char*vmprelude;
void vmexec(struct vmscratch*,struct stack*,const char*);
void vmstart(const char*);
