#include "xfer.h"

int main(int argc, char**argv){
	fnode*root = mkfdir(0, "", f_read);
	fnode*bin = mkfdir(root,"bin",f_read);
	fnode*home = mkfdir(root, "home", f_read);
	fnode*mnt = mkfdir(root, "mnt", f_read|f_write);
	fnode*dev = mkfdir(root, "dev", f_read);
	fnode*mntjack = mkfdir(mnt, "jack", f_read|f_write);
	fnode*binfalse = mkftext(bin, "false", f_read|f_exec, "0");
	fnode*bintrue = mkftext(bin, "true", f_read|f_exec, "1");
	//fnode*devjack;
	//fnode*devrandom = mkfreal(dev, "random", f_read|f_write, "/dev/urandom");
	//fnode*devzero = mkfreal(dev, "zero", f_read|f_write, "/dev/zero");
	//fnode*devnull = mkfreal(dev, "zero", f_read|f_write, "/dev/null");

	const char*path = argv[1] ? argv[1] : "fsinit";
	FILE*dst = fopen(path, "w");
}