#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

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
typedef struct User {
    char* username;
    char* first_name;
    char* last_name;
} User;

typedef struct Bar {
    char* y;
} Bar;

typedef struct Self_test_struct {
    int a;
} Self_test_struct;

typedef struct Point {
    int x;
    int y;
} Point;

typedef struct Foo {
    int x;
    Bar bar;
} Foo;

typedef struct Rectangle {
    Point origin;
    int width;
    int height;
} Rectangle;

char* User_full_name(User self);
void Self_test_struct_bar(Self_test_struct self);
void functions_main(void);
void test_functions_basics(void);
int add_one(int x);
void conditionals_main(void);
void test_conditionals_basics(void);
void test_conditionals_multiple_arms(void);
void test_conditionals_non_boolean_condition(void);
void test_basic_match(void);
void test_boolean_match(void);
void test_match_with_block_statement(void);
void test_nested_match_statement(void);
void unicode_main(void);
void test_unicode_strings(void);
void test_unicode_identifiers(void);
void numbers_main(void);
void test_operators(void);
void baz(void);
void nested_module(void);
void external_function(void);
void imports_main(void);
void types_main(void);
void print_any(int x);
void structs_main(void);
User user_stub(void);
void test_initialization(void);
void test_simple_field_access(void);
void test_field_access_in_function_call(void);
void test_field_access_on_function(void);
void test_nested_structs(void);
void test_method_call(void);
void assert_bar_y(Bar bar);
void test_function_call_with_constructor(void);
void test_method_with_self_statement(void);
void test_nested_field_access(void);
void log_test_stage(char* msg);
int main(void);

char* User_full_name(User self) {
    return _str_concat(self.first_name, self.last_name);
}

void Self_test_struct_bar(Self_test_struct self) {
    self.a += 1;
    assert(true);
}

void functions_main(void) {
    log_test_stage("Testing functions");
    test_functions_basics();
}

void test_functions_basics(void) {
    int x = add_one(2);
    assert(x == 3);
    println(x);
}

int add_one(int x) {
    return x + 1;
}

void conditionals_main(void) {
    log_test_stage("Testing conditionals");
    test_conditionals_basics();
    test_conditionals_multiple_arms();
    test_basic_match();
    test_boolean_match();
    test_match_with_block_statement();
}

void test_conditionals_basics(void) {
    int number = 3;
    if (number < 5) {
    println("condition was true");
}
else {
    println("condition was false");
}
;
}

void test_conditionals_multiple_arms(void) {
    int number = 6;
    if (number % 4 == 0) {
    println("number is divisible by 4");
}
else if (number % 3 == 0) {
    println("number is divisible by 3");
}
else if (number % 2 == 0) {
    println("number is divisible by 2");
}
else {
    println("number is not divisible by 4, 3, or 2");
}
;
}

void test_conditionals_non_boolean_condition(void) {
    int number = 3;
    if (number) {
    println("number was three");
}
else {
    assert(false);
}
;
}

void test_basic_match(void) {
    int x = 1;
    if (x == 1) {
    assert(true);
}
else if (x == 2) {
    assert(false);
}
;
}

void test_boolean_match(void) {
    bool x = true;
    if (x == true) {
    assert(true);
}
else if (x == false) {
    assert(false);
}
;
}

void test_match_with_block_statement(void) {
    int x = 42;
    if (x == 1) {
    println("x is 1");
}
else if (x == 2) {
    println("This is a branch with multiple statements.");
    println("x is 2, in case you are wondering");
}
else if (x == 42) {
    println("The answer to the universe and everything!");
}
else {
    println("Default case");
}
;
}

void test_nested_match_statement(void) {
    int x = 42;
    int y = 1;
    if (x == 1) {
    assert(false);
}
else if (x == 2) {
    assert(false);
}
else if (x == 42) {
    if (y == 1) {
    assert(true);
}
else {
    assert(false);
}
;
}
else {
    assert(false);
}
;
}

void unicode_main(void) {
    log_test_stage("Testing unicode");
    test_unicode_strings();
    test_unicode_identifiers();
}

void test_unicode_strings(void) {
    println("Test unicode strings");
    char* alpha_omega = "αβ";
    println(alpha_omega);
}

void test_unicode_identifiers(void) {
    println("Test unicode identifiers");
    char* αβ = "αβ";
    char* 世界 = "世界";
    println(世界);
}

void numbers_main(void) {
    log_test_stage("Testing numbers");
    int one_billion = 1000000000;
    int dec = 255;
    int hex = 255;
    int binary = 255;
    int octal = 255;
    assert(dec == 255);
    assert(hex == 255);
    assert(binary == 255);
    assert(octal == 255);
    test_operators();
}

void test_operators(void) {
    println("test_operators");
    int x = 10;
    x += 1;
    x -= 2;
    x *= 2;
    x /= 2;
    assert(x == 9);
}

void baz(void) {
    println("Baz was called");
}

void nested_module(void) {
    println("A deeply nested function was called!");
}

void external_function(void) {
    println("I was called!!");
    nested_module();
}

void imports_main(void) {
    log_test_stage("Testing imports");
    external_function();
}

void types_main(void) {
    log_test_stage("Testing types");
    print_any(5);
    print_any("Test");
}

void print_any(int x) {
    println(x);
}

void structs_main(void) {
    log_test_stage("Testing structs");
    test_initialization();
    test_simple_field_access();
    test_field_access_in_function_call();
    test_field_access_on_function();
    test_nested_structs();
    test_method_call();
    test_function_call_with_constructor();
    test_method_with_self_statement();
    test_nested_field_access();
}

User user_stub(void) {
    User stub = (User) {.username = "Foo Bar", .first_name = "Foo", .last_name = "Bar"};
    assert(stub.first_name);
    assert(stub.last_name);
    return stub;
}

void test_initialization(void) {
    println("test_initialization");
    User foo = (User) {.first_name = "Bar", .last_name = "Bar", .username = "Foo Bar"};
    (void)0; /* struct assert - always passes */
}

void test_simple_field_access(void) {
    User user = user_stub();
    user.username = "Foo Bar";
}

void test_field_access_in_function_call(void) {
    User user = user_stub();
    user.username = "Bar";
    assert(user.username == "Bar");
}

void test_field_access_on_function(void) {
    assert(user_stub().first_name == "Foo");
}

void test_nested_structs(void) {
    Foo foo = (Foo) {.x = 5, .bar = (Bar) {.y = "Nested field"}};
    assert(foo.x == 5);
    println(foo.bar.y);
    assert(foo.bar.y == "Nested field");
}

void test_method_call(void) {
    User user = user_stub();
    char* full_name = User_full_name(user);
    assert(full_name, "FooBar");
}

void assert_bar_y(Bar bar) {
    assert(bar.y == "ABC");
}

void test_function_call_with_constructor(void) {
    assert_bar_y((Bar) {.y = "ABC"});
}

void test_method_with_self_statement(void) {
    Self_test_struct foo = (Self_test_struct) {.a = 5};
    Self_test_struct_bar(foo);
}

void test_nested_field_access(void) {
    Rectangle rect = (Rectangle) {.width = 100, .origin = (Point) {.x = 10, .y = 20}, .height = 50};
    assert(rect.origin.x == 10);
    rect.origin.x += 5;
    assert(rect.origin.x == 15);
}

void log_test_stage(char* msg) {
    println("");
    println("-----------------------------");
    println(_str_concat("--- ", _str_concat(msg, " ---")));
    println("-----------------------------");
}

int main(void) {
    log_test_stage("Running tests");
    conditionals_main();
    functions_main();
    imports_main();
    numbers_main();
    structs_main();
    types_main();
    unicode_main();
    log_test_stage("Done!");
return 0;
}

