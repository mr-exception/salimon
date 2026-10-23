#define _GNU_SOURCE
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

/* A small parent avoids inheriting the Python driver's RSS high-water floor. */
int main(int argc, char **argv) {
    if (argc < 3) { fprintf(stderr, "usage: measure-rss output.json executable [args...]\n"); return 2; }
    struct timespec start, end;
    if (clock_gettime(CLOCK_MONOTONIC, &start) != 0) return 2;
    pid_t child = fork();
    if (child < 0) return 2;
    if (child == 0) { execv(argv[2], argv + 2); perror("execv"); _exit(127); }
    int status;
    struct rusage usage;
    pid_t result;
    do { result = wait4(child, &status, 0, &usage); } while (result < 0 && errno == EINTR);
    if (result < 0 || clock_gettime(CLOCK_MONOTONIC, &end) != 0) return 2;
    int code = WIFEXITED(status) ? WEXITSTATUS(status) : 128 + WTERMSIG(status);
    FILE *out = fopen(argv[1], "wx");
    if (!out) { perror("measurement output"); return 2; }
    double wall_ms = (end.tv_sec-start.tv_sec)*1000.0 + (end.tv_nsec-start.tv_nsec)/1000000.0;
    fprintf(out, "{\"exit_code\":%d,\"wall_ms\":%.9f,\"max_rss_kib\":%ld,"
                 "\"user_seconds\":%.9f,\"system_seconds\":%.9f,"
                 "\"voluntary_switches\":%ld,\"involuntary_switches\":%ld}\n",
            code, wall_ms, usage.ru_maxrss,
            usage.ru_utime.tv_sec+usage.ru_utime.tv_usec/1000000.0,
            usage.ru_stime.tv_sec+usage.ru_stime.tv_usec/1000000.0,
            usage.ru_nvcsw, usage.ru_nivcsw);
    if (fclose(out) != 0) return 2;
    return code;
}
