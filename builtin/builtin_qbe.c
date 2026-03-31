#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void _printf(char *msg)
{
    printf("%s", msg);
}

void _exit(int code)
{
    fflush(stdout);
    fflush(stderr);
    _Exit(code);
}

char *_int_to_str(long n)
{
    char *buf = malloc(32);
    snprintf(buf, 32, "%ld", n);
    return buf;
}

char *_str_concat(char *a, char *b)
{
    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    char *result = malloc(len_a + len_b + 1);
    memcpy(result, a, len_a);
    memcpy(result + len_a, b, len_b + 1);
    return result;
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
