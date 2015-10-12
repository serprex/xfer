#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
#include "linenoise/linenoise.h"

void pr(const char*text){
	char ch;
	while(ch=*text){
		putchar(ch);
		fflush(stdout);
		usleep(1000*(
			ch=='\n' ? 20 :
			ch=='.' ? 50 :
			ch=='*' ? 90 :
			10));
		text++;
	}
}
void prline(const char*text){
	pr(text);
	putchar('\n');
}
void clearprev(){
	printf("\x1B[A\x1B[2K");
}
bool iscmd(char*line, char*cmp){
	for(;;){
		if (!*cmp) return true;
		if (*line != *cmp) return false;
		line++;
		cmp++;
	}
}
int strcountchr(char*str,char c){
	int n = 0;
	for(;;){
		if (*str == c) n++;
		if (!*++str) return n;
	}
}

void calc(const char*line){
	pr(line);
}

char rootdir[1024];
char pwdbuf[1024];

int main(int argc,char**argv){
	if (chdir("fs")) return -1;
	getcwd(rootdir, 1024);
	prline("HackNetZyr0\\ERROR527:DATAJACK MEMORY OVERLOAD");
	prline("USERNAME ARMITAGE807");
	prline("PASSWORD SHADOW24");
	prline("EMPLOYEE EXECUTIVE");
	prline("");
	prline("VERIFYING");
	for(int i=0; i<6; i++){
		usleep(100000);
		pr("*");
	}
	prline("\nACCESS GRANTED\n\nHackNetZyr0\\INTEGRATING H:\\");
	for(int i=0; i<8; i++){
		usleep(100000);
		pr("*");
	}
	prline("");
	int cwddep = 0;
	char* prompt = "> ";
	char* line;
	while ((line = linenoise(prompt)) != NULL){
		if (*line) linenoiseHistoryAdd(line);
		if (iscmd(line, "..")){
			if (cwddep>0 && !chdir("..")) cwddep--;
		}else if (iscmd(line, "cd")){
			if (strlen(line) == 2){
				if (!chdir(rootdir)) cwddep = 0;
			}else if (!strstr(line, "..") && !strstr(line,"/")){
				if (!chdir(line+3)) cwddep++;
			}
		}else if (iscmd(line, "pwd")){
			getcwd(pwdbuf, 1024);
			pr(pwdbuf+strlen(rootdir));
		}else if (iscmd(line, "ls")){
			system("ls");
		}
		free(line);
	}
}