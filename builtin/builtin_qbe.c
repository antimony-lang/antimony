#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void _printf(char *msg)
{
    printf("%s", msg);
}

void _exit(int code)
{
    exit(code);
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
