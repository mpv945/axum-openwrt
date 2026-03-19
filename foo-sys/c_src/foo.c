#include "foo.h"
#include <stdlib.h>
#include <string.h>

// c++ 添加方法 -> 添加方法foo_sqrt
double foo_sqrt(double x) {
    return sqrt(x);  // 调用系统 math 库
}

int32_t foo_add(int32_t a, int32_t b) {
    return a + b;
}

const char* foo_hello() {
    return "Hello from C!";
}

char* foo_alloc_string() {
    const char* msg = "Allocated from C";
    char* buf = (char*)malloc(strlen(msg) + 1);
    strcpy(buf, msg);
    return buf;
}

void foo_free(char* ptr) {
    free(ptr);
}