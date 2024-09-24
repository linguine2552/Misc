#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <err.h>
#include <string.h>
#include <time.h>
#include <dev/wscons/wsconsio.h>
#include <sys/ioctl.h>
#include <errno.h>

#define CONSOLE "/dev/ttyC0"

void print_error(const char *msg) {
    fprintf(stderr, "%s: %s\n", msg, strerror(errno));
}

int get_brightness(int fd) {
    struct wsdisplay_param param;
    param.param = WSDISPLAYIO_PARAM_BRIGHTNESS;
    if (ioctl(fd, WSDISPLAYIO_GETPARAM, &param) == -1) {
        print_error("Cannot get brightness");
        return -1;
    }
    return param.curval;
}

int set_brightness(int fd, int value) {
    struct wsdisplay_param param;
    param.param = WSDISPLAYIO_PARAM_BRIGHTNESS;
    param.curval = value;
    if (ioctl(fd, WSDISPLAYIO_SETPARAM, &param) == -1) {
        print_error("Cannot set brightness");
        return -1;
    }
    return 0;
}

void fade_brightness(int fd, int start, int end, int steps, int delay_ms) {
    int i, brightness;
    for (i = 0; i <= steps; i++) {
        brightness = start + (end - start) * i / steps;
        set_brightness(fd, brightness);
        usleep(delay_ms * 1000);
    }
}

int main(int argc, char *argv[]) {
    int fd = open(CONSOLE, O_RDWR);
    if (fd == -1) {
        print_error("Cannot open console");
        return 1;
    }

    if (argc < 2) {
        printf("Usage: %s [get|set|fade|night] [value]\n", argv[0]);
        return 1;
    }

    if (strcmp(argv[1], "get") == 0) {
        printf("Current brightness: %d\n", get_brightness(fd));
    } else if (strcmp(argv[1], "set") == 0 && argc == 3) {
        int value = atoi(argv[2]);
        if (set_brightness(fd, value) == 0) {
            printf("Brightness set to: %d\n", value);
        }
    } else if (strcmp(argv[1], "fade") == 0 && argc == 3) {
        int end = atoi(argv[2]);
        int start = get_brightness(fd);
        fade_brightness(fd, start, end, 50, 20);  // 50 steps, 20ms delay
        printf("Faded brightness from %d to %d\n", start, end);
    } else if (strcmp(argv[1], "night") == 0) {
        int start = get_brightness(fd);
        fade_brightness(fd, start, 10, 50, 20);  // Fade to 10% brightness
        printf("Night mode activated (10%% brightness)\n");
    } else {
        printf("Invalid command or missing argument\n");
    }

    close(fd);
    return 0;
}
