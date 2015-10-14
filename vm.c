#include "xfer.h"
typedef intptr_t vint;
/*
xfer's executable format is a
forth dialect, nothing fancy,
only enough to implement coreutils,
rather than some lisp or something,
tho maybe I should've just chosen lua,
however I've been itching to forth
*/
struct obj{
	int16_t r;
	uint8_t t;
	uint8_t d[];
};
typedef struct obj obj;
struct stack{
	size_t s,z;
	union{
		obj*o;
		void**p;
		vint*i;
	};
};
typedef struct stack stack;
struct vmscratch{
	size_t faecount;
	char**faewords;
	size_t gccount;
	void**gc;
};
typedef struct vmscratch vmscratch;
static void*peek(stack*st,int n){
	return st->p+st->s-n;
}
static void*pop(stack*st){
	if(st->s) st->s--;
	return st->p+st->s;
}
static void pushfix(stack*st){
	if (st->s>st->z){
		st->z = st->s*2;
		st->p = realloc(st->p, st->z*sizeof(void*));
	}
}
static void*pushx(stack*st,size_t x){
	st->s+=x;
	pushfix(st);
	return st->p+st->s-1;
}
static void*push(stack*st){
	return pushx(st,1);
}
void pokei(stack*st,int n,vint x){
	*(vint*)peek(st,n) = x;
}
void pokev(stack*st,int n,void*x){
	*(void**)peek(st,n) = x;
}
vint peeki(stack*st,int n){
	return *(vint*)peek(st,n);
}
void*peekv(stack*st,int n){
	return *(void**)peek(st,n);
}
vint popi(stack*st){
	return *(vint*)pop(st);
}
void*popv(stack*st){
	return *(void**)pop(st);
}
void pushi(stack*st,vint i){
	*(vint*)push(st) = i;
}
void pushv(stack*st,void*p){
	*(void**)push(st) = p;
}
void pushptrsz(stack*st){
	pushi(st,sizeof(void*));
}
void pushdepth(stack*st){
	pushi(st,st->s);
}
void pushstack(stack*st){
	vint copy=peeki(st,1);
	stack*nst=malloc(sizeof(stack));
	nst->p=malloc(copy*sizeof(void*));
	memcpy(nst->p, st->p-copy, copy*sizeof(void*));
	pushv(st, nst);
}
void add(stack*st){
	vint*slot=peek(st,2);
	*slot+=popi(st);
}
void sub(stack*st){
	vint*slot=peek(st,2);
	*slot-=popi(st);
}
void mul(stack*st){
	vint*slot=peek(st,2);
	*slot*=popi(st);
}
void divmod(stack*st){
	vint a=peeki(st,2),b=peeki(st,1);
	if (b != 0){
		pokei(st,2,a/b);
		pokei(st,2,a%b);
	}
}
void sform(stack*st){
	if (st->s<2) return;
	const vint popx = peeki(st,1), newbase = peeki(st,2);
	if (popx>st->s || newbase>st->s) return;
	const size_t oidx = st->s-popx-2, bidx = st->s-newbase-2;
	vint i=0;
	do st->i[oidx+i]=st->i[oidx-st->i[oidx+i]]; while(++i<popx);
	memmove(st->i+bidx, st->i+oidx, popx*8);
	st->s=bidx+popx;
}
void pick(stack*st){
	if (peeki(st, 1)) pokei(st, 3, peeki(st, 2));
	st->s-=2;
}
void printint(stack*st){
	printf("%" PRId64,popi(st));
}
void printchr(stack*st){
	putchar(popi(st));
}
void printstr(stack*st){
	fputs(popv(st),stdout);
}
void getchr(stack*st){
	pushi(st, getchar());
}
const struct builtin{
	const char*op;
	void(*f)(struct stack*);
}builtin[]={
	{"+",add},
	{"-",sub},
	{"*",mul},
	{"%/",divmod},
	{"$",sform},
	{"?",pick},
	{"getc",getchr},
	{"print",printint},
	{"prchr",printchr},
	{"prstr",printstr},
	{"ptrsz",pushptrsz},
	{"depth",pushdepth},
};
const char*vmprelude = " : dup 1 1 1 $ : "
	": pop 1 0 $ : "
	": swap 1 2 2 2 $ : "
	": rsh3 1 3 2 3 3 $ : "
	": if ? . : "
	": iff [] rsh3 if : "
	": neg -1 * : "
	": prln prstr 10 prchr : ";
const char*isop(const char*restrict cop, const char*restrict bop){
	for(;;){
		if (*cop == ' ') return *bop?0:cop;
		else if (*cop != *bop) return 0;
		cop++;
		bop++;
	}
}
const char*trybuiltins(stack*st,const char*code){
	const char*r;
	for(int i=0; i<sizeof(builtin)/sizeof(*builtin); i++){
		if (r=isop(code, builtin[i].op)){
			builtin[i].f(st);
			return r;
		}
	}
	return 0;
}
const char*isfaeop(const char*restrict cop, const char*restrict bop){
	for(;;){
		if (*cop == ' ') return *bop != ' '?0:cop;
		else if (*cop != *bop) return 0;
		cop++;
		bop++;
	}
}
const char*tryfaewords(vmscratch*vs,stack*st,const char*code){
	const char*r;
	for(int i=0; i<vs->faecount; i++){
		if (r=isfaeop(code, vs->faewords[i]+1)){
			vmexec(vs, st, vs->faewords[i]+(r-code)+1);
			return r;
		}
	}
	return 0;
}
void*mkgc(vmscratch*vs,size_t x){
	vs->gc = realloc(vs->gc, sizeof(void*)*++vs->gccount);
	return vs->gc[vs->gccount-1] = malloc(x);
}
void freegc(vmscratch*vs,void*p){
	for(size_t i=0; i<vs->gccount; i++){
		if (vs->gc[i] == p){
			vs->gc[i] = 0;
			free(vs->gc[i]);
			return;
		}
	}
}
const char*defword(vmscratch*vs, const char*code){
	vs->faewords = realloc(vs->faewords, sizeof(char*)*++vs->faecount);
	const char*c2 = code;
	while(*c2 != ':' || c2[1] != ' ') c2++;
	vs->faewords[vs->faecount-1] = mkgc(vs,c2-code+1);
	memcpy(vs->faewords[vs->faecount-1], code-1, c2-code+2);
	vs->faewords[vs->faecount-1][c2-code+1]=0;
	return c2+2;
}
void vmfree(vmscratch*vs, stack*st){
	while(vs->gccount--) free(vs->gc[vs->gccount]);
	free(vs->gc);
	free(vs->faewords);
	free(st->p);
}
void vmexec(vmscratch*vs,stack*st,const char*code){
	const char*const selfcode = code;
	for (;;)
	{
		while(*code == ' ') code++;
		if (!*code) return;
		if (*code == ':' && code[1] == ' '){
			code = defword(vs, code+2);
			continue;
		}
		if (*code == '@' && code[1] == ' '){
			pushv(st, (char*)selfcode);
			code+=2;
			continue;
		}
		if (*code == '#' && code[1] == ' '){
			pushv(st, (char*)(code+1));
			code+=2;
			continue;
		}
		if (*code == '.' && code[1] == ' '){
			vmexec(vs, st, popv(st));
			code+=2;
			continue;
		}
		if (*code == '['){
			const char*const start = code+1;
			int pm = 1;
			while(*++code){
				if (*code == '[') pm++;
				else if (*code == ']' && !--pm) break;
			}
			char*p=mkgc(vs, code-start+1);
			memcpy(p, start, code-start);
			p[code-start] = 0;
			pushv(st,p);
			code++;
			continue;
		}
		if ((*code == '-' && isdigit(code[1])) || isdigit(*code)){
			int neg = *code == '-';
			while(*++code!=' ');
			vint x=0,n=1;
			const char*c2=code;
			while(*--c2!=(neg?'-':' ')){
				x+=(*c2&15)*n;
				n*=10;
			}
			pushi(st,neg?-x:x);
			continue;
		}
		const char*r=trybuiltins(st,code);
		if (!r) r=tryfaewords(vs,st,code);
		if (r){
			code=r;
			continue;
		}
		code++;
	}
}
void vmstart(const char*code){
	int codelen = strlen(code), prefix = code[0] != ' ', postfix = code[codelen-1] != ' ';
	if (prefix || postfix){
		char*newcode = malloc(codelen+prefix+postfix+1);
		if (prefix) newcode[0] = ' ';
		memcpy(newcode+prefix, code, codelen);
		if (postfix) newcode[codelen+postfix+prefix-1] = ' ';
		newcode[codelen+prefix+postfix] = 0;
		code=newcode;
	}
	struct stack st = {};
	struct vmscratch vs = {};
	vmexec(&vs, &st, vmprelude);
	vmexec(&vs, &st, code);
	vmfree(&vs, &st);
	if (prefix || postfix){
		free((void*)code);
	}
}
