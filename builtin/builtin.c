/* START builtins */
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>

void _printf(char *msg)
{
    printf("%s", msg);
}

void _exit(int code)
{
    exit(code);
}

// Antimony array layout (QBE/C backends): { i64 length @ offset 0; elements @ offset 8.. }
int64_t _array_len(int64_t *arr)
{
    return arr[0];
}

/* END builtins */
