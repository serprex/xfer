#include "xfer.h"
/*
xfer's executable format is a
forth dialect, nothing fancy,
only enough to implement coreutils,
rather than some lisp or something,
tho maybe I should've just chosen lua,
however I've been itching to forth
*/
struct stack{
	size_t s,z;
	union{
		void**p;
		int64_t*i;
		double*d;
	};
};
typedef struct stack stack;
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
void pokei(stack*st,int n,int64_t x){
	*(int64_t*)peek(st,n) = x;
}
void pokev(stack*st,int n,void*x){
	*(void**)peek(st,n) = x;
}
int64_t peeki(stack*st,int n){
	return *(int64_t*)peek(st,n);
}
void*peekv(stack*st,int n){
	return *(void**)peek(st,n);
}
int64_t popi(stack*st){
	return *(int64_t*)pop(st);
}
void*popv(stack*st){
	return *(void**)pop(st);
}
void pushi(stack*st,int64_t i){
	*(int64_t*)push(st) = i;
}
void pushv(stack*st,void*p){
	*(void**)push(st) = p;
}
void pushptrsz(stack*st){
	pushi(st,sizeof(void*));
}
void add(stack*st){
	int64_t*slot=peek(st,2);
	*slot+=popi(st);
}
void sub(stack*st){
	int64_t*slot=peek(st,2);
	*slot-=popi(st);
}
void mul(stack*st){
	int64_t*slot=peek(st,2);
	*slot*=popi(st);
}
void divmod(stack*st){
	int64_t a=peeki(st,2),b=peeki(st,1);
	if (b != 0){
		pokei(st,2,a/b);
		pokei(st,2,a%b);
	}
}
void neg(stack*st){
	pokei(st,1,-peeki(st,1));
}
void sform(stack*st){
	if (st->s<2) return;
	const int64_t popx = peeki(st,1), newbase = peeki(st,2);
	if (popx>st->s || newbase>st->s) return;
	const size_t oidx = st->s-popx-2, bidx = st->s-newbase-2;
	int64_t i=0;
	do st->i[oidx+i]=st->i[oidx-st->i[oidx+i]]; while(++i<popx);
	memmove(st->i+bidx, st->i+oidx, popx*8);
	st->s=bidx+popx;
}
void _exec(stack*st){
	vmexec(st, st->p[st->s-1]);
}
void _if(stack*st){
	vmexec(st, (st->i[st->s-1] ? st->p[st->s-2] : st->p[st->s-3]));
}
void printint(stack*st){
	printf("%" PRId64,popi(st));
}
void printchr(stack*st){
	putchar(popi(st));
}
void printstr(stack*st){
	printf("%s",popv(st));
}
const struct builtin{
	const char*op;
	void(*f)(struct stack*);
}builtin[]={
	{"+",add},
	{"-",sub},
	{"*",mul},
	{"%/",divmod},
	{"neg",neg},
	{"$",sform},
	{"if",_if},
	{"()",_exec},
	//{"bit&",band},
	//{"bit|",bor},
	//{"bit^",bxor},
	{"printint",printint},
	{"printchr",printchr},
	{"print",printstr},
	{"ptrsz",pushptrsz}
};
size_t faecount;
char**faewords;
const char*vmprelude = " : dup 1 1 1 $ : : pop 1 0 $ : ";
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
const char*tryfaewords(stack*st,const char*code){
	const char*r;
	for(int i=0; i<faecount; i++){
		if (r=isfaeop(code, faewords[i]+1)){
			vmexec(st, faewords[i]+(r-code)+1);
			return r;
		}
	}
	return 0;
}
const char*defword(const char*code){
	faewords = realloc(faewords, sizeof(char*)*(++faecount));
	const char*c2 = code;
	while(*c2 != ':' || c2[1] != ' ') c2++;
	faewords[faecount-1] = malloc(c2-code+1);
	memcpy(faewords[faecount-1], code-1, c2-code+2);
	faewords[faecount-1][c2-code+1]=0;
	return c2+2;
}
void vmexec(struct stack*st,const char*code){
	for (;;){
		while(*code == ' ') code++;
		if (!*code) return;
		if (isdigit(*code)){
			while(*++code!=' ');
			int64_t x=0,n=1;
			const char*c2=code;
			while(*--c2!=' '){
				x+=(*c2&15)*n;
				n*=10;
			}
			pushi(st,x);
			continue;
		}
		if (*code == ':' && code[1] == ' '){
			code = defword(code+2);
			continue;
		}
		const char*r=trybuiltins(st,code);
		if (!r) r=tryfaewords(st,code);
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
	vmexec(&st, vmprelude);
	vmexec(&st, code);
	while(faecount--) free(faewords[faecount]);
	free(faewords);
	free(st.p);
	if (prefix || postfix){
		free((void*)code);
	}
}
