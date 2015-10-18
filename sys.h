typedef struct fnode fnode;
typedef void (*syscall)(vmscratch*,stack*);

#define f_file 1
#define f_dir 2
#define f_link 4
#define f_read 256
#define f_write 512
#define f_exec 1024

fnode*mkfnode(fnode*,const char*,unsigned,unsigned,const void*);
fnode*mkfdir(fnode*parent,const char*,unsigned);
fnode*mkftext(fnode*,const char*,unsigned,const char*);
