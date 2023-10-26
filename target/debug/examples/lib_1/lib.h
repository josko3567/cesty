#pragma once

#define BOOK_AUTHOR "Me"
#define BOOK_NAME "Yipee"
#define BOOK_SALES 95923

struct innerbook {
    char * author      ;
    char * name        ;
    int64_t sold_amount;
};

int foo(int i);
int bar(int i);