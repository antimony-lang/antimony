/* START builtins */
#include "stdio.h"
#include <stdbool.h>

void _printf(char *msg)
{
    printf("%s", msg);
}

void _exit(int code)
{
    exit(code);
}

/* END builtins */
