/* START builtins */
/* These are standard C library headers available on all supported platforms.
 * stdlib.h was already an implicit dependency (exit); string.h is new, also
 * used by builtin_qbe.c from the start. */
#include "stdio.h"
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

void _printf(char *msg)
{
    printf("%s", msg);
}

void _exit(int code)
{
    exit(code);
}

int _strlen(char *s)
{
    return (int)strlen(s);
}

int _parse_int(char *s)
{
    return atoi(s);
}

char *_read_line()
{
    size_t capacity = 256;
    char *buf = malloc(capacity);
    if (!buf) return "";
    if (!fgets(buf, (int)capacity, stdin)) {
        buf[0] = '\0';
    } else {
        size_t n = strlen(buf);
        if (n > 0 && buf[n - 1] == '\n') buf[n - 1] = '\0';
    }
    return buf;
}

/* END builtins */
