#ifndef FOO_H
#define FOO_H

#include <stdint.h>
#pragma once
#include <math.h>  // 系统库

// c++ 添加方法 -> 申明方法foo_sqrt
double foo_sqrt(double x);

// 简单加法
int32_t foo_add(int32_t a, int32_t b);

// 返回字符串（静态内存，不需要释放）
const char* foo_hello();

// 分配字符串（需要调用 foo_free 释放）
char* foo_alloc_string();

// 释放函数
void foo_free(char* ptr);

#endif