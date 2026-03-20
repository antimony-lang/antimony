/* START builtins */
#include "stdio.h"
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

/* --- Core builtins --- */

void _printf(char *msg)
{
    printf("%s", msg);
}

void _exit(int code)
{
    exit(code);
}

char* _str_concat(char *s1, char *s2)
{
    char *result = malloc(strlen(s1) + strlen(s2) + 1);
    strcpy(result, s1);
    strcat(result, s2);
    return result;
}

/* --- Standard library equivalents for C target ---
 * These correspond to the Antimony stdlib (lib/) functions, which are
 * only compiled for the JS target.  For C we provide C implementations
 * so programs can call println/print/assert/exit without the Antimony
 * stdlib being included.
 */

void print(char* arg)
{
    printf("%s", arg);
}

static void _println_str(char* msg)  { printf("%s\n", msg); }
static void _println_int(int x)      { printf("%d\n", x); }
static void _println_bool(bool x)    { printf("%s\n", x ? "true" : "false"); }
static void _println_ptr(void* p)    { printf("%p\n", p); }

/* Type-generic println using C11 _Generic */
#define println(x) _Generic((x),    \
    char*: _println_str,            \
    int:   _println_int,            \
    bool:  _println_bool,           \
    void*: _println_ptr             \
)(x)

static void _assert_impl(bool condition)
{
    if (!condition) {
        printf("Assertion failed\n");
        exit(1);
    }
}

/* Variadic wrapper so both assert(cond) and assert(cond, msg) compile.
 * The optional message argument is accepted but ignored (JS compat). */
#define assert(cond, ...) _assert_impl(cond)

void _antimony_exit(int code)
{
    exit(code);
}

/* len() for null-terminated int arrays (sentinel 0 at end) */
int len(int* arr)
{
    int c = 0;
    while (arr[c]) { c++; }
    return c;
}

/* END builtins */
