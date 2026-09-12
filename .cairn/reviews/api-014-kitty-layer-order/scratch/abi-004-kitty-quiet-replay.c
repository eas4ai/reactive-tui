#define _DEFAULT_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/ioctl.h>
#include <sys/select.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

static double now(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec + t.tv_nsec / 1e9;
}
static void send_file(const char *name) {
    char path[256], data[65536];
    snprintf(path, sizeof(path), "/tmp/abi-004-kitty-frames-quiet/%s.bin", name);
    FILE *file = fopen(path, "rb");
    if (!file) exit(2);
    size_t n = fread(data, 1, sizeof(data), file);
    fclose(file);
    size_t offset = 0;
    while (offset < n) {
        ssize_t count = write(1, data + offset, n - offset);
        if (count < 0 && errno == EINTR) continue;
        if (count <= 0) exit(3);
        offset += count;
    }
}
int main(int argc, char **argv) {
    if (argc != 3) return 2;
    char path[4096];
    snprintf(path, sizeof(path), "%s/responses.log", argv[2]);
    FILE *log = fopen(path, "w");
    if (!log) return 2;
    struct termios saved, raw;
    if (tcgetattr(0, &saved)) return 2;
    raw = saved;
    cfmakeraw(&raw);
    if (tcsetattr(0, TCSANOW, &raw)) return 2;
    double start = now();
    send_file("prefix");
    int shown = -1;
    unsigned rows = 0, cols = 0;
    while (now() - start < 45) {
        FILE *stage_file = fopen(argv[1], "r");
        int stage = -1;
        if (stage_file) { int count = fscanf(stage_file, "%d", &stage); fclose(stage_file); if (count != 1) break; }
        if (stage < 0 || stage >= 3) break;
        struct winsize size = {0};
        if (ioctl(0, TIOCGWINSZ, &size)) break;
        if (rows != size.ws_row || cols != size.ws_col) {
            fprintf(log, "%.6f size %u %u pixels %u %u\n", now()-start, size.ws_row, size.ws_col, size.ws_xpixel, size.ws_ypixel);
            rows = size.ws_row; cols = size.ws_col;
        }
        if (stage != shown) {
            char name[2] = {(char)('0' + stage), 0};
            fprintf(log, "%.6f send stage %d\n", now()-start, stage);
            send_file(name); shown = stage;
        }
        fflush(log);
        fd_set reads; FD_ZERO(&reads); FD_SET(0, &reads);
        struct timeval timeout = {.tv_sec=0, .tv_usec=20000};
        if (select(1, &reads, NULL, NULL, &timeout) > 0) {
            char reply[8192]; ssize_t n = read(0, reply, sizeof(reply));
            fprintf(log, "%.6f reply stage %d: ", now()-start, stage);
            if (n > 0) fwrite(reply, 1, n, log);
            fputc('\n', log); fflush(log);
        }
    }
    send_file("suffix");
    tcsetattr(0, TCSANOW, &saved);
    fclose(log);
    return 0;
}
