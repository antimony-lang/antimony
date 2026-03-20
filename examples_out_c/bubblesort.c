#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

/* START builtins */
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

char* _str_concat(char *s1, char *s2)
{
    char *result = malloc(strlen(s1) + strlen(s2) + 1);
    strcpy(result, s1);
    strcat(result, s2);
    return result;
}

/* END builtins */
int main(void);
int len(int* arr);
int* rev(int* arr);
void assert(bool condition);
void print(char* arg);
void println(char* msg);
void exit(int code);

int main(void) {
    int* arr = {2, 5, 3, 1, 4};
    int n = len(arr);
    int c = 0;
    while (c < n) {
    int d = 0;
    while (d < n - c - 1) {
    int current = arr[d];
    int next = arr[d + 1];
    if (current > next) {
    int swap = arr[d];
    arr[d] = arr[d + 1];
    arr[d + 1] = swap;
}
;
    d += 1;
}
;
    c += 1;
}
;
    println(arr);
return 0;
}

int len(int* arr) {
    int c = 0;
    while (arr[c]) {
    c += 1;
}
;
    return c;
}

int* rev(int* arr) {
    int l = len(arr);
    int* new_arr = {};
    int i = 0;
    int j = l;
    while (i < l) {
    new_arr[i] = arr[j];
    i = i - 1;
    j = j - 1;
}
;
    return new_arr;
}

void assert(bool condition) {
    if (condition == false) {
    println("Assertion failed");
    exit(1);
}
;
}

void print(char* arg) {
    _printf(arg);
}

void println(char* msg) {
    print(_str_concat(msg, "\n"));
}

void exit(int code) {
    _exit(code);
}

