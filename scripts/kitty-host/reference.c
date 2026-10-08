/* Independent Kitty sender: no reactive-tui encoder or repaint request. */
#define _POSIX_C_SOURCE 200809L
#include <errno.h>
#include <openssl/evp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <zlib.h>

static void fail(const char *message) {
    fprintf(stderr, "Kitty reference: %s\n", message);
    exit(2);
}

static double now(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC, &value)) fail("clock_gettime failed");
    return value.tv_sec + value.tv_nsec / 1e9;
}

static void pause_ms(long milliseconds) {
    struct timespec delay = {milliseconds / 1000, (milliseconds % 1000) * 1000000};
    while (nanosleep(&delay, &delay)) {
        if (errno != EINTR) fail("nanosleep failed");
    }
}

static void send_bytes(const char *data, size_t size) {
    while (size) {
        ssize_t count = write(STDOUT_FILENO, data, size);
        if (count < 0 && errno == EINTR) continue;
        if (count <= 0) fail("terminal write failed");
        data += count;
        size -= (size_t)count;
    }
}

static void send_text(const char *text) {
    send_bytes(text, strlen(text));
}

static void send_image(int stage) {
    unsigned char rgba[128 * 64 * 4];
    const unsigned char colors[2][2][4] = {
        {{255, 0, 0, 255}, {0, 0, 255, 255}},
        {{0, 255, 0, 255}, {255, 255, 0, 255}},
    };
    for (size_t y = 0; y < 64; y++) {
        for (size_t x = 0; x < 128; x++) {
            memcpy(rgba + (y * 128 + x) * 4, colors[stage][x >= 64], 4);
        }
    }
    unsigned char compressed[1024], encoded[2048];
    uLongf length = sizeof(compressed);
    if (compress2(compressed, &length, rgba, sizeof(rgba), Z_BEST_COMPRESSION) != Z_OK)
        fail("reference image compression failed");
    int encoded_length = EVP_EncodeBlock(encoded, compressed, (int)length);
    if (encoded_length <= 0 || (size_t)encoded_length >= sizeof(encoded))
        fail("reference image encoding failed");
    char command[4096];
    int size = snprintf(command, sizeof(command),
        "\033_Ga=T,f=32,s=128,v=64,i=%d,q=2,z=0,o=z;%s\033\\", stage + 1, encoded);
    if (size < 0 || (size_t)size >= sizeof(command)) fail("reference command overflow");
    send_bytes(command, (size_t)size);
}

int main(int argc, char **argv) {
    if (argc != 3 || strcmp(argv[2], "kitty")) fail("expected stage-file and kitty");
    const char *headers[] = {
        "\033[?1049h\033[?25l\033[40m\033[2J\033[HIMAGE PROTOCOL STAGE 0\033[3;3H",
        "\033[2J\033[HIMAGE PROTOCOL STAGE 1\033[9;25H",
        "\033[2J\033[HIMAGE PROTOCOL STAGE 2",
    };
    int shown = -1;
    double deadline = now() + 45;
    while (now() < deadline) {
        FILE *file = fopen(argv[1], "r");
        if (!file) fail("cannot read stage file");
        int stage;
        int count = fscanf(file, "%d", &stage);
        int close_status = fclose(file);
        if (count != 1 || close_status || stage < 0 || stage > 3) fail("invalid stage file");
        if (stage == 3) {
            send_text("\033[0m\033[?25h\033[?1049l");
            return shown == 2 ? 0 : 2;
        }
        if (stage != shown) {
            if (stage != shown + 1) fail("stage skipped or repeated out of order");
            send_text(headers[stage]);
            if (stage < 2) {
                /* Settle text before ONE image transmission. A later repaint
                 * would conceal the host's stale image-layer decision. */
                pause_ms(1000);
                send_image(stage);
            }
            shown = stage;
        }
        pause_ms(20);
    }
    fail("stage driver timed out");
}
