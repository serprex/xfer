#include <stdlib.h>
#include <string.h>
#include "vm.h"
#include "sys.h"
struct fnode{
	unsigned len, type;
	const char*name;
	fnode*parent;
	union{
		fnode**children;
		char*text;
		void*data;
	};
};

fnode*mkfnode(fnode*parent, const char*name, unsigned type, unsigned len, const void*data){
	fnode*n = malloc(sizeof(fnode));
	n->len = len;
	n->type = type;
	n->name = name;
	n->parent = parent;
	n->children = 0;
	if (len){
		n->data = malloc(len);
		if (data && n->data) memcpy(n->data, data, len);
	}
	if (parent && (parent->type&f_dir)){
		parent->children = realloc(parent->children, (parent->len+=sizeof(fnode*)));
		parent->children[len/sizeof(fnode*)-1] = n;
	}
	return n;
}
fnode*mkfdir(fnode*parent, const char*name, unsigned mask){
	return mkfnode(parent, name, f_dir|mask, 0, 0);
}
fnode*mkftext(fnode*parent, const char*name, unsigned mask, const char*text){
	return mkfnode(parent, name, f_file|mask, strlen(text), text);
}

fnode*resolve(fnode*from,char*name){
	if (!name) return 0;
	if (!*name) return from;
	if (*name == '.' && (!name[1] || name[1] == '/')){
		return name[1] ? resolve(from, name+2) : from;
	}
	if (*name == '.' && name[1] == '.' && (!name[2] || name[2] == '/')){
		return name[2] ? resolve(from->parent, name+3) : from->parent;
	}
	if (!from->children) return 0;
	for(fnode**childp = from->children; *childp; childp++){
		int i=0;
		const char *cname = (*childp)->name;
		for(int i=0;;i++){
			if (!cname[i] && name[i] == '/') return resolve(*childp, name+i+1); 
			if (cname[i] != name[i]) break;
		}
	}
	return 0;
}
fnode*fstate;
void open(stack*st){
	char *name = peekv(st, 1);
	fnode*fn = resolve(fstate, name);
	pokev(st, 1, fn);
}

syscall syscalls[]={
	//open,
};