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
 * <!-- #!cesty;
 * ---  
 * info:
 *     standalone: true
 *     warn: true
 *     run: true
 * test:
 *     - name: def
 *       code: |
 *         struct innerbook = my_book();
 *         if(innerbook.author == BOOK_AUTHOR
 *         && innerbook.name == BOOK_NAME
 *         && innerbook.sold_amount == BOOK_SALES ) {
 *           return true;
 *         }
 *         return false;
 *       expect: true
 * 
 * prerun:
 *     - make -C ".."
 * 
 * compiler:
 *   name: gcc
 *   libraries: 
 *     append: true
 *     new: -lncursesw
 *   flags: 
 *     append: true
 *     new: -std=c11
 * ... 
 * --->
 */
struct innerbook my_book()
{
	int foobar(int i) {
		return i+2;
	}

	int i = foobar(foo(bar(2)));
	return (struct innerbook){
		.author = BOOK_AUTHOR,
		.name = BOOK_NAME,
		.sold_amount = BOOK_SALES+i
	};

}
`
int fooishlybarful(int i) {

	return 100+i;

}