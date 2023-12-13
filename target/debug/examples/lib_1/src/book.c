// #include <stdio.h>
#include <stdint.h>
#include "../lib.h"
#define hello(x) ({ \
	int i = 0;     \
	i; \
})

/**
 * @brief
 * returns a copy of the default book.
 * EXECUTING: file --> function --> num. yaml --> test name
 * <!-- 
 * #!cesty;
 * ---  
 * info:
 *     standalone: true
 *     warn: true
 *     run: false
 * 
 * test:
 *     - name: cool_tests__com
 *       code: |
 *         struct innerbook book = my_book();
 *         if(book.author == BOOK_AUTHOR
 *         && book.name == BOOK_NAME
 *         && book.sold_amount == BOOK_SALES ) {
 *           printf("true\n");
 *           return true;
 *         }
 *         printf("false\n");
 *         return false;
 *       expect: true
 * 
 * prerun:
 *     - echo "hello"
 * 
 * include:
 *     - <assert.h>
 *     - <stdio.h>
 * 
 * compiler:
 *   name: gcc
 *   libraries: 
 *     append: false
 *     new: -lm
 *   flags: 
 *     append: false
 *     new: -std=c11
 * ... 
 * --->
 */
struct innerbook my_book()
{
	// int foobar(int i) {
	// 	return i+2;
	// }

	// int i = foobar(foo(bar(2)));
	return (struct innerbook){
		.author = BOOK_AUTHOR,
		.name = BOOK_NAME,
		.sold_amount = BOOK_SALES
	};

}

int fooishlybarful(int i) {

	return 100+i;

}