#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/*
 * QBE runtime helpers that cannot be expressed cleanly in QBE IL:
 *
 *   _str_concat  — heap-allocates a new string from two inputs
 *   _int_to_str  — formats a long integer into a heap-allocated string
 *   _read_line   — reads one line from stdin into a heap-allocated buffer
 *
 * _printf, _exit, _strlen, and _parse_int are implemented directly in QBE IL
 * (see the RUNTIME_PREAMBLE in src/generator/qbe.rs) and are no longer here.
 */

char *_str_concat(char *a, char *b)
{
    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    char *result = malloc(len_a + len_b + 1);
    memcpy(result, a, len_a);
    memcpy(result + len_a, b, len_b + 1);
    return result;
}

char *_int_to_str(long n)
{
    char *buf = malloc(32);
    snprintf(buf, 32, "%ld", n);
    return buf;
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

/* _str_char_at(s, idx) — return the character at index idx as a 1-char string */
char *_str_char_at(char *s, long idx)
{
    char *buf = malloc(2);
    buf[0] = s[idx];
    buf[1] = '\0';
    return buf;
}

/* _str_substr(s, start, len) — extract a substring [start, start+len) */
char *_str_substr(char *s, long start, long len)
{
    char *buf = malloc(len + 1);
    memcpy(buf, s + start, len);
    buf[len] = '\0';
    return buf;
}
