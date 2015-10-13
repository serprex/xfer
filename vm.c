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
	};
};
typedef struct stack stack;
void pop(stack*st){
	if(st->s) st->s--;
}
int64_t popi(stack*st){
	pop(st);
	return st->i[st->s];
}
void*popv(stack*st){
	pop(st);
	return st->p[st->s];
}
void pushfix(stack*st){
	if (st->s>st->z){
		st->z = st->s*2;
		st->p = realloc(st->p, st->z*sizeof(void*));
	}
}
void pushx(stack*st,size_t x){
	st->s+=x;
	pushfix(st);
}
void push(stack*st){
	pushx(st,1);
}
void pushi(stack*st,int64_t i){
	push(st);
	st->i[st->s-1] = i;
}
void pushv(stack*st,void*p){
	push(st);
	st->p[st->s-1] = p;
}
void pushptrsz(stack*st){
	pushi(st,sizeof(void*));
}
void swap(stack*st){
	void*t=st->p[st->s-1];
	st->p[st->s-1]=st->p[st->s-2];
	st->p[st->s-2]=t;
}
void duptop(stack*st){
	pushi(st,st->i[st->s-1]);
}
void add(stack*st){
	int64_t*slot=st->i+st->s-2;
	*slot+=popi(st);
}
void sub(stack*st){
	int64_t*slot=st->i+st->s-2;
	*slot-=popi(st);
}
void mul(stack*st){
	int64_t*slot=st->i+st->s-2;
	*slot*=popi(st);
}
void divmod(stack*st){
	int64_t a=st->i[st->s-2],b=st->i[st->s-1];
	if (b != 0){
		st->i[st->s-2]=a/b;
		st->i[st->s-1]=a%b;
	}
}
void sform(stack*st){
	if (st->s<2) return;
	int64_t popx = st->i[st->s-1],
		newbase = st->i[st->s-2];
	if (popx>st->s || newbase>st->s) return;
	const size_t oidx = st->s-popx, bidx = st->s-newbase;
	int64_t i=0;
	do st->i[oidx+i]=st->i[oidx-st->i[oidx+i]]; while(++i<popx);
	i=0;
	do st->i[bidx+i]=st->i[oidx+i]; while(++i<popx);
	st->s=bidx+popx;
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
	{"$",sform},
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
const char*prelude = " : dup 0 3 4 $ : : pop 3 3 $ : : neg 0 1 - * : ";
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
const char*tryfaewords(stack*st,const char*code){
	const char*r;
	for(int i=0; i<faecount; i++){
		if (r = isop(code, faewords[i])){
			vmexec(st, faewords[i]+(r-code));
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
	while (*code){
		while(*code == ' ')code++;
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
	}
}
void vmstart(const char*code){
	int codelen = strlen(code), prefix = code[0] != ' ', postfix = code[codelen-1] != ' ';
	if (prefix || postfix){
		char*newcode = malloc(codelen+prefix+postfix+1);
		if (prefix) newcode[0] = ' ';
		memcpy(newcode+postfix, code, codelen);
		if (postfix) newcode[codelen+postfix+prefix-1] = ' ';
		newcode[codelen+prefix+postfix] = 0;
		code=newcode;
	}
	struct stack st = {};
	vmexec(&st, prelude);
	vmexec(&st, code);
	while(faecount--) free(faewords[faecount]);
	free(faewords);
	free(st.p);
	if (prefix || postfix){
		free((void*)code);
	}
}
