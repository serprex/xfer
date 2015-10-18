#include <stdint.h>
#include <stddef.h>
typedef intptr_t vint;
typedef struct obj obj;
typedef struct stack stack;
typedef struct vmscratch vmscratch;
void*peeki(stack*,int);
void*peekv(stack*,int);
vint popi(stack*);
void*popv(stack*);
void pushi(stack*,vint);
void pushv(stack*,void*);
void pokei(stack*,int,vint);
void pokev(stack*,int,void*);
void*mkgc(vmscratch*,size_t);
void*freegc(vmscratch*,void*);
extern const char*vmprelude;
void vmexec(struct vmscratch*,struct stack*,const char*);
void vmstart(const char*);
