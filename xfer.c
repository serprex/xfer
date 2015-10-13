#include "xfer.h"

void pr(const char*text){
	char ch;
	while(ch=*text){
		putchar(ch);
		fflush(stdout);
		usleep(16384*(
			ch=='\n' ? 6 :
			ch=='.' ? 3 :
			1));
		text++;
	}
}
void prslow(const char*text, int msec){
	char ch;
	while(ch=*text){
		putchar(ch);
		fflush(stdout);
		usleep(msec);
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
bool iscmd(char*restrict line, char*restrict cmp){
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

char rootdir[512], pwdbuf[512];

int main(int argc,char**argv){
	if (chdir("fs")) return -1;
	getcwd(rootdir, 1024);
	int cwddep = 0;
	char *prompt = "> ", *line;
	int firstLoad = linenoiseHistoryLoad("/mnt/jack/.linenoise");
	if (firstLoad){
		// todo disable stdin echo for this bit
		pr("login: ");
		usleep(563423);
		prline("atage");
		usleep(8192);
		pr("password: ");
		usleep(731201);
		prline("\nLogged in");
		pr("> ");
		usleep(69105);
		prslow("mount /dev/jack /mnt/jack\n", 4096);
		usleep(3333333);
		prline("Warning: biomem corrupted while mounting device");
		usleep(999999);
		prline("Success");
	}else{
		bool loggedIn = false;
		while (!loggedIn && (line = linenoise("login: "))){
			char *pwd = linenoise("password: ");
			usleep(731201);
			if (strcmp(line, "atage")){
				prline("Logged in");
				loggedIn = true;
			}else{
				prline("Incorrect username or password");
			}
			free(pwd);
			free(line);
		}
	}
	while (line = linenoise(prompt)){
		if (!*line) continue;
		linenoiseHistoryAdd(line);
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
		}else if (iscmd(line, "umount")){
			if (strcmp(line+6, "/mnt/jack")) break;
		}else if (iscmd(line, "exit")){
			break;
		}
		free(line);
	}
	return linenoiseHistorySave("/mnt/jack/.linenoise");
}